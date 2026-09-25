//! Files the runner is named, read as text, and the TOML documents among them
//! read value by value, each fault with its place.

use std::fmt;
use std::ops::Range;
use std::path::Path;

use toml_edit::{Document, Item, TableLike};

/// Where in a file a fault is: the file as it was named, and the line and the
/// column in characters, each counted from one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Place {
    /// The file, as the person or the manifest named it.
    pub file: String,
    /// The line, counted from one.
    pub line: usize,
    /// The column in characters, counted from one.
    pub column: usize,
}

impl fmt::Display for Place {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// A file that could not be read as text.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FileFault {
    /// The file is not there, or could not be read.
    Unreadable {
        /// The file, as it was named.
        file: String,
        /// The operating system's account of why.
        message: String,
    },
    /// The file is not UTF-8.
    NotText {
        /// The file, as it was named.
        file: String,
    },
}

impl fmt::Display for FileFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { file, message } => write!(f, "{file}: cannot be read: {message}"),
            Self::NotText { file } => write!(f, "{file}: is not UTF-8 text"),
        }
    }
}

/// What is wrong at one place in a TOML document the runner reads.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum KeyFault {
    /// The text is not TOML.
    Syntax {
        /// The parser's own account, without its place.
        message: String,
    },
    /// A key the document may not hold.
    Unexpected {
        /// The key, from the top of the document down.
        key: Vec<String>,
    },
    /// A key the document must hold is absent.
    Missing {
        /// The key, from the top of the document down.
        key: Vec<String>,
    },
    /// A value is not of the kind its key holds.
    WrongKind {
        /// The key, from the top of the document down.
        key: Vec<String>,
        /// The kind of value the key holds.
        expected: &'static str,
        /// What it holds instead.
        found: String,
    },
}

impl fmt::Display for KeyFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Syntax { message } => write!(f, "not TOML: {message}"),
            Self::Unexpected { key } => write!(f, "{} is not a key this file holds", key.join(".")),
            Self::Missing { key } => write!(f, "{} is missing", key.join(".")),
            Self::WrongKind {
                key,
                expected,
                found,
            } => write!(f, "{} holds {found}, not {expected}", key.join(".")),
        }
    }
}

/// The text of the file at `path`, named `file` in a fault.
pub(crate) fn read_text(path: &Path, file: &str) -> Result<String, FileFault> {
    let bytes = std::fs::read(path).map_err(|error| FileFault::Unreadable {
        file: file.to_owned(),
        message: error.to_string(),
    })?;
    String::from_utf8(bytes).map_err(|_| FileFault::NotText {
        file: file.to_owned(),
    })
}

/// A TOML document being read, named `file` in its faults.
pub(crate) struct Toml<'t> {
    file: &'t str,
    text: &'t str,
    document: Document<&'t str>,
}

impl<'t> Toml<'t> {
    /// `text` parsed, or refused where the parser stopped.
    pub(crate) fn parse(file: &'t str, text: &'t str) -> Result<Self, (Place, KeyFault)> {
        let document = Document::parse(text).map_err(|refused| {
            (
                place(file, text, refused.span()),
                KeyFault::Syntax {
                    message: refused.message().to_owned(),
                },
            )
        })?;
        Ok(Self {
            file,
            text,
            document,
        })
    }

    /// The document's top-level table.
    pub(crate) fn root(&self) -> &dyn TableLike {
        self.document.as_table()
    }

    /// The place `span` starts at, or the document's start for none.
    pub(crate) fn place(&self, span: Option<Range<usize>>) -> Place {
        place(self.file, self.text, span)
    }

    /// Nothing, or the first key of `table`, under `key`, that is not in
    /// `allowed`, refused at that key.
    pub(crate) fn only(
        &self,
        table: &dyn TableLike,
        key: &[String],
        allowed: &[&str],
    ) -> Result<(), (Place, KeyFault)> {
        match table.iter().find(|(name, _)| !allowed.contains(name)) {
            None => Ok(()),
            Some((name, _)) => Err((
                self.place(table.get_key_value(name).and_then(|(k, _)| k.span())),
                KeyFault::Unexpected {
                    key: path(key, name),
                },
            )),
        }
    }

