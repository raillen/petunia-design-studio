//! Debounced background recovery for every open session, including inactive
//! tabs. One worker and two queued jobs; no worker can mutate a document.
use crate::{session::SessionIdentity, DocumentSession};
use petunia_design_io::recovery::{RecoveryKey, RecoveryMetadata, RecoveryStore};
use petunia_design_jobs::{JobExecutor, JobFailure, JobHandle, JobManager};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::{Duration, Instant},
};
struct Pending {
    revision: u64,
    cleaning: bool,
    handle: JobHandle<RecoveryKey>,
}
struct Slot {
    revision: u64,
    dirty: bool,
    changed: Instant,
    stored: Option<u64>,
    key: Option<RecoveryKey>,
    pending: Option<Pending>,
    failed: Option<Instant>,
}
pub struct RecoveryController {
    directory: PathBuf,
    executor: JobExecutor,
    slots: HashMap<SessionIdentity, Slot>,
}
impl RecoveryController {
    pub fn new(directory: PathBuf) -> Result<Self, JobFailure> {
        Ok(Self {
            directory,
            executor: JobExecutor::new(1, 2, JobManager::new())?,
            slots: HashMap::new(),
        })
    }
    /// Scheduling/polling only; directories, binary encoding and fsync run in
    /// the worker. Completed old revisions remain valid backups, never edits.
    pub fn tick(&mut self, sessions: &[&DocumentSession]) -> Result<(), JobFailure> {
        let now = Instant::now();
        let alive: HashSet<_> = sessions.iter().map(|session| session.identity()).collect();
        if sessions.len() > 32 {
            return Err(JobFailure::Failed(
                "recovery session count budget exceeded".into(),
            ));
        }
        for slot in self.slots.values_mut() {
            if let Some(pending) = &slot.pending {
                match pending.handle.try_result(pending.revision) {
                    Ok(None) => {}
                    Ok(Some(key)) => {
                        if pending.cleaning {
                            slot.key = None;
                            slot.stored = None;
                        } else {
                            slot.key = Some(key);
                            slot.stored = Some(pending.revision);
                        }
                        slot.pending = None;
                    }
                    Err(error) => {
                        slot.pending = None;
                        slot.failed = Some(now);
                        return Err(error);
                    }
                }
            }
        }
        for session in sessions {
            let id = session.identity();
            let dirty = session.is_dirty();
            if !dirty && !self.slots.contains_key(&id) {
                continue;
            }
            let slot = self.slots.entry(id).or_insert_with(|| Slot {
                revision: session.current_revision(),
                dirty,
                changed: now,
                stored: None,
                key: None,
                pending: None,
                failed: None,
            });
            if slot.revision != session.current_revision() || slot.dirty != dirty {
                slot.revision = session.current_revision();
                slot.dirty = dirty;
                slot.changed = now;
                slot.failed = None;
            }
            if !dirty
                || slot.stored == Some(slot.revision)
                || slot.pending.is_some()
                || now.duration_since(slot.changed) < Duration::from_secs(2)
                || slot
                    .failed
                    .is_some_and(|failure| now.duration_since(failure) < Duration::from_secs(30))
            {
                continue;
            }
            let document = session.document().clone();
            let metadata = RecoveryMetadata::new(
                session.title().into(),
                session.path().map(std::path::Path::to_path_buf),
                slot.revision,
            );
            let directory = self.directory.clone();
            let previous = slot.key.clone();
            let revision = slot.revision;
            match self
                .executor
                .submit("recovery-write", revision, move |context| {
                    context.check_cancelled()?;
                    RecoveryStore::new(directory)
                        .and_then(|store| store.write(&document, &metadata, previous.as_ref()))
                        .map_err(|error| JobFailure::Failed(error.to_string()))
                }) {
                Ok(handle) => {
                    slot.pending = Some(Pending {
                        revision,
                        cleaning: false,
                        handle,
                    })
                }
                Err(JobFailure::QueueFull) => {}
                Err(error) => return Err(error),
            }
        }
        let mut retired = Vec::new();
        for (id, slot) in &mut self.slots {
            if alive.contains(id) && slot.dirty
                || slot.pending.is_some()
                || slot
                    .failed
                    .is_some_and(|failure| now.duration_since(failure) < Duration::from_secs(30))
            {
                continue;
            }
            if let Some(key) = slot.key.clone() {
                let directory = self.directory.clone();
                match self.executor.submit("recovery-cleanup", 0, move |_| {
                    RecoveryStore::new(directory)
                        .and_then(|store| store.remove(&key))
                        .map_err(|error| JobFailure::Failed(error.to_string()))?;
                    Ok(key)
                }) {
                    Ok(handle) => {
                        slot.pending = Some(Pending {
                            revision: 0,
                            cleaning: true,
                            handle,
                        })
                    }
                    Err(JobFailure::QueueFull) => {}
                    Err(error) => return Err(error),
                }
            } else {
                retired.push(*id);
            }
        }
        for id in retired {
            self.slots.remove(&id);
        }
        Ok(())
    }
}
