use crate::serde::{deserialize_hex, deserialize_hex_map};
use serde::Deserialize;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, Error>;

pub type TypeId = usize;
type InnerMap<'a> = HashMap<TypeId, Layout<'a>>;

/// Layout maps are usually loaded from RSZ JSON files, and contain a mapping of type IDs to their
/// definitions. These are required in order to properly understand the contents of an RSZ file.
#[derive(Debug, Deserialize)]
pub struct LayoutMap<'a>(#[serde(borrow, deserialize_with = "deserialize_hex_map")] InnerMap<'a>);

impl<'a> LayoutMap<'a> {
    /// Creates a new instance of `LayoutMap`. This is useful if you intend to parse the RSZ file
    /// yourself.
    pub fn new(inner: InnerMap<'a>) -> Self {
        Self(inner)
    }

    /// Creates a new instance of `LayoutMap` from an RSZ JSON file.
    pub fn from_json(input: &'a str) -> Result<Self> {
        serde_json::from_str(input).map_err(From::from)
    }

    /// Retrieves a type layout by ID.
    pub fn get_layout(&self, id: TypeId) -> Option<&Layout<'a>> {
        self.0.get(&id)
    }
}

/// Each entry in a layout file defines a type layout that can be encoded in an RSZ file.
#[derive(Debug, Deserialize)]
pub struct Layout<'a> {
    /// If the property defined by this layout contains any fields, this will be a collection of
    /// layout information for those fields.
    pub fields: Vec<FieldLayout<'a>>,

    /// The original in-engine property name.
    pub name: &'a str,

    /// A CRC value for correctness checking.
    #[serde(deserialize_with = "deserialize_hex")]
    pub crc: usize,
}

/// Defines a field contained within a property layout.
#[derive(Debug, Deserialize)]
pub struct FieldLayout<'a> {
    /// The field's byte alignment.
    ///
    /// When parsing, the input stream _must_ be aligned by this value in order to find the field's
    /// bytes.
    pub align: usize,

    /// The number of bytes the field occupies in the file.
    ///
    /// This is mostly provided for debugging and sanity checking, as the types we deserialize into
    /// are already sized.
    pub size: usize,

    /// Indicates that this field is an array of values.
    #[serde(rename = "array")]
    pub is_array: bool,

    /// The display name of the field.
    pub name: &'a str,

    /// The purpose of the field is not known.
    #[serde(rename = "native")]
    pub is_native: bool,

    /// The original in-engine name of the field's type.
    #[serde(rename = "original_type")]
    pub original_type_name: &'a str,

    /// The type contained in the field. For arrays, this is the type of each element.
    #[serde(rename = "type")]
    pub kind: FieldKind,
}

/// Represents a type within the type system.
///
/// Implementation of some types is on an as-needed basis, so many things are not currently
/// supported. If you need a type that hasn't been implemented yet, please open an issue (or a pull
/// request if you want to be my favorite person ever).
///
/// Types that are not implemented yet are commented to indicate that.
#[derive(Debug, Deserialize, Copy, Clone)]
pub enum FieldKind {
    #[serde(rename = "Bool")]
    Boolean,

    /// A 16-bit float
    F16,

    /// A 32-bit float
    F32,

    /// A 64-bit float
    F64,

    /// A UUID
    Guid,

    /// An index that references one of the instances contained in the document.
    #[serde(rename = "Object")]
    InstanceRef,

    /// An 8-bit signed integer
    S8,

    /// A 16-bit signed integer
    S16,

    /// A 32-bit signed integer
    S32,

    /// A 64-bit signed integer
    S64,

    /// A string
    String,

    /// An 8-bit unsigned integer
    U8,

    /// A 16-bit unsigned integer
    U16,

    /// A 32-bit unsigned integer
    U32,

    /// A 64-bit unsigned integer
    U64,

    /// Currently unknown.
    Data,

    /// An index referencing an interned string from the userdata section of the document.
    UserData,

    // --- All items below this line are not fully supported ---
    /// Not implemented.
    AABB,
    /// Not implemented.
    Capsule,
    /// Not implemented.
    Color,
    /// Not implemented.
    Cylinder,
    /// Not implemented.
    DateTime,
    /// Not implemented.
    Float2,
    /// Not implemented.
    Float3,
    /// Not implemented.
    Float4,
    /// Not implemented.
    Frustum,
    /// Not implemented.
    GameObjectRef,
    /// Not implemented.
    Half4,
    /// Not implemented.
    Int2,
    /// Not implemented.
    Int3,
    /// Not implemented.
    Int4,
    /// Not implemented.
    KeyFrame,
    /// Not implemented.
    Mat4,
    /// Not implemented.
    OBB,
    /// Not implemented.
    Plane,
    /// Not implemented.
    Point,
    /// Not implemented.
    Position,
    /// Not implemented.
    Quaternion,
    /// Not implemented.
    Range,
    /// Not implemented.
    RangeI,
    /// Not implemented.
    Rect,
    /// Not implemented.
    Resource,
    /// Not implemented.
    RuntimeType,
    /// Not implemented.
    Size,
    /// Not implemented.
    Sphere,
    /// Not implemented.
    Struct,
    /// Not implemented.
    Uint2,
    /// Not implemented.
    Vec2,
    /// Not implemented.
    Vec3,
    /// Not implemented.
    Vec4,
    /// Not implemented.
    C8,
    /// Not implemented.
    Uint3,
    /// Not implemented.
    Triangle,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("deserialize failed: {0}")]
    Deserialize(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
