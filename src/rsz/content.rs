use super::{Error, Result};
use crate::check_magic;
use crate::layout::{FieldKind, FieldLayout, LayoutMap, TypeId};
use crate::types::mat4::Mat4;
use crate::types::vec3::Vec3;
use half::f16;
use serde::Serialize;
use std::any::type_name;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use strum_macros::EnumTryAs;
use uuid::Uuid;
use zerocopy::{FromBytes, KnownLayout};

pub type Magic = u32;

/// The data content of an RSZ document.
#[derive(Debug, Default)]
pub struct Content {
    /// A collection of all object instances contained in the document.
    pub instances: Objects,

    /// Interned strings from the RSZ document.
    ///
    /// As far as I can tell, this is only used for path references.
    pub interned_strings: InternedStrings,

    /// A collection of root objects contained in the document.
    ///
    /// If you need to iterate over the document, this is your starting point.
    pub root_objects: Objects,
}

impl Content {
    pub const MAGIC: Magic = 0x5A5352;

    /// Parse a content section out of an RSZ document.
    pub fn parse<T: RszStream>(data: &mut T, layout: &LayoutMap) -> Result<Self> {
        let header: Header = data.next_section()?;
        check_magic!(Self::MAGIC, header.magic);

        // region Roots
        let mut roots: Vec<ObjectReference> = Vec::with_signed_capacity(header.object_count);

        for _ in 0..header.object_count {
            roots.push(data.next_section()?);
        }

        log::debug!("Found {} root object(s)", roots.len());
        // endregion

        let mut content = Self::default();

        // region Interned Strings
        content.interned_strings.reserve_signed(header.intern_count);
        data.seek(header.intern_offset as usize)?;

        for _ in 0..header.intern_count {
            let info: InternedString = data.next_section()?;

            {
                let index = info.index;
                let offset = info.offset;

                log::trace!(
                    ">> Found external reference for slot {} at relative offset = {:#X}",
                    index,
                    offset
                );
            }

            data.seek(info.offset as usize)?;

            let value = read_string(data, None)?;
            log::debug!("value = {value}");

            content.interned_strings.insert(info.index, value);
        }

        log::debug!(
            "Found {} interned string(s)",
            content.interned_strings.len()
        );
        // endregion

        // region Instances
        let mut instances = Vec::with_signed_capacity(header.instance_count);
        data.seek(header.instance_offset as usize)?;

        for _ in 0..header.instance_count {
            let instance: Instance = data.next_section()?;

            {
                let id = instance.type_id;
                log::trace!(">> Found instance ID {:x} at index {}", id, instances.len());
            }

            instances.push(instance);
        }

        log::debug!("Found {} instance(s)", instances.len());
        // endregion

        // region Objects
        content.instances.reserve(instances.len());
        data.seek(header.data_offset as usize)?;

        for (index, info) in instances.into_iter().enumerate() {
            let Some(layout) = layout.get_layout(info.type_id) else {
                return Err(Error::UnknownLayoutId(info.type_id));
            };

            {
                let type_id = info.type_id;

                if !layout.name.is_empty() {
                    log::debug!(
                        "Found named type {} ({type_id:x}) at index = {index}",
                        layout.name
                    );
                } else {
                    log::debug!("Found anonymous type ID {type_id:x} at index = {index}");
                }
            }

            // Also referred to as "external references" or "userdata", these are basically just
            // interned strings. These are stored in the data section of the document, and are
            // assigned an index which is the "slot" they occupy in the instance list.
            let object = if let Some(intern_ref) = content.interned_strings.get(&(index as u32)) {
                Rc::new(Object {
                    name: layout.name.to_owned(),
                    fields: vec![Field {
                        name: "Path".to_owned(),
                        value: Value::String(intern_ref.to_owned()),
                    }],
                })
            } else {
                Rc::new(Object {
                    name: layout.name.to_owned(),
                    fields: layout
                        .fields
                        .iter()
                        .map(|v| data.next_field(v, &content))
                        .collect::<Result<_>>()?,
                })
            };

            content.instances.push(object);
        }
        // endregion

        content.root_objects.reserve(roots.len());

        for root in roots {
            let index = root.index.unsigned_abs() as usize;
            content.root_objects.push(content.instances[index].clone());
        }

        Ok(content)
    }
}

