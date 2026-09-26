//! The grants reader: a grants file read into what a run's tools may do.
//!
//! A grants file is a TOML document:
//!
//! ```toml
//! image = "alpine@sha256:..."          # by digest, or an image id
//! images = ["python@sha256:..."]       # others a tool may name, if any
//! network = true                       # no network unless true
//! actions = ["read", "write", "run"]   # none unless named
//! trust = "certs/ca.pem"               # certificates a step trusts, if any
//!
//! [folders.project]                    # mounted at /work/project
//! path = "../project"                  # from this file's directory
//! writable = true                      # read-only unless true
//!
//! [limits]                             # each optional
//! seconds = 60                         # the time a command may run
//! output = 16384                       # the bytes a step gives back
//! tmp = 268435456                      # the bytes /tmp holds
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

use crate::text::{FileFault, KeyFault, Place, Toml, path, read_text};

/// What a tool does: reads a file, writes one, or runs a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Action {
    /// Reads a file.
    Read,
    /// Writes a file.
    Write,
    /// Runs a command.
    Run,
}

impl Action {
    /// The action a grants file or a manifest names `name`, or `None`.
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "run" => Some(Self::Run),
            _ => None,
        }
    }

    /// The action's name, as a grants file or a manifest writes it.
    pub fn name(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Run => "run",
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One granted folder: the name it is mounted under, its path, and whether a
/// tool may write to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Folder {
    /// The name it is mounted under.
    pub name: String,
    /// Its path: the grants file's directory joined with the path the file
    /// writes.
    pub path: PathBuf,
    /// Whether a tool may write to it.
    pub writable: bool,
}

/// The limits every step's command runs under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandLimits {
    /// The seconds a command may run.
    pub seconds: u32,
    /// The bytes of output a step gives back.
    pub output: usize,
    /// The bytes a container's `/tmp` holds.
    pub tmp: u64,
}

impl Default for CommandLimits {
    /// 60 seconds, 16384 bytes of output and 268435456 of `/tmp`.
    // @Limits of 60 seconds and 16384 bytes and a /tmp of 256 MiB unless given,IMPL_GRANTS_DEFAULT_LIMITS,impl,[CREQ_GRANTS_READS, CREQ_GRANTS_READS_TMP_LIMIT],[DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN, DEC_TMP_LIMITED_BY_GRANTS]
    fn default() -> Self {
        Self {
            seconds: 60,
            output: 16384,
            tmp: 268_435_456,
        }
    }
}

/// A grants file read: the image, the folders, whether the network is
/// granted, the actions allowed and the limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grants {
    image: String,
    images: Vec<String>,
    folders: Vec<Folder>,
    network: bool,
    actions: Vec<Action>,
    trust: Option<PathBuf>,
    limits: CommandLimits,
}

impl Grants {
    /// The image, as the file names it: the one a tool naming none runs in.
    pub fn image(&self) -> &str {
        &self.image
    }

    /// The other images a tool may name, in the order the file names them.
    pub fn images(&self) -> &[String] {
        &self.images
    }

    /// Whether a tool may run in `image`: the image, or one of the images.
    pub fn allows_image(&self, image: &str) -> bool {
        self.image == image || self.images.iter().any(|allowed| allowed == image)
    }

    /// The folders, in the order the file names them.
    pub fn folders(&self) -> &[Folder] {
        &self.folders
    }

    /// Whether the network is granted.
    pub fn network(&self) -> bool {
        self.network
    }

    /// The actions allowed, each once, in the order the file first names them.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// Whether `action` is allowed.
    pub fn allows(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    /// The trust file's path, any link in it resolved, if the file names
    /// one: the certificate authorities a step trusts.
    pub fn trust(&self) -> Option<&Path> {
        self.trust.as_deref()
    }

    /// The limits every step's command runs under.
    pub fn limits(&self) -> CommandLimits {
        self.limits
    }
}

/// Why a granted folder is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FolderFault {
    /// Its name is not a single plain part of a path.
    Name,
    /// Its path is not there, or cannot be looked at.
    Absent {
        /// The path, as the file writes it.
        path: String,
        /// The operating system's account of why.
        message: String,
    },
    /// Its path is not a directory.
    NotADirectory {
        /// The path, as the file writes it.
        path: String,
    },
}

