//! The record keeper: a run's record file, held for one run at a time and
//! replaced whole with each record the run hands over.
//!
//! Beside a record file `run.toml` the keeper keeps `run.toml.lock`, which it
//! holds locked for as long as it holds the record, and writes each record to
//! `run.toml.new` before renaming it over the record.

use std::ffi::OsString;
use std::fmt;
use std::fs::{File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::text::{FileFault, read_text};

/// A record file held for one run.
#[derive(Debug)]
pub struct RecordKeeper {
    record: PathBuf,
    file: String,
    _lock: File,
    handed: usize,
    kept: usize,
    failure: Option<String>,
}

/// Why a record file was not held for a run.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeeperRefusal {
    /// A new run was given a record file that already exists.
    Exists {
        /// The record file, as it was named.
        file: String,
    },
    /// Another run holds the record file, through this lock file.
    Held {
        /// The lock file.
        lock: String,
    },
    /// The lock file could not be opened or locked.
    Lock {
        /// The lock file.
        lock: String,
        /// The operating system's account of why.
        message: String,
    },
    /// The record a run is resumed from cannot be read.
    Record(FileFault),
}

impl fmt::Display for KeeperRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exists { file } => write!(
                f,
                "{file} already exists, and a new run does not replace another run's record"
            ),
            Self::Held { lock } => write!(
                f,
                "another run holds the record through {lock}; wait for it to stop"
            ),
            Self::Lock { lock, message } => write!(f, "{lock} cannot be locked: {message}"),
            Self::Record(fault) => fault.fmt(f),
        }
    }
}

impl std::error::Error for KeeperRefusal {}

/// A record file that does not hold the latest record handed over: which of
/// the records it holds, counted from one in the order they were handed over,
/// or none of them, and why the latest was not written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unkept {
    /// The record file, as it was named.
    pub file: String,
    /// Which record it holds: 0 for none of those handed over.
    pub holds: usize,
    /// How many records were handed over.
    pub handed: usize,
    /// The operating system's account of the last write that failed.
    pub message: String,
}

impl fmt::Display for Unkept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            file,
            holds,
            handed,
            message,
        } = self;
        match holds {
            0 => write!(
                f,
                "{file} holds none of the {handed} records this run handed over: {message}"
            ),
            _ => write!(
                f,
                "{file} holds record {holds} of the {handed} this run handed over: {message}"
            ),
        }
    }
}

impl RecordKeeper {
    /// The record file `record` held for a new run, or refused: held by another
    /// run, or already there.
    // @A new run's record file held and one that exists refused,IMPL_KEEPER_NEW_RUN,impl,[CREQ_KEEPER_REFUSES_EXISTING, CREQ_KEEPER_ONE_HOLDER],[DEC_RECORD_FILE_LOCKED]
    pub fn new_run(record: &Path) -> Result<Self, KeeperRefusal> {
        let keeper = Self::held(record)?;
        match record.try_exists() {
            Ok(false) => Ok(keeper),
            Ok(true) => Err(KeeperRefusal::Exists { file: keeper.file }),
            Err(error) => Err(KeeperRefusal::Record(FileFault::Unreadable {
                file: keeper.file,
                message: error.to_string(),
            })),
        }
    }

    /// The record file `record` held for a run resumed from it, with the
    /// record it holds, or refused: held by another run, or unreadable.
    // @The record read only once its file is held,IMPL_KEEPER_RESUME,impl,[CREQ_KEEPER_ONE_HOLDER],[DEC_RECORD_FILE_LOCKED]
    pub fn resume(record: &Path) -> Result<(Self, String), KeeperRefusal> {
        let keeper = Self::held(record)?;
        let text = read_text(record, &keeper.file).map_err(KeeperRefusal::Record)?;
        Ok((keeper, text))
    }

    /// Makes the record file hold exactly `record`. A write that fails is
    /// remembered and reported when the keeper is let go, unless a later one
    /// succeeds.
    // @Each record written beside the file and renamed over it,IMPL_KEEPER_KEEP,impl,[CREQ_KEEPER_REPLACES_WHOLE, CREQ_KEEPER_TELLS_WHICH_IS_KEPT],[DEC_RECORD_REPLACED_BY_RENAME, DEC_RECORD_NOT_KEPT_TOLD]
    pub fn keep(&mut self, record: &str) {
        self.handed += 1;
        let beside = beside(&self.record, ".new");
        let written = File::create(&beside)
            .and_then(|mut file| file.write_all(record.as_bytes()))
            .and_then(|()| std::fs::rename(&beside, &self.record));
        match written {
            Ok(()) => {
                self.kept = self.handed;
                self.failure = None;
            }
            Err(error) => {
                let _ = std::fs::remove_file(&beside);
                self.failure = Some(error.to_string());
            }
        }
    }