/// The RSZ header from a `Content` section.
#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct Header {
    /// The magic 4-byte sequence that identifies the section.
    pub magic: Magic,

    /// The version number.
    pub version: u32,

    /// The number of roots in the object graph.
    pub object_count: i32,

    /// The number of instances defined in the entire object graph.
    pub instance_count: i32,

    /// The number of interned strings defined in the section.
    pub intern_count: i32,

    /// An unknown 4-byte sequence. Doesn't appear to be needed.
    pub _reserved: u32,

    /// The position in the section where instances begin.
    ///
    /// This is relative to the start of the `Content` section.
    pub instance_offset: u64,

    /// The position in the section where field data begins.
    ///
    /// This is relative to the start of the `Content` section.
    pub data_offset: u64,

    /// The position in the section where interned strings begin.
    ///
    /// This is relative to the start of the `Content` section.
    pub intern_offset: u64,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
pub struct ObjectReference {
    pub index: i32,
}

type InternedStrings = HashMap<u32, String>;

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
struct InternedString {
    pub index: u32,
    pub hash: u32,
    pub offset: u64,
}

#[derive(Debug, Default)]
pub struct Objects(Vec<Rc<Object>>);

impl Deref for Objects {
    type Target = Vec<Rc<Object>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Objects {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, FromBytes, KnownLayout)]
#[repr(C, packed)]
struct Instance {
    pub type_id: TypeId,
    pub crc: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Values(pub Vec<Value>);

impl Deref for Values {
    type Target = Vec<Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, EnumTryAs, PartialEq)]
#[serde(untagged)]
pub enum Value {
    /// An array of [Value] objects.
    Array(Values),

    /// A simple boolean value.
    Boolean(bool),

    /// A 16-bit floating point number, backed by an [f16] from the
    /// [`half`](https://docs.rs/half/latest/half/) library.
    F16(f16),

    /// A 32-bit floating point number.
    F32(f32),

    /// A 64-bit floating point number.
    F64(f64),

    /// A UUID, backed by a [UUID] from the [uuid](https://docs.rs/uuid/latest/uuid/) library.
    Guid(Uuid),

    /// An [Object] instance.
    Object(Rc<Object>),

    /// An 8-bit signed integer.
    S8(i8),

    /// A 16-bit signed integer.
    S16(i16),

    /// A 32-bit signed integer.
    S32(i32),

    /// A 64-bit signed integer.
    S64(i64),

    /// A string value.
    String(String),

    /// An 8-bit unsigned integer.
    U8(u8),

    /// A 16-bit unsigned integer.
    U16(u16),

    /// A 32-bit unsigned integer.
    U32(u32),

    /// A 64-bit unsigned integer.
    U64(u64),

    /// A 3-dimensional vector.
    Vec3(Vec3),

    /// A 4x4 matrix value.
    Mat4(Mat4),

    /// Not sure what this is, I just know it's backed by a `u8`, shows up in files I care about
    /// parsing, but isn't actually needed right now.
    Data(u8),
}

pub trait RszStream {
    /// Creates a new stream that starts at the current position of this stream.
    fn as_relative(&self) -> Self;

    /// Returns the current position of the stream.
    fn position(&self) -> usize;

    /// Returns the absolute position of the stream, relevant to the start of the file.
    ///
    /// This is not directly used by the library, and is provided for debugging and logging
    /// purposes.
    fn position_absolute(&self) -> usize;

    /// Seeks to a specific position within the stream.
    fn seek(&mut self, position: usize) -> Result<()>;

    /// Skips the next `len` bytes.
    fn skip(&mut self, len: usize) -> Result<()>;

    /// Attempts to align the stream to a byte width.
    ///
    /// Parsing some values (usually strings) can leave us misaligned, since not every type has a
    /// length equal to the byte alignment of every time. This method is called before attempting to
    /// parse a value to ensure that we're on the correct alignment for that value's type.
    fn align(&mut self, alignment: usize) -> Result<()>;

    /// Attempts to parse the field defined by `layout`, starting at the current position in the
    /// stream. The stream's position should be advanced by the width of the field.
    fn next_field<T: ParseField>(&mut self, layout: &T, partial_content: &Content)
    -> Result<Field>;

