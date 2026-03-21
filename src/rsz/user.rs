use super::Result;
use crate::check_magic;
use crate::layout::LayoutMap;
use crate::rsz::content::{Content, Magic, RszStream, SliceStream};
use std::fmt::Debug;
use std::fs;
use std::path::Path;
use zerocopy::{FromBytes, KnownLayout};

#[derive(Debug)]
pub struct User {
    /// The [Content] section in the RSZ document.
    pub content: Content,
}

impl User {
    pub const MAGIC: Magic = 0x525355;

    /// Loads a USER document from `path` and attempts to parse it.
    pub fn load<P>(path: P, layout: &LayoutMap) -> Result<Self>
    where
        P: AsRef<Path> + Debug,
    {
        log::info!("Loading USER document, path = {path:?}");

        let data = fs::read(path)?;
        Self::parse(&mut SliceStream::from(&data[..]), layout)
    }

    /// Parses an RSZ stream into a USER document.
    pub fn parse<T: RszStream>(data: &mut T, layout: &LayoutMap) -> Result<Self> {
        let header: Header = data.next_section()?;
        check_magic!(Self::MAGIC, header.magic);

        let offset = header.data_offset;
        log::trace!("Seeking to RSZ document, offset = {offset:#X}");
        data.seek(offset as usize)?;

        Ok(Self {
            content: Content::parse(&mut data.as_relative(), layout)?,
        })
    }
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
struct Header {
    magic: Magic,
    instance_count: i32,
    intern_count: i32,
    info_count: i32,
    instance_offset: u64,
    intern_offset: u64,
    data_offset: u64,
}