    /// Lets the record file go, and says which record it holds when that is
    /// not the latest handed over.
    // @The file let go with which record it holds,IMPL_KEEPER_RELEASE,impl,[CREQ_KEEPER_TELLS_WHICH_IS_KEPT],[DEC_RECORD_NOT_KEPT_TOLD]
    pub fn release(self) -> Option<Unkept> {
        (self.kept < self.handed).then(|| Unkept {
            file: self.file.clone(),
            holds: self.kept,
            handed: self.handed,
            message: self.failure.clone().unwrap_or_default(),
        })
    }

    /// A keeper holding the lock beside `record`, or refused.
    fn held(record: &Path) -> Result<Self, KeeperRefusal> {
        let lock_path = beside(record, ".lock");
        let lock = lock_path.display().to_string();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| KeeperRefusal::Lock {
                lock: lock.clone(),
                message: error.to_string(),
            })?;
        match file.try_lock() {
            Ok(()) => Ok(Self {
                record: record.to_owned(),
                file: record.display().to_string(),
                _lock: file,
                handed: 0,
                kept: 0,
                failure: None,
            }),
            Err(TryLockError::WouldBlock) => Err(KeeperRefusal::Held { lock }),
            Err(TryLockError::Error(error)) => Err(KeeperRefusal::Lock {
                lock,
                message: error.to_string(),
            }),
        }
    }
}

