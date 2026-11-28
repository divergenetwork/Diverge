//! The tools one run is served.

use std::collections::HashMap;

use tokio::sync::watch;

use super::Registry;
use crate::containers::ToolKey;
use crate::store::DependencyToolId;
use crate::store::connected_standalone_tools::ConnectedTool;
use crate::store::tools::Tool;

/// One served tool: an attached record, standalone or connected, whose
/// run — up or not, and a new one after every end — is the live set's
/// to find; or a dependency tool, by its row, whose run is the live
/// set's to find as well: a lazy one of the agent's own, a lazy one
/// shared with other agents, or an eager one. Whichever it is, its
/// run is started at the first use that finds none.
pub enum Entry {
    /// A standalone tool the agent has, directly or through its
    /// template: its record. Its run is looked up at every use and
    /// started at the first that finds none.
    Attached {
        /// The record.
        tool: Tool,
    },
    /// A connected tool the agent has, directly or through its
    /// template: its record. Its connect is looked up at every use and
    /// opened at the first that finds none.
    Connected {
        /// The record.
        tool: ConnectedTool,
    },
    /// A lazy dependency tool that is the agent's alone, its scope
    /// `instance`: deployed at the agent's registration, and its run
    /// looked up at every use and started at the first that finds
    /// none.
    Own {
        /// The row.
        id: DependencyToolId,
        /// The template it was made from, by id.
        template: String,
    },
    /// A lazy dependency tool the agent shares with others, its scope
    /// `template`: its run looked up at every use and
    /// started at the first that finds none.
    Shared {
        /// The row.
        id: DependencyToolId,
        /// The template it was made from, by id.
        template: String,
    },
    /// An eager tool the agent is attached to: kept by the daemon while
    /// any agent is, its run looked up when it is called.
    Eager {
        /// The row.
        id: DependencyToolId,
        /// The template it was made from, by id.
        template: String,
    },
}

/// The tools one run is served, each under its prefix, the revision
/// every listener announces the merged lists by when the set changes,
/// and the word to the daemon's own relay of the tools' notifications
/// whenever the set or a served tool's run changed.
pub struct Served {
    registry: Registry<ToolKey>,
    entries: HashMap<ToolKey, Entry>,
    /// Bumped at every change of the set: what every notification
    /// stream announces both merged lists by. A watch, so no listener can
    /// fall behind it, and any number of changes is one wake.
    revision: watch::Sender<u64>,
    /// Bumped at every change of the set and every start of a served
    /// tool's run: what the notifications relay keeps in step with.
    generation: watch::Sender<u64>,
}

impl Default for Served {
    fn default() -> Self {
        Served::new()
    }
}

impl Served {
    /// Nothing served.
    pub fn new() -> Self {
        Served {
            registry: Registry::new(),
            entries: HashMap::new(),
            revision: watch::channel(0).0,
            generation: watch::channel(0).0,
        }
    }

    /// Serve the attached tool under `serve_name`, idle; the prefix it
    /// gets, kept until it leaves. One served already keeps its
    /// prefix and its entry.
    pub fn insert(&mut self, tool: Tool, serve_name: &str) -> String {
        let key = ToolKey::Record(tool.id);
        let prefix = self.registry.assign(key, serve_name);
        self.entries.entry(key).or_insert(Entry::Attached { tool });
        self.changed();
        prefix
    }

    /// Serve the attached connected tool under `serve_name`, idle; the
    /// prefix it gets, kept until it leaves. One served already keeps
    /// its prefix and its entry.
    pub fn insert_connected(&mut self, tool: ConnectedTool, serve_name: &str) -> String {
        let key = ToolKey::Connected(tool.id);
        let prefix = self.registry.assign(key, serve_name);
        self.entries.entry(key).or_insert(Entry::Connected { tool });
        self.changed();
        prefix
    }