/// Why a grants file cannot be read. Each fault names the file as it was
/// given.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum GrantsFault {
    /// The file cannot be read as text.
    File(FileFault),
    /// The file is not a grants file, and where.
    Grants {
        /// Where in the file.
        place: Place,
        /// What is wrong there.
        fault: KeyFault,
    },
    /// The file allows an action there is none of.
    Action {
        /// Where the action is named.
        place: Place,
        /// The action, as the file names it.
        action: String,
    },
    /// The file names its image by neither a digest nor an image id.
    Image {
        /// Where the image is named.
        place: Place,
        /// The image, as the file names it.
        image: String,
    },
    /// A granted folder is refused.
    Folder {
        /// Where the folder, or its path, is named.
        place: Place,
        /// The folder's name.
        name: String,
        /// Why it is refused.
        fault: FolderFault,
    },
    /// The trust file is not a file that can be read.
    Trust {
        /// Where the trust file is named.
        place: Place,
        /// The path, as the file writes it.
        path: String,
        /// Why it cannot be read.
        message: String,
    },
}

impl fmt::Display for GrantsFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(fault) => fault.fmt(f),
            Self::Grants { place, fault } => write!(f, "{place}: {fault}"),
            Self::Action { place, action } => write!(
                f,
                "{place}: {action} is not an action; the actions are read, write and run"
            ),
            Self::Image { place, image } => write!(
                f,
                "{place}: the image {image} is named by neither a digest, as name@sha256:<64 hexadecimal digits>, nor an image id, as sha256:<64 hexadecimal digits>"
            ),
            Self::Folder { place, name, fault } => match fault {
                FolderFault::Name => write!(
                    f,
                    "{place}: the folder name {name:?} is not one plain part of a path"
                ),
                FolderFault::Absent { path, message } => write!(
                    f,
                    "{place}: the folder {name} is granted {path}, which cannot be found: {message}"
                ),
                FolderFault::NotADirectory { path } => write!(
                    f,
                    "{place}: the folder {name} is granted {path}, which is not a directory"
                ),
            },
            Self::Trust {
                place,
                path,
                message,
            } => write!(
                f,
                "{place}: the trust file {path} cannot be read: {message}"
            ),
        }
    }
}

impl std::error::Error for GrantsFault {}

