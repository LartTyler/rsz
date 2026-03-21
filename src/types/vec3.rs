use zerocopy::{FromBytes, KnownLayout};

#[derive(Debug, KnownLayout, FromBytes, Clone, PartialOrd, PartialEq)]
#[repr(C, packed)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    _dummy: f32,
}