    /// Serve the agent's own lazy dependency tool under `serve_name`,
    /// its template's id short; the prefix it gets.
    pub fn insert_own(&mut self, id: DependencyToolId, template: &str, serve_name: &str) -> String {
        let key = ToolKey::Dependency(id);
        let prefix = self.registry.assign(key, serve_name);
        self.entries.insert(
            key,
            Entry::Own {
                id,
                template: template.to_string(),
            },
        );
        self.changed();
        prefix
    }

    /// Serve the shared lazy dependency tool under `serve_name`, its
    /// template's id short, idle; the prefix it gets.
    pub fn insert_shared(&mut self, id: DependencyToolId, template: &str, serve_name: &str) -> String {
        let key = ToolKey::Dependency(id);
        let prefix = self.registry.assign(key, serve_name);
        self.entries.insert(
            key,
            Entry::Shared {
                id,
                template: template.to_string(),
            },
        );
        self.changed();
        prefix
    }

    /// Serve the eager tool under `serve_name`, its template's id short;
    /// the prefix it gets.
    pub fn insert_eager(&mut self, id: DependencyToolId, template: &str, serve_name: &str) -> String {
        let key = ToolKey::Dependency(id);
        let prefix = self.registry.assign(key, serve_name);
        self.entries.insert(
            key,
            Entry::Eager {
                id,
                template: template.to_string(),
            },
        );
        self.changed();
        prefix
    }

    /// A served tool's run started: the relay of the tools'
    /// notifications is told to look again.
    pub fn started(&self) {
        self.bump();
    }

    /// The tool leaves: its prefix is free.
    pub fn remove(&mut self, key: ToolKey) -> Option<Entry> {
        let entry = self.entries.remove(&key);
        if entry.is_some() {
            self.registry.release(&key);
            self.changed();
        }
        entry
    }

    /// The entry for the tool, if served.
    pub fn entry(&self, key: ToolKey) -> Option<&Entry> {
        self.entries.get(&key)
    }

    /// Every served tool, by key, in no order.
    pub fn keys(&self) -> Vec<ToolKey> {
        self.entries.keys().copied().collect()
    }

    /// Every lazy dependency tool that is the agent's alone, by id,
    /// in no order: those among them that mount the agent's files go
    /// down with the agent's container.
    pub fn own(&self) -> Vec<DependencyToolId> {
        self.entries
            .values()
            .filter_map(|entry| match entry {
                Entry::Own { id, .. } => Some(*id),
                Entry::Attached { .. } | Entry::Connected { .. } | Entry::Shared { .. } | Entry::Eager { .. } => None,
            })
            .collect()
    }

    /// Every eager tool, by id, in no order.
    pub fn eager(&self) -> Vec<DependencyToolId> {
        self.entries
            .values()
            .filter_map(|entry| match entry {
                Entry::Eager { id, .. } => Some(*id),
                Entry::Attached { .. } | Entry::Connected { .. } | Entry::Own { .. } | Entry::Shared { .. } => None,
            })
            .collect()
    }

    /// The tool's prefix.
    pub fn prefix(&self, key: ToolKey) -> Option<String> {
        self.registry.prefix(&key).map(str::to_string)
    }

    /// The tool a prefix belongs to.
    pub fn owner(&self, prefix: &str) -> Option<ToolKey> {
        self.registry.owner(prefix).copied()
    }

    /// Every change of the set from now on: what a notification stream
    /// announces both merged lists by.
    pub fn revision(&self) -> watch::Receiver<u64> {
        self.revision.subscribe()
    }

    /// Every change of the set, and every start of a served tool's
    /// run, from now on: what the relay of the tools' notifications
    /// keeps in step with.
    pub fn generation(&self) -> watch::Receiver<u64> {
        self.generation.subscribe()
    }

    /// The set changed: the revision bumps, and every listener announces
    /// that the merged tool list and resource list changed; the relay is
    /// told too.
    fn changed(&self) {
        self.revision.send_modify(|revision| *revision = revision.wrapping_add(1));
        self.bump();
    }

    /// The relay is told to look again.
    fn bump(&self) {
        self.generation.send_modify(|generation| *generation = generation.wrapping_add(1));
    }
}