/// The grants file at `grants`, or the first fault met reading it.
///
/// Each folder's path is read from the file's own directory, and must be an
/// existing directory. A folder is read-only, the network not granted and no
/// action allowed unless the file says otherwise, and each limit is its
/// default unless the file gives it.
// @A grants file read with every key it may hold and none other,IMPL_GRANTS_READ,impl,[CREQ_GRANTS_READS, CREQ_GRANTS_REFUSES_UNREADABLE],[DEC_GRANTS_IN_A_FILE_OF_THEIR_OWN, DEC_GRANTS_NARROW_BY_DEFAULT, DEC_UNKNOWN_KEYS_REFUSED]
pub fn read_grants(grants: &Path) -> Result<Grants, GrantsFault> {
    let file = grants.display().to_string();
    let text = read_text(grants, &file).map_err(GrantsFault::File)?;
    let at = |(place, fault)| GrantsFault::Grants { place, fault };
    let toml = Toml::parse(&file, &text).map_err(at)?;
    let root = toml.root();
    let top: &[String] = &[];
    toml.only(
        root,
        top,
        &[
            "image", "images", "network", "actions", "folders", "trust", "limits",
        ],
    )
    .map_err(at)?;
    let key = |name: &str| path(top, name);

    let item = toml.needed(root, top, "image").map_err(at)?;
    let image = toml.string(item, &key("image")).map_err(at)?;
    if !pinned(image) {
        return Err(GrantsFault::Image {
            place: toml.place(item.span()),
            image: image.to_owned(),
        });
    }

    let images = match root.get("images") {
        Some(item) => images(&toml, item, &key("images"))?,
        None => Vec::new(),
    };

    let network = match root.get("network") {
        Some(item) => toml.boolean(item, &key("network")).map_err(at)?,
        None => false,
    };

    let mut actions = Vec::new();
    if let Some(item) = root.get("actions") {
        let names = toml.strings(item, &key("actions")).map_err(at)?;
        let values = item.as_array().into_iter().flatten();
        for (name, value) in names.iter().zip(values) {
            let action = Action::named(name).ok_or_else(|| GrantsFault::Action {
                place: toml.place(value.span()),
                action: name.clone(),
            })?;
            if !actions.contains(&action) {
                actions.push(action);
            }
        }
    }

    let directory = grants.parent().unwrap_or(Path::new(""));
    let mut folders = Vec::new();
    if let Some(item) = root.get("folders") {
        let under = key("folders");
        let table = toml.table(item, &under).map_err(at)?;
        for (name, item) in table.iter() {
            let named = table.get_key_value(name).and_then(|(k, _)| k.span());
            folders.push(folder(&toml, directory, &under, name, named, item)?);
        }
    }

    let trust = match root.get("trust") {
        Some(item) => Some(trust(&toml, directory, item, &key("trust"))?),
        None => None,
    };

    let mut limits = CommandLimits::default();
    if let Some(item) = root.get("limits") {
        let under = key("limits");
        let table = toml.table(item, &under).map_err(at)?;
        toml.only(table, &under, &["seconds", "output", "tmp"])
            .map_err(at)?;
        if let Some(item) = table.get("seconds") {
            limits.seconds =
                at_least_one(&toml, item, &path(&under, "seconds"), u32::MAX.into())? as u32;
        }
        if let Some(item) = table.get("output") {
            limits.output =
                at_least_one(&toml, item, &path(&under, "output"), usize::MAX as u64)? as usize;
        }
        if let Some(item) = table.get("tmp") {
            limits.tmp = at_least_one(&toml, item, &path(&under, "tmp"), i64::MAX as u64)?;
        }
    }

    Ok(Grants {
        image: image.to_owned(),
        images,
        folders,
        network,
        actions,
        trust,
        limits,
    })
}

/// The path of the trust file `item`, under `key`, names, read from
/// `directory` with any link in it resolved - or refused at that key when it
/// is not a file that can be read.
// @A trust file read from the grants file's directory or refused at its key,IMPL_GRANTS_TRUST,impl,[CREQ_GRANTS_READS_TRUST, CREQ_GRANTS_REFUSES_BAD_TRUST],[DEC_TRUST_IN_THE_GRANTS]
fn trust(
    toml: &Toml<'_>,
    directory: &Path,
    item: &toml_edit::Item,
    key: &[String],
) -> Result<PathBuf, GrantsFault> {
    let written = toml
        .string(item, key)
        .map_err(|(place, fault)| GrantsFault::Grants { place, fault })?;
    let joined = directory.join(written);
    let refused = |message: String| GrantsFault::Trust {
        place: toml.place(item.span()),
        path: written.to_owned(),
        message,
    };
    let file = std::fs::canonicalize(&joined).map_err(|error| refused(error.to_string()))?;
    let file = unverbatim(file);
    match std::fs::metadata(&file) {
        Ok(metadata) if !metadata.is_file() => Err(refused("it is not a file".to_owned())),
        Ok(_) => match std::fs::File::open(&file) {
            Ok(_) => Ok(file),
            Err(error) => Err(refused(error.to_string())),
        },
        Err(error) => Err(refused(error.to_string())),
    }
}

