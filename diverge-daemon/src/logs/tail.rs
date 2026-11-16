//! A log read from its end, and a log's files made whole.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};

use super::{ENTRY, Error};

/// A log read backwards: its latest item first, then the one before
/// it, to the first. Both files are opened once and kept, so a long
/// run is walked without opening them per item. Each item is the line
/// as JSON and nothing more: what reads a log this way asks an item
/// its `type`, and no book's entry type.
#[derive(Debug)]
pub struct Tail {
    log: File,
    index: File,
    log_path: PathBuf,
    index_path: PathBuf,
    /// How many items are still to be read: the `logs_index` of the
    /// next one handed out.
    left: u64,
}

impl Tail {
    /// The log in `dir`, at its end; `None` for a directory with no
    /// log in it, or an empty one.
    pub async fn open(dir: &Path) -> Result<Option<Tail>, Error> {
        let log_path = dir.join("log");
        let index_path = dir.join("index");
        let Some(index_len) = length(&index_path).await? else {
            return Ok(None);
        };
        let left = index_len / ENTRY;
        if left == 0 {
            return Ok(None);
        }
        let index = File::open(&index_path).await.map_err(|source| Error::io(&index_path, source))?;
        let log = File::open(&log_path).await.map_err(|source| Error::io(&log_path, source))?;
        Ok(Some(Tail { log, index, log_path, index_path, left }))
    }

    /// The item before the last one handed out, or `None` once the
    /// first has been.
    pub async fn previous(&mut self) -> Result<Option<Value>, Error> {
        if self.left == 0 {
            return Ok(None);
        }
        let (offset, length) = entry(&mut self.index, &self.index_path, self.left).await?;
        self.left -= 1;
        self.log.seek(SeekFrom::Start(offset)).await.map_err(|source| Error::io(&self.log_path, source))?;
        let mut line = vec![0u8; usize::try_from(length).unwrap_or(0)];
        self.log.read_exact(&mut line).await.map_err(|source| Error::io(&self.log_path, source))?;
        serde_json::from_slice(&line).map(Some).map_err(Error::Json)
    }
}

/// The log's two files in `dir` made whole, after a daemon that ended
/// without stopping. An append writes the line to `log` and then its
/// entry to `index`, so a daemon that died under one leaves the end of
/// either torn, and nothing before the end. The rules:
///
/// 1. `index` is cut down to a whole number of entries: a part of one
///    is an entry that was not written.
/// 2. From the last entry backwards, an entry is dropped while it is
///    not good — it reaches past the end of `log`, or its line does
///    not end in a newline, or its line is not JSON. The first good
///    entry ends the walk: what a death tears is the end.
/// 3. `log` is cut down to the end of the last good entry's line,
///    which removes a line written with no entry for it.
///
/// A directory with neither file is left as it is, and so is a log
/// already whole: no file is opened for writing that needs no cut.
pub async fn repair(dir: &Path) -> Result<(), Error> {
    let log_path = dir.join("log");
    let index_path = dir.join("index");
    let index_len = length(&index_path).await?;
    let log_len = length(&log_path).await?;
    if index_len.is_none() && log_len.is_none() {
        return Ok(());
    }
    let (index_len, log_len) = (index_len.unwrap_or(0), log_len.unwrap_or(0));
    let mut entries = index_len / ENTRY;
    let mut end = 0;
    if entries > 0 && log_len > 0 {
        let mut index = File::open(&index_path).await.map_err(|source| Error::io(&index_path, source))?;
        let mut log = File::open(&log_path).await.map_err(|source| Error::io(&log_path, source))?;
        while entries > 0 {
            let (offset, length) = entry(&mut index, &index_path, entries).await?;
            if good(&mut log, &log_path, log_len, offset, length).await? {
                end = offset + length;
                break;
            }
            entries -= 1;
        }
    } else {
        entries = 0;
    }
    if entries * ENTRY != index_len {
        cut(&index_path, entries * ENTRY).await?;
    }
    if end != log_len {
        cut(&log_path, end).await?;
    }
    Ok(())
}

/// How long the file at `path` is; `None` where there is none.
async fn length(path: &Path) -> Result<Option<u64>, Error> {
    match tokio::fs::metadata(path).await {
        Ok(meta) => Ok(Some(meta.len())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(Error::io(path, source)),
    }
}

/// The entry of the item numbered `number`, from `1`: its line's
/// offset in `log`, and its length.
async fn entry(index: &mut File, path: &Path, number: u64) -> Result<(u64, u64), Error> {
    index.seek(SeekFrom::Start((number - 1) * ENTRY)).await.map_err(|source| Error::io(path, source))?;
    let mut raw = [0u8; 16];
    index.read_exact(&mut raw).await.map_err(|source| Error::io(path, source))?;
    let offset = u64::from_be_bytes(raw[..8].try_into().unwrap_or([0; 8]));
    let length = u64::from_be_bytes(raw[8..].try_into().unwrap_or([0; 8]));
    Ok((offset, length))
}

/// Whether the entry names a whole line of a `log` that is `log_len`
/// long: within the file, ended by a newline, and JSON.
async fn good(log: &mut File, path: &Path, log_len: u64, offset: u64, length: u64) -> Result<bool, Error> {
    let within = length > 0 && offset.checked_add(length).is_some_and(|end| end <= log_len);
    let Some(size) = usize::try_from(length).ok().filter(|_| within) else {
        return Ok(false);
    };
    log.seek(SeekFrom::Start(offset)).await.map_err(|source| Error::io(path, source))?;
    let mut line = vec![0u8; size];
    log.read_exact(&mut line).await.map_err(|source| Error::io(path, source))?;
    Ok(line.last() == Some(&b'\n') && serde_json::from_slice::<Value>(&line).is_ok())
}

/// The file at `path` cut down to `length` bytes.
async fn cut(path: &Path, length: u64) -> Result<(), Error> {
    let file = OpenOptions::new().write(true).open(path).await.map_err(|source| Error::io(path, source))?;
    file.set_len(length).await.map_err(|source| Error::io(path, source))
}
