//! What the tests of every module share: a directory of files of their own.

use std::path::PathBuf;

/// A directory for one test's files, emptied when it is made and removed when
/// it is dropped.
pub(crate) struct Scratch {
    root: PathBuf,
}

impl Scratch {
    /// A directory named after `test` and this process, in the system's
    /// temporary directory.
    pub(crate) fn new(test: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("agconflo-runner-{test}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("a scratch directory");
        Self { root }
    }

    /// The path of `file` in the directory.
    pub(crate) fn path(&self, file: &str) -> PathBuf {
        self.root.join(file)
    }

    /// `text` written to `file` in the directory, its directories made first.
    pub(crate) fn write(&self, file: &str, text: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("the file's directory");
        }
        std::fs::write(&path, text).expect("the file written");
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}