/// The folder `name`, whose key is at `named`, read from `item` under `under`,
/// its path joined to `directory` - or refused at its name or its path.
// @A folder named as one part of a path and granted an existing directory,IMPL_GRANTS_FOLDER,impl,[CREQ_GRANTS_REFUSES_BAD_FOLDER],[DEC_PATHS_IN_GRANTED_FOLDERS]
fn folder(
    toml: &Toml<'_>,
    directory: &Path,
    under: &[String],
    name: &str,
    named: Option<std::ops::Range<usize>>,
    item: &toml_edit::Item,
) -> Result<Folder, GrantsFault> {
    let at = |(place, fault)| GrantsFault::Grants { place, fault };
    let refused = |place, fault| GrantsFault::Folder {
        place,
        name: name.to_owned(),
        fault,
    };
    if !plain(name) {
        return Err(refused(toml.place(named), FolderFault::Name));
    }
    let key = path(under, name);
    let table = toml.table(item, &key).map_err(at)?;
    toml.only(table, &key, &["path", "writable"]).map_err(at)?;
    let item = toml.needed(table, &key, "path").map_err(at)?;
    let written = toml.string(item, &path(&key, "path")).map_err(at)?;
    let writable = match table.get("writable") {
        Some(item) => toml.boolean(item, &path(&key, "writable")).map_err(at)?,
        None => false,
    };

    let joined = directory.join(written);
    match std::fs::metadata(&joined) {
        Ok(metadata) if metadata.is_dir() => Ok(Folder {
            name: name.to_owned(),
            path: joined,
            writable,
        }),
        Ok(_) => Err(refused(
            toml.place(item.span()),
            FolderFault::NotADirectory {
                path: written.to_owned(),
            },
        )),
        Err(error) => Err(refused(
            toml.place(item.span()),
            FolderFault::Absent {
                path: written.to_owned(),
                message: error.to_string(),
            },
        )),
    }
}

/// The images `item`, under `key`, names, each pinned, or refused at the
/// first that is not.
// @Each image a tool may name read and every one pinned,IMPL_GRANTS_IMAGES,impl,[CREQ_GRANTS_READS_IMAGES, CREQ_GRANTS_REFUSES_TAGGED_IMAGES],[DEC_GRANTS_LIST_IMAGES, DEC_IMAGE_BY_DIGEST_NEVER_PULLED]
fn images(
    toml: &Toml<'_>,
    item: &toml_edit::Item,
    key: &[String],
) -> Result<Vec<String>, GrantsFault> {
    let named = toml
        .strings(item, key)
        .map_err(|(place, fault)| GrantsFault::Grants { place, fault })?;
    let values = item.as_array().into_iter().flatten();
    for (image, value) in named.iter().zip(values) {
        if !pinned(image) {
            return Err(GrantsFault::Image {
                place: toml.place(value.span()),
                image: image.clone(),
            });
        }
    }
    Ok(named)
}