    /// Attempts to parse an arbitrary value from the stream. The stream's position should be
    /// advanced by the width of the type.
    fn next_section<T>(&mut self) -> Result<T>
    where
        T: FromBytes + KnownLayout;
}

/// An RSZ data stream over an in-memory slice of bytes.
pub struct SliceStream<'a> {
    /// The slice containing the RSZ document.
    data: &'a [u8],

    /// The current position in the stream.
    position: usize,

    /// The stream's cumulative offset relative to the original slice.
    ///
    /// This property is exclusively used for debugging and logging.
    global_offset: usize,
}

impl SliceStream<'_> {
    fn eof(&self) -> bool {
        self.position > self.data.len()
    }
}

impl<'a> From<&'a [u8]> for SliceStream<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            data: value,
            position: 0,
            global_offset: 0,
        }
    }
}

impl RszStream for SliceStream<'_> {
    fn as_relative(&self) -> Self {
        let global_position = self.position_absolute();
        log::debug!(
            "Creating new relative stream, base_position = {:#X} ({:#X})",
            self.position,
            global_position
        );

        Self {
            data: &self.data[self.position..],
            position: 0,
            global_offset: global_position,
        }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn position_absolute(&self) -> usize {
        self.global_offset + self.position
    }

    fn seek(&mut self, position: usize) -> Result<()> {
        self.position = position;
        log::trace!(
            "Seeking to {position:#X} (abs = {:#X})",
            self.position_absolute()
        );

        if self.eof() {
            Err(Error::UnexpectedEof {
                position,
                length: self.data.len(),
            })
        } else {
            Ok(())
        }
    }

    fn skip(&mut self, len: usize) -> Result<()> {
        log::trace!("Skipping {len} byte(s)");
        self.seek(self.position + len)
    }

    fn align(&mut self, alignment: usize) -> Result<()> {
        log::trace!(
            "Aligning to {alignment}, start = {:#X} (abs = {:#X})",
            self.position,
            self.position_absolute()
        );

        let delta = self.position % alignment;

        if delta != 0 {
            self.position += alignment - delta;
            log::trace!(
                ">> end = {:#X} (abs = {:#X})",
                self.position,
                self.position_absolute()
            );

            if self.eof() {
                return Err(Error::UnexpectedEof {
                    position: self.position,
                    length: self.data.len(),
                });
            }
        } else {
            log::trace!(">> Already aligned.");
        }

        Ok(())
    }

    fn next_field<T: ParseField>(&mut self, field: &T, partial_content: &Content) -> Result<Field> {
        field.parse(self, partial_content)
    }

    fn next_section<T>(&mut self) -> Result<T>
    where
        T: FromBytes + KnownLayout,
    {
        let type_name = type_name::<T>();
        let length = size_of::<T>();

        log::trace!(
            "Reading {} @ {:#X} (abs = {:#X})",
            type_name,
            self.position,
            self.position_absolute()
        );

        let result = T::read_from_prefix(&self.data[self.position..])
            .map(|(v, _)| v)
            .map_err(|_| Error::InvalidSection { type_name, length })?;

        self.skip(length)?;

        Ok(result)
    }
}

/// This trait should be implemented by any time that should be parseable via [RszStream::next_field()].
pub trait ParseField {
    /// The main entrypoint for parsing a field using this trait.
    ///
    /// Implementations should handle arrays and any initial alignments in this method.
    fn parse<T: RszStream>(&self, data: &mut T, partial_content: &Content) -> Result<Field>;

    /// Parses the actual field represented by the type implementing this trait.
    ///
    /// Implementations should only handle parsing the field's underlying type, and should rely on
    /// [ParseField::parse()] for any alignment or special case handling (such as arrays).
    fn parse_value<T: RszStream>(&self, data: &mut T, partial_content: &Content) -> Result<Value>;
}