    /// The item under `name` in `table`, or a fault at the document's start for
    /// its absence.
    pub(crate) fn needed<'i>(
        &self,
        table: &'i dyn TableLike,
        key: &[String],
        name: &str,
    ) -> Result<&'i Item, (Place, KeyFault)> {
        table.get(name).ok_or_else(|| {
            (
                self.place(None),
                KeyFault::Missing {
                    key: path(key, name),
                },
            )
        })
    }

    /// `item`, under `key`, read as a string.
    pub(crate) fn string<'i>(
        &self,
        item: &'i Item,
        key: &[String],
    ) -> Result<&'i str, (Place, KeyFault)> {
        item.as_str()
            .ok_or_else(|| self.wrong_kind(item, key, "a string"))
    }

    /// `item`, under `key`, read as an array of strings.
    pub(crate) fn strings(
        &self,
        item: &Item,
        key: &[String],
    ) -> Result<Vec<String>, (Place, KeyFault)> {
        let array = item
            .as_array()
            .ok_or_else(|| self.wrong_kind(item, key, "an array of strings"))?;
        array
            .iter()
            .map(|value| {
                value.as_str().map(str::to_owned).ok_or_else(|| {
                    (
                        self.place(value.span()),
                        KeyFault::WrongKind {
                            key: key.to_vec(),
                            expected: "an array of strings",
                            found: format!("an array holding {}", article(value.type_name())),
                        },
                    )
                })
            })
            .collect()
    }

    /// `item`, under `key`, read as a table, whichever of TOML's ways of
    /// writing one it is.
    pub(crate) fn table<'i>(
        &self,
        item: &'i Item,
        key: &[String],
    ) -> Result<&'i dyn TableLike, (Place, KeyFault)> {
        item.as_table_like()
            .ok_or_else(|| self.wrong_kind(item, key, "a table"))
    }

    /// `item`, under `key`, read as a whole number from zero to `most`.
    pub(crate) fn whole(
        &self,
        item: &Item,
        key: &[String],
        most: u64,
    ) -> Result<u64, (Place, KeyFault)> {
        match item.as_integer() {
            Some(number) if number >= 0 && number as u64 <= most => Ok(number as u64),
            Some(number) => Err((
                self.place(item.span()),
                KeyFault::WrongKind {
                    key: key.to_vec(),
                    expected: "a whole number from 0",
                    found: number.to_string(),
                },
            )),
            None => Err(self.wrong_kind(item, key, "a whole number from 0")),
        }
    }

    fn wrong_kind(&self, item: &Item, key: &[String], expected: &'static str) -> (Place, KeyFault) {
        (
            self.place(item.span()),
            KeyFault::WrongKind {
                key: key.to_vec(),
                expected,
                found: article(item.type_name()),
            },
        )
    }
}

/// `key` with `name` after it.
pub(crate) fn path(key: &[String], name: &str) -> Vec<String> {
    let mut path = key.to_vec();
    path.push(name.to_owned());
    path
}

/// A kind of value as toml_edit names it, with its article.
pub(crate) fn article(kind: &str) -> String {
    match kind.chars().next() {
        Some('a' | 'e' | 'i' | 'o' | 'u') => format!("an {kind}"),
        _ => format!("a {kind}"),
    }
}

/// The place `span` starts at in `text`, or its start for none.
fn place(file: &str, text: &str, span: Option<Range<usize>>) -> Place {
    let (line, column) = line_and_column(text, span.map_or(0, |span| span.start));
    Place {
        file: file.to_owned(),
        line,
        column,
    }
}

/// The line and column of byte `offset` in `text`, counted from one, the column
/// in characters, as the topology reader counts them: an offset at or past the
/// end is placed just after the last character, and one inside a character
/// wider than a byte falls back to counting bytes.
fn line_and_column(text: &str, offset: usize) -> (usize, usize) {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return (1, offset + 1);
    }

    let within = offset.min(bytes.len() - 1);
    let past_the_end = offset - within;
    let line_start = bytes[..within]
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(0, |newline| newline + 1);
    let line = bytes[..line_start]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count();
    let column = std::str::from_utf8(&bytes[line_start..=within])
        .map_or(within - line_start, |up_to| up_to.chars().count() - 1);

    (line + 1, column + past_the_end + 1)
}