/// `path` without the `\\?\` prefix Windows gives a resolved path on a drive,
/// as [`std::path::absolute`] writes one; any other path as it is.
fn unverbatim(path: PathBuf) -> PathBuf {
    if cfg!(windows) {
        let drive = path
            .to_str()
            .and_then(|text| text.strip_prefix(r"\\?\"))
            .filter(|rest| !rest.starts_with(r"UNC\"))
            .map(PathBuf::from);
        if let Some(drive) = drive {
            return drive;
        }
    }
    path
}

/// Whether `name` is one plain part of a path: not empty, not `.` or `..`,
/// and holding no separator and no control character.
fn plain(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
}

/// Whether `image` names an image by a digest - a name, `@sha256:` and 64
/// lowercase hexadecimal digits - or by an image id - `sha256:` and the same.
/// A name may not begin with `-`, hold `@`, a space or a control character.
// @An image named by a digest or an image id and nothing else,IMPL_GRANTS_IMAGE,impl,[CREQ_GRANTS_REFUSES_TAGGED_IMAGE],[DEC_IMAGE_BY_DIGEST_NEVER_PULLED]
pub(crate) fn pinned(image: &str) -> bool {
    let digest = |digits: &str| {
        digits.len() == 64
            && digits
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
    };
    if let Some(digits) = image.strip_prefix("sha256:") {
        return digest(digits);
    }
    match image.split_once("@sha256:") {
        Some((name, digits)) => {
            !name.is_empty()
                && !name.starts_with('-')
                && !name
                    .chars()
                    .any(|c| c == '@' || c.is_whitespace() || c.is_control())
                && digest(digits)
        }
        None => false,
    }
}

/// `item`, under `key`, read as a whole number from one to `most`.
fn at_least_one(
    toml: &Toml<'_>,
    item: &toml_edit::Item,
    key: &[String],
    most: u64,
) -> Result<u64, GrantsFault> {
    match toml.whole(item, key, most) {
        Ok(0) => Err(GrantsFault::Grants {
            place: toml.place(item.span()),
            fault: KeyFault::WrongKind {
                key: key.to_vec(),
                expected: "a whole number from 1",
                found: "0".to_owned(),
            },
        }),
        Ok(number) => Ok(number),
        Err((place, fault)) => Err(GrantsFault::Grants { place, fault }),
    }
}

// --- tests -------------------------------------------------------------------
// Bare functions named after their test cases.

#[cfg(test)]
use crate::testing::{IMAGE, Scratch};

/// The fault a grants file was refused for, or a panic naming what came
/// instead.
#[cfg(test)]
fn refused(scratch: &Scratch, text: &str) -> GrantsFault {
    match read_grants(&scratch.write("grants.toml", text)) {
        Err(fault) => fault,
        Ok(grants) => panic!("expected the grants refused, got {grants:?}"),
    }
}

#[cfg(test)]
#[test]
fn reads_what_it_names() {
    let scratch = Scratch::new("grants_reads_what_it_names");
    scratch.write("src/a.txt", "");
    scratch.write("notes/b.txt", "");
    let grants = scratch.write(
        "grants.toml",
        format!(
            "image = \"{IMAGE}\"\nnetwork = true\nactions = [\"run\", \"read\", \"write\", \"read\"]\n\n[folders.src]\npath = \"src\"\nwritable = true\n\n[folders.notes]\npath = \"notes\"\nwritable = false\n\n[limits]\nseconds = 5\noutput = 100\n"
        ),
    );

    let grants = read_grants(&grants).expect("the grants read");

    assert_eq!(grants.image(), IMAGE);
    assert_eq!(
        grants.folders(),
        [
            Folder {
                name: "src".to_owned(),
                path: scratch.path("src"),
                writable: true,
            },
            Folder {
                name: "notes".to_owned(),
                path: scratch.path("notes"),
                writable: false,
            },
        ]
    );
    assert!(grants.network());
    assert_eq!(grants.actions(), [Action::Run, Action::Read, Action::Write]);
    assert!(
        [Action::Read, Action::Write, Action::Run]
            .into_iter()
            .all(|action| grants.allows(action))
    );
    assert_eq!(
        grants.limits(),
        CommandLimits {
            seconds: 5,
            output: 100,
            tmp: 268_435_456,
        }
    );
}

#[cfg(test)]
#[test]
fn narrow_by_default() {
    let scratch = Scratch::new("grants_narrow_by_default");
    scratch.write("src/a.txt", "");
    let grants = scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\n\n[folders.src]\npath = \"src\"\n"),
    );

    let grants = read_grants(&grants).expect("the grants read");

    assert_eq!(grants.folders().len(), 1);
    assert!(!grants.folders()[0].writable);
    assert!(!grants.network());
    assert_eq!(grants.actions(), []);
    assert!(
        ![Action::Read, Action::Write, Action::Run]
            .into_iter()
            .any(|action| grants.allows(action))
    );
    assert_eq!(
        grants.limits(),
        CommandLimits {
            seconds: 60,
            output: 16384,
            tmp: 268_435_456,
        }
    );
}

#[cfg(test)]
#[test]
fn paths_from_the_file() {
    let scratch = Scratch::new("grants_paths_from_the_file");
    // The same path read from where the test stands names a directory too,
    // and another one.
    let from_here = Path::new("../agconflo-lua");
    assert!(from_here.is_dir());
    scratch.write("a/agconflo-lua/beside.txt", "");
    let grants = scratch.write(
        "a/b/grants.toml",
        format!("image = \"{IMAGE}\"\n\n[folders.beside]\npath = \"../agconflo-lua\"\n"),
    );

    let grants = read_grants(&grants).expect("the grants read");

    let granted = grants.folders()[0].path.canonicalize().expect("the folder");
    assert_eq!(
        granted,
        scratch
            .path("a/agconflo-lua")
            .canonicalize()
            .expect("the folder")
    );
    assert_ne!(granted, from_here.canonicalize().expect("the directory"));
}

#[cfg(test)]
#[test]
fn unreadable_refused() {
    let scratch = Scratch::new("grants_unreadable_refused");
    scratch.write("src/a.txt", "");

    let absent = scratch.path("absent.toml");
    match read_grants(&absent) {
        Err(GrantsFault::File(FileFault::Unreadable { file, message })) => {
            assert_eq!(file, absent.display().to_string());
            assert!(!message.is_empty());
        }
        other => panic!("expected the file unreadable, got {other:?}"),
    }

    let undecodable = scratch.write("undecodable.toml", b"image = \"\xff\"\n");
    assert_eq!(
        read_grants(&undecodable),
        Err(GrantsFault::File(FileFault::NotText {
            file: undecodable.display().to_string()
        }))
    );

    let named = scratch.path("grants.toml").display().to_string();
    match refused(
        &scratch,
        &format!("image = \"{IMAGE}\"\nnetwork = true\n    = 1\n"),
    ) {
        GrantsFault::Grants {
            place,
            fault: KeyFault::Syntax { .. },
        } => {
            assert_eq!(place.file, named);
            assert_eq!((place.line, place.column), (3, 5));
        }
        other => panic!("expected the TOML refused, got {other:?}"),
    }

    let key = |names: &[&str]| names.iter().map(|n| (*n).to_owned()).collect::<Vec<_>>();
    for (text, at, expected) in [
        (
            "network = true\n".to_owned(),
            (1, 1),
            KeyFault::Missing {
                key: key(&["image"]),
            },
        ),
        (
            format!("image = \"{IMAGE}\"\nnetwrk = true\n"),
            (2, 1),
            KeyFault::Unexpected {
                key: key(&["netwrk"]),
            },
        ),
        (
            format!("image = \"{IMAGE}\"\n\n[folders.src]\npath = \"src\"\nwriteable = true\n"),
            (5, 1),
            KeyFault::Unexpected {
                key: key(&["folders", "src", "writeable"]),
            },
        ),
        (
            format!("image = \"{IMAGE}\"\n[limits]\nseconds = 0\n"),
            (3, 11),
            KeyFault::WrongKind {
                key: key(&["limits", "seconds"]),
                expected: "a whole number from 1",
                found: "0".to_owned(),
            },
        ),
    ] {
        match refused(&scratch, &text) {
            GrantsFault::Grants { place, fault } => {
                assert_eq!(place.file, named);
                assert_eq!((place.line, place.column), at, "{fault}");
                assert_eq!(fault, expected);
            }
            other => panic!("expected a fault in the file, got {other:?}"),
        }
    }

    match refused(
        &scratch,
        &format!("image = \"{IMAGE}\"\nactions = [\"read\", \"rn\"]\n"),
    ) {
        GrantsFault::Action { place, action } => {
            assert_eq!(action, "rn");
            assert_eq!((place.line, place.column), (2, 20));
        }
        other => panic!("expected the action refused, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn tagged_image_refused() {
    let scratch = Scratch::new("grants_tagged_image_refused");
    let digits = "0123456789abcdef".repeat(4);
    let refusals = [
        "alpine".to_owned(),
        "alpine:3".to_owned(),
        "alpine:sha256".to_owned(),
        format!("alpine@sha256:{}", &digits[1..]),
        format!("alpine@sha256:{digits}0"),
        format!("alpine@sha256:{}A", &digits[1..]),
        format!("@sha256:{digits}"),
        format!("-x@sha256:{digits}"),
        format!("sha256:{}", &digits[1..]),
    ];
    for image in refusals {
        match refused(&scratch, &format!("image = \"{image}\"\n")) {
            GrantsFault::Image {
                place,
                image: named,
            } => {
                assert_eq!(named, image);
                assert_eq!((place.line, place.column), (1, 9));
            }
            other => panic!("expected {image} refused, got {other:?}"),
        }
    }

    for image in [
        format!("alpine@sha256:{digits}"),
        format!("registry.example:5000/a/b:3@sha256:{digits}"),
        format!("sha256:{digits}"),
    ] {
        let grants = read_grants(&scratch.write("grants.toml", format!("image = \"{image}\"\n")))
            .expect("a pinned image read");
        assert_eq!(grants.image(), image);
    }
}

#[cfg(test)]
#[test]
fn bad_folder_refused() {
    let scratch = Scratch::new("grants_bad_folder_refused");
    scratch.write("src/a.txt", "");
    scratch.write("file.txt", "not a folder");

    for (written, absent) in [("nowhere", true), ("file.txt", false)] {
        let fault = refused(
            &scratch,
            &format!("image = \"{IMAGE}\"\n\n[folders.src]\npath = \"{written}\"\n"),
        );
        match fault {
            GrantsFault::Folder { place, name, fault } => {
                assert_eq!(name, "src");
                assert_eq!((place.line, place.column), (4, 8));
                match fault {
                    FolderFault::Absent { path, message } if absent => {
                        assert_eq!(path, written);
                        assert!(!message.is_empty());
                    }
                    FolderFault::NotADirectory { path } if !absent => assert_eq!(path, written),
                    other => panic!("expected {written} refused for itself, got {other:?}"),
                }
            }
            other => panic!("expected the folder refused, got {other:?}"),
        }
    }
    assert!(!scratch.path("nowhere").exists());

    for name in ["a/b", "a\\\\b", "..", ".", ""] {
        match refused(
            &scratch,
            &format!("image = \"{IMAGE}\"\n\n[folders.\"{name}\"]\npath = \"src\"\n"),
        ) {
            GrantsFault::Folder {
                place,
                fault: FolderFault::Name,
                ..
            } => assert_eq!(place.line, 3),
            other => panic!("expected the name {name:?} refused, got {other:?}"),
        }
    }

    let grants = read_grants(&scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\n\n[folders.src-2]\npath = \"src\"\n"),
    ))
    .expect("a plain name read");
    assert_eq!(grants.folders()[0].name, "src-2");
}

#[cfg(test)]
#[test]
fn trust_read() {
    let scratch = Scratch::new("grants_trust_read");
    scratch.write("certs/ca.pem", "");
    let file = std::fs::canonicalize(scratch.path("certs/ca.pem")).expect("the file");
    assert_ne!(
        std::env::current_dir().expect("a working directory"),
        scratch.path("")
    );

    let grants = read_grants(&scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\ntrust = \"certs/ca.pem\"\n"),
    ))
    .expect("the grants read");
    let given = grants.trust().expect("a trust file");
    assert!(given.is_absolute(), "{}", given.display());
    assert!(
        !given.to_string_lossy().starts_with(r"\\?\"),
        "{}",
        given.display()
    );
    assert_eq!(std::fs::canonicalize(given).expect("the file"), file);

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("certs/ca.pem", scratch.path("link.pem")).expect("a link");
        let grants = read_grants(&scratch.write(
            "grants.toml",
            format!("image = \"{IMAGE}\"\ntrust = \"link.pem\"\n"),
        ))
        .expect("the grants read");
        assert_eq!(grants.trust(), Some(file.as_path()));
    }

    let grants = read_grants(&scratch.write("grants.toml", format!("image = \"{IMAGE}\"\n")))
        .expect("the grants read");
    assert_eq!(grants.trust(), None);
}