impl ParseField for FieldLayout<'_> {
    fn parse<T: RszStream>(&self, data: &mut T, partial_content: &Content) -> Result<Field> {
        log::debug!(
            "Parsing field \"{}\"; kind = {:?}, type_name = {}, is_array = {}",
            self.name,
            self.kind,
            self.original_type_name,
            self.is_array
        );

        let value = if self.is_array {
            // Arrays always align to 4.
            data.align(4)?;

            let element_count: i32 = data.next_section()?;
            log::debug!(">> Field is array, len = {element_count}");

            let mut elements = Vec::with_signed_capacity(element_count);

            if element_count > 0 {
                // Only align on the inner type's alignment if we have elements.
                data.align(self.align)?;

                for _ in 0..element_count {
                    elements.push(self.parse_value(data, partial_content)?);
                }
            }

            Value::Array(Values(elements))
        } else {
            data.align(self.align)?;
            self.parse_value(data, partial_content)?
        };

        log::debug!("value = {value:?}");

        Ok(Field {
            name: self.name.to_owned(),
            value,
        })
    }

    fn parse_value<T: RszStream>(&self, data: &mut T, partial_content: &Content) -> Result<Value> {
        use FieldKind::*;

        match self.kind {
            Boolean => {
                let byte: u8 = data.next_section()?;
                Ok(Value::Boolean(byte != 0))
            }
            F16 => data.next_section().map(Value::F16),
            F32 => data.next_section().map(Value::F32),
            F64 => data.next_section().map(Value::F64),
            Guid => {
                let bytes: [u8; 16] = data.next_section()?;
                Ok(Value::Guid(Uuid::from_bytes_le(bytes)))
            }
            InstanceRef => {
                let target_index: i32 = data.next_section()?;
                log::debug!(">> Reading instance ref, target_index = {target_index}");

                let target_index = target_index.unsigned_abs() as usize;
                match partial_content.instances.get(target_index) {
                    Some(target) => Ok(Value::Object(target.clone())),
                    None => Err(Error::ObjectNotFound(target_index)),
                }
            }
            S8 => data.next_section().map(Value::S8),
            S16 => data.next_section().map(Value::S16),
            S32 => data.next_section().map(Value::S32),
            S64 => data.next_section().map(Value::S64),
            String => Ok(Value::String(read_bound_string(data)?)),
            U8 => data.next_section().map(Value::U8),
            U16 => data.next_section().map(Value::U16),
            U32 => data.next_section().map(Value::U32),
            U64 => data.next_section().map(Value::U64),
            Vec3 => data.next_section().map(Value::Vec3),
            Mat4 => data.next_section().map(Value::Mat4),
            Data => data.next_section().map(Value::Data),
            UserData => {
                let index: u32 = data.next_section()?;
                let Some(value) = partial_content.interned_strings.get(&index) else {
                    panic!("Could not find interned string where index = {index}");
                };

                Ok(Value::String(value.to_owned()))
            }
            other => Err(Error::UnimplementedFieldType(other)),
        }
    }
}

/// Reads a signed 32-bit length followed by a character sequence from the [`RszStream`]. A `null`
/// byte will terminate the string, even if `len` bytes have not been read.
fn read_bound_string<T: RszStream>(data: &mut T) -> Result<String> {
    let len: i32 = data.next_section()?;
    log::debug!(">> Reading string, len = {len}");

    read_string(data, Some(len.unsigned_abs() as usize))
}

/// Reads a string from the [`RszStream`], terminating on the first `null` byte.
fn read_string<T: RszStream>(data: &mut T, size_hint: Option<usize>) -> Result<String> {
    let mut bytes: Vec<u16> = Vec::with_capacity(size_hint.unwrap_or(30));

    loop {
        let byte: u16 = data.next_section()?;

        if byte == 0 {
            break;
        }

        bytes.push(byte);
    }

    Ok(String::from_utf16_lossy(&bytes))
}

trait SignedCapacity {
    fn with_signed_capacity(capacity: i32) -> Self;
    fn reserve_signed(&mut self, additional: i32) -> ();
}

impl<T> SignedCapacity for Vec<T> {
    fn with_signed_capacity(capacity: i32) -> Self {
        Self::with_capacity(capacity.unsigned_abs() as usize)
    }

    fn reserve_signed(&mut self, additional: i32) -> () {
        self.reserve(additional.unsigned_abs() as usize)
    }
}

impl<K, V> SignedCapacity for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn with_signed_capacity(capacity: i32) -> Self {
        Self::with_capacity(capacity.unsigned_abs() as usize)
    }

    fn reserve_signed(&mut self, additional: i32) -> () {
        self.reserve(additional.unsigned_abs() as usize)
    }
}
