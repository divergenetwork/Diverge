//! The log itself: lines handed over, and a task that writes them.

use std::path::{Path, PathBuf};

use tokio::fs::{File, OpenOptions};
use tokio::io::AsyncWriteExt as _;
use tokio::sync::mpsc;

use super::stamp;

/// The most one log file holds, in bytes: four mebibytes. A line that
/// would take a file past it begins a new file.
const FILE_BYTES: u64 = 4 * 1024 * 1024;

/// The most log files kept: the newest eight. With [`FILE_BYTES`], the
/// log holds at most thirty-two mebibytes.
const FILES: usize = 8;

/// The provider's log. The rules:
///
/// 1. **What is written.** One line for each of: a connection a peer
///    dialled, with the IP it came from, said before its handshake so
///    that a handshake that never ends is a line with nothing after
///    it; its handshake taken, with the identity it was taken as, or
///    not taken, with why; a peer the provider dialled, likewise;
///    every request a connection opens, by its tag — the byte the
///    protocol names its endpoints by — and never its body, so no
///    argument and no file's bytes are written, but for a `capacity`,
///    which every daemon asks every ten seconds and is left out;
///    and a connection's end, with how long it lasted. No channel
///    request, no response and no payload is written.
/// 2. **Where.** Under `logs/` in the provider's own directory, in a
///    file named for when it was made:
///    `provider-<year><month><day>T<hour><minute><second>.<millis>Z.log`,
///    in UTC. A provider begins a file at its start.
/// 3. **Bounded.** A file holds at most [`FILE_BYTES`]; a line that
///    would take it past that begins a new one. The newest [`FILES`]
///    files are kept, and every older one is deleted, when a file is
///    begun.
/// 4. **Repeats are counted, not written.** A line the same as the
///    one before it is not written again: the next line that differs
///    is preceded by one saying how many times the last was repeated.
///    A peer that redials every five seconds and is refused the same
///    way is two lines, not thousands.
/// 5. **Never in the way.** A line is handed to a queue and written by
///    a task of the log's own: nothing that says a line waits on the
///    disk. A line that cannot be written is dropped, and the provider
///    goes on.
/// 6. **Off.** With `logging` `false` in the configuration nothing is
///    written, no file is made, and a line said is dropped where it is
///    said.
#[derive(Debug)]
pub struct Log {
    /// The way to the writer: `None` where the log is off.
    lines: Option<mpsc::UnboundedSender<String>>,
}

impl Log {
    /// A log that writes nothing: rule 6.
    pub fn off() -> Log {
        Log { lines: None }
    }

    /// The log of a provider whose own directory is `dir`, on or off
    /// as `enabled` says. `logs/` is made if it is not there and the
    /// first file begun; a log that cannot begin its file is off, and
    /// the provider starts all the same.
    pub async fn start(dir: &Path, enabled: bool) -> Log {
        if !enabled {
            return Log::off();
        }
        let folder = dir.join("logs");
        let Some(writer) = Writer::begin(folder).await else {
            return Log::off();
        };
        let (lines, said) = mpsc::unbounded_channel();
        tokio::spawn(writer.write(said));
        Log { lines: Some(lines) }
    }

    /// One line, with the time it was said before it: rule 5.
    pub fn say(&self, line: String) {
        if let Some(lines) = &self.lines {
            let _ = lines.send(line);
        }
    }
}

/// The task that writes: the file it appends to, and what it needs to
/// keep rules 3 and 4.
struct Writer {
    /// `logs/`.
    folder: PathBuf,
    /// The file being written.
    file: File,
    /// How many bytes it holds.
    bytes: u64,
    /// The last line written, without its time, and how many times it
    /// was said again since.
    last: Option<(String, u64)>,
}

impl Writer {
    /// `folder` made, and a first file begun in it.
    async fn begin(folder: PathBuf) -> Option<Writer> {
        tokio::fs::create_dir_all(&folder).await.ok()?;
        let file = open(&folder).await?;
        Some(Writer { folder, file, bytes: 0, last: None })
    }

    /// Every line said, written in the order said, until nobody is
    /// left to say one.
    async fn write(mut self, mut said: mpsc::UnboundedReceiver<String>) {
        while let Some(line) = said.recv().await {
            // Rule 4.
            match &mut self.last {
                Some((last, repeats)) if *last == line => {
                    *repeats += 1;
                    continue;
                }
                Some((_, repeats)) if *repeats > 0 => {
                    let repeated = format!("the line before was repeated {repeats} more times");
                    self.append(&repeated).await;
                }
                _ => {}
            }
            self.append(&line).await;
            self.last = Some((line, 0));
        }
    }

    /// One line appended, with the time before it: rule 3.
    async fn append(&mut self, line: &str) {
        let text = format!("{} {line}\n", stamp(false));
        if self.bytes + text.len() as u64 > FILE_BYTES {
            let Some(file) = open(&self.folder).await else {
                return;
            };
            self.file = file;
            self.bytes = 0;
        }
        if self.file.write_all(text.as_bytes()).await.is_ok() {
            self.bytes += text.len() as u64;
        }
    }
}

/// A new file in `folder`, named for now, and every file but the
/// newest [`FILES`] deleted: rules 2 and 3. The names sort as their
/// times do, so the newest are the last by name.
async fn open(folder: &Path) -> Option<File> {
    let path = folder.join(format!("provider-{}.log", stamp(true)));
    let file = OpenOptions::new().create(true).append(true).open(&path).await.ok()?;
    let mut kept = Vec::new();
    if let Ok(mut listed) = tokio::fs::read_dir(folder).await {
        while let Ok(Some(entry)) = listed.next_entry().await {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with("provider-") && name.ends_with(".log") {
                kept.push(entry.path());
            }
        }
    }
    kept.sort();
    let surplus = kept.len().saturating_sub(FILES);
    for old in kept.into_iter().take(surplus) {
        let _ = tokio::fs::remove_file(old).await;
    }
    Some(file)
}