#[cfg(test)]
#[test]
fn trust_refused() {
    let scratch = Scratch::new("grants_trust_refused");
    scratch.write("certs/ca.pem", "");

    for written in ["nowhere.pem", "certs"] {
        match refused(
            &scratch,
            &format!("image = \"{IMAGE}\"\ntrust = \"{written}\"\n"),
        ) {
            GrantsFault::Trust {
                place,
                path,
                message,
            } => {
                assert_eq!((place.line, place.column), (2, 9));
                assert_eq!(path, written);
                assert!(!message.is_empty());
            }
            other => panic!("expected {written} refused as the trust file, got {other:?}"),
        }
    }

    match refused(&scratch, &format!("image = \"{IMAGE}\"\ntrust = 1\n")) {
        GrantsFault::Grants {
            place,
            fault: KeyFault::WrongKind { key, .. },
        } => {
            assert_eq!(place.line, 2);
            assert_eq!(key, ["trust"]);
        }
        other => panic!("expected a number refused as the trust file, got {other:?}"),
    }
}

#[cfg(test)]
#[test]
fn reads_images() {
    let scratch = Scratch::new("grants_reads_images");
    let python = format!("python@sha256:{}", "a".repeat(64));
    let other = format!("sha256:{}", "b".repeat(64));
    let grants = read_grants(&scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\nimages = [\"{python}\", \"{other}\"]\n"),
    ))
    .expect("the grants read");

    assert_eq!(grants.image(), IMAGE);
    assert_eq!(grants.images(), [python.clone(), other.clone()]);
    for image in [IMAGE, python.as_str(), other.as_str()] {
        assert!(grants.allows_image(image), "{image}");
    }
    assert!(!grants.allows_image(&format!("python@sha256:{}", "c".repeat(64))));

    let grants = read_grants(&scratch.write("grants.toml", format!("image = \"{IMAGE}\"\n")))
        .expect("the grants read");
    assert_eq!(grants.images(), [] as [String; 0]);
    assert!(grants.allows_image(IMAGE));
    assert!(!grants.allows_image(&python));
}