/// `record` with `suffix` after its file name.
fn beside(record: &Path, suffix: &str) -> PathBuf {
    let mut path = OsString::from(record.as_os_str());
    path.push(suffix);
    PathBuf::from(path)
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::{Scratch, without_keys};
#[cfg(test)]
use proptest::prelude::*;

/// Texts with LF and CRLF line endings, as a run's records might be written.
#[cfg(test)]
fn any_records() -> impl Strategy<Value = Vec<String>> {
    let piece = prop_oneof![
        Just("\n".to_owned()),
        Just("\r\n".to_owned()),
        "[ -~]{0,12}",
    ];
    prop::collection::vec(
        prop::collection::vec(piece, 0..12).prop_map(|pieces| pieces.concat()),
        1..6,
    )
}

#[cfg(test)]
proptest! {
    #[test]
    fn record_replaced_whole(records in any_records()) {
        let scratch = Scratch::new("record_replaced_whole");
        let record = scratch.path("run.toml");
        let mut keeper = RecordKeeper::new_run(&record).expect("the file is held");

        for text in &records {
            keeper.keep(text);
            prop_assert_eq!(std::fs::read(&record).expect("the record"), text.as_bytes());
        }
        prop_assert_eq!(keeper.release(), None);

        let mut left: Vec<String> = std::fs::read_dir(scratch.root())
            .expect("the directory")
            .map(|entry| entry.expect("an entry").file_name().to_string_lossy().into_owned())
            .collect();
        left.sort();
        prop_assert_eq!(left, ["run.toml", "run.toml.lock"]);
    }
}

#[cfg(test)]
#[test]
fn existing_file_refused() {
    let scratch = Scratch::new("existing_file_refused");
    for (name, text) in [("held.toml", "a record"), ("empty.toml", "")] {
        let record = scratch.write(name, text);
        let before = std::fs::metadata(&record)
            .expect("the file")
            .modified()
            .expect("a time");

        assert_eq!(
            RecordKeeper::new_run(&record).map(|_| ()),
            Err(KeeperRefusal::Exists {
                file: record.display().to_string()
            })
        );

        assert_eq!(std::fs::read_to_string(&record).expect("the file"), text);
        let after = std::fs::metadata(&record)
            .expect("the file")
            .modified()
            .expect("a time");
        assert_eq!(before, after);
    }
}

#[cfg(test)]
#[test]
fn held_file_refused() {
    let scratch = Scratch::new("held_file_refused");
    let record = scratch.path("run.toml");
    let lock = format!("{}.lock", record.display());

    let mut first = RecordKeeper::new_run(&record).expect("the file is held");
    // No record exists yet, so a second keeper that read before locking would
    // be refused for the missing record rather than for the lock.
    assert_eq!(
        RecordKeeper::resume(&record).map(|_| ()),
        Err(KeeperRefusal::Held { lock: lock.clone() })
    );
    assert_eq!(
        RecordKeeper::new_run(&record).map(|_| ()),
        Err(KeeperRefusal::Held { lock: lock.clone() })
    );
    assert!(std::path::Path::new(&lock).exists());

    first.keep("one");
    assert_eq!(std::fs::read_to_string(&record).expect("the record"), "one");
    assert_eq!(first.release(), None);

    let (third, text) = RecordKeeper::resume(&record).expect("the file is free");
    assert_eq!(text, "one");
    drop(third);

    // Sixteen at once for one free file, each holding what it took until all
    // have tried.
    let fresh = scratch.path("raced.toml");
    let tried = std::sync::Arc::new(std::sync::Barrier::new(16));
    let started = std::sync::Arc::new(std::sync::Barrier::new(16));
    let racers: Vec<_> = (0..16)
        .map(|_| {
            let (fresh, tried, started) = (fresh.clone(), tried.clone(), started.clone());
            std::thread::spawn(move || {
                started.wait();
                let taken = RecordKeeper::new_run(&fresh);
                tried.wait();
                taken.is_ok()
            })
        })
        .collect();
    let won = racers
        .into_iter()
        .map(|racer| racer.join().expect("a racer"))
        .filter(|&won| won)
        .count();
    assert_eq!(won, 1);
}

#[cfg(test)]
#[test]
fn killed_holder_releases() {
    if let Some(record) = std::env::var_os("AGCONFLO_RUNNER_TEST_HOLD") {
        let _keeper =
            RecordKeeper::new_run(std::path::Path::new(&record)).expect("the file is held");
        println!("held");
        std::thread::sleep(std::time::Duration::from_secs(60));
        return;
    }

    let scratch = Scratch::new("killed_holder_releases");
    let record = scratch.path("run.toml");
    let mut child = without_keys(&mut std::process::Command::new(
        std::env::current_exe().expect("the test binary"),
    ))
    .args(["--exact", "keeper::killed_holder_releases", "--nocapture"])
    .env("AGCONFLO_RUNNER_TEST_HOLD", &record)
    .stdout(std::process::Stdio::piped())
    .spawn()
    .expect("the child started");
    let mut lines = std::io::BufRead::lines(std::io::BufReader::new(
        child.stdout.take().expect("the child's output"),
    ));
    assert!(
        lines.any(|line| line.expect("a line") == "held"),
        "the child never held the file"
    );

    let refused = RecordKeeper::new_run(&record).map(|_| ());
    child.kill().expect("the child killed");
    child.wait().expect("the child ended");

    assert!(
        matches!(refused, Err(KeeperRefusal::Held { .. })),
        "{refused:?}"
    );
    RecordKeeper::new_run(&record).expect("the killed child's lock is gone");
}

#[cfg(test)]
#[test]
fn unkept_record_told() {
    let scratch = Scratch::new("unkept_record_told");
    let record = scratch.path("run.toml");
    let blocked = format!("{}.new", record.display());

    let mut keeper = RecordKeeper::new_run(&record).expect("the file is held");
    keeper.keep("one");
    keeper.keep("two");
    // A directory where the new record is written makes every write fail, on
    // every system, and leaves the record file alone.
    std::fs::create_dir_all(std::path::Path::new(&blocked).join("inside")).expect("the block");
    keeper.keep("three");
    keeper.keep("four");
    match keeper.release() {
        Some(Unkept {
            file,
            holds,
            handed,
            message,
        }) => {
            assert_eq!(file, record.display().to_string());
            assert_eq!((holds, handed), (2, 4));
            assert!(!message.is_empty());
        }
        None => panic!("expected the record reported not kept"),
    }
    assert_eq!(std::fs::read_to_string(&record).expect("the record"), "two");

    // A write that fails and a later one that succeeds leave the file current.
    let (mut keeper, text) = RecordKeeper::resume(&record).expect("the file is held");
    assert_eq!(text, "two");
    keeper.keep("five");
    std::fs::remove_dir_all(&blocked).expect("the block removed");
    keeper.keep("six");
    assert_eq!(keeper.release(), None);
    assert_eq!(std::fs::read_to_string(&record).expect("the record"), "six");
}
