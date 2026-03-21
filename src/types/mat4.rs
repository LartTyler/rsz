use std::fmt::{Debug, Formatter};
use zerocopy::{FromBytes, KnownLayout};

#[derive(KnownLayout, FromBytes)]
#[repr(C, packed)]
pub struct Mat4([Mat4Element; 4]);

impl Debug for Mat4 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Mat4(")?;
        self.0.fmt(f)?;
        f.write_str(")")
    }
}

impl Clone for Mat4 {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Debug, FromBytes, KnownLayout, Clone)]
#[repr(C, packed)]
pub struct Mat4Element {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}