#[cfg(test)]
#[test]
fn tagged_images_refused() {
    let scratch = Scratch::new("grants_tagged_images_refused");
    let pinned = format!("python@sha256:{}", "a".repeat(64));
    for (images, column) in [
        (
            format!("[\"{pinned}\", \"python:3\"]"),
            11 + pinned.len() + 4,
        ),
        ("[\"python:3\"]".to_owned(), 11),
    ] {
        match refused(
            &scratch,
            &format!("image = \"{IMAGE}\"\nimages = {images}\n"),
        ) {
            GrantsFault::Image { place, image } => {
                assert_eq!(image, "python:3");
                assert_eq!((place.line, place.column), (2, column), "{images}");
            }
            other => panic!("expected python:3 refused, got {other:?}"),
        }
    }

    read_grants(&scratch.write(
        "grants.toml",
        format!("image = \"{IMAGE}\"\nimages = [\"{pinned}\"]\n"),
    ))
    .expect("pinned images read");
}

#[cfg(test)]
#[test]
fn tmp_limit_read() {
    let scratch = Scratch::new("grants_tmp_limit_read");
    let limits = |extra: &str| {
        read_grants(&scratch.write("grants.toml", format!("image = \"{IMAGE}\"\n{extra}")))
            .map(|grants| grants.limits().tmp)
    };

    assert_eq!(limits("[limits]\ntmp = 1048576\n"), Ok(1_048_576));
    assert_eq!(limits(""), Ok(268_435_456));
    assert_eq!(limits("[limits]\nseconds = 5\n"), Ok(268_435_456));
    match limits("[limits]\ntmp = 0\n") {
        Err(GrantsFault::Grants {
            fault: KeyFault::WrongKind { key, .. },
            ..
        }) => assert_eq!(key, ["limits", "tmp"]),
        other => panic!("expected a /tmp of 0 refused, got {other:?}"),
    }
}
