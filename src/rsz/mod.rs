use crate::layout::{FieldKind, LayoutMap, TypeId};
use crate::rsz::content::{Magic, RszStream, SliceStream};
use crate::rsz::user::User;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub mod content;
pub mod user;

type Result<T> = std::result::Result<T, Error>;

/// Represents any supported type of RSZ document.
#[derive(Debug)]
pub enum Rsz {
    User(User),
}

impl Rsz {
    /// Attempts to load an RSZ document from the given `path`.
    ///
    /// The file referenced by `path` must be a regular, readable file. The entire file will be
    /// loaded into memory.
    pub fn load(path: &Path, layout: &LayoutMap) -> Result<Self> {
        Self::read_from(&mut File::open(path)?, layout)
    }

    /// Attempts to read an RSZ document from any type implementing [Read]. Document type will be
    /// inferred from the magic 4-byte sequence at the start of the file.
    pub fn read_from<R: Read>(reader: &mut R, layout: &LayoutMap) -> Result<Self> {
        let mut data: Vec<u8> = Vec::new();
        reader.read_to_end(&mut data)?;

        let mut data = SliceStream::from(&data[..]);
        let magic: Magic = data.next_section()?;

        // Reset the stream so document parsing begins at the top of file.
        data.seek(0)?;

        let rsz = match magic {
            User::MAGIC => Self::User(User::parse(&mut data, layout)?),
            _ => return Err(Error::UnrecognizedMagic(magic)),
        };

        Ok(rsz)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unrecognized magic value 0x{0:#06X} in file")]
    UnrecognizedMagic(u32),

    #[error("expected magic value {expected:#06X} but got {actual:#06X}")]
    MagicMismatch { expected: u32, actual: u32 },

    #[error("unrecognized type id {0}")]
    UnknownLayoutId(TypeId),

    #[error("unexpected end of file: length is {length}, tried to read {position}")]
    UnexpectedEof { position: usize, length: usize },

    #[error("field type '{0:?}' is not yet implemented")]
    UnimplementedFieldType(FieldKind),

    #[error("object not found while resolving reference, index = {0}")]
    ObjectNotFound(usize),

    #[error("invalid byte section: name = {type_name}, len = {length}")]
    InvalidSection {
        type_name: &'static str,
        length: usize,
    },
}

#[macro_export]
macro_rules! check_magic {
    ($expected:expr, $actual:expr) => {{
        let expected = $expected;
        let actual = $actual;

        log::debug!(
            "Checking magic values match: expected = {expected:#06X}, actual = {actual:#06X}"
        );

        if expected != actual {
            return std::result::Result::Err($crate::rsz::Error::MagicMismatch {
                expected,
                actual,
            });
        }
    }};
}
