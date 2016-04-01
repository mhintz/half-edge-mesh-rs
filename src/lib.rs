// TODO: use clippy linter on this code

extern crate cgmath;

pub mod ptr;
pub mod edge;
pub mod vert;
pub mod face;
pub mod iterators;
pub mod mesh;
pub mod util;

pub use self::mesh::HalfEdgeMesh;
pub use self::edge::Edge;
pub use self::vert::Vert;
pub use self::face::Face;

// Export the pointer types too, in case you need them
pub use self::ptr::*;

// Export relevant iterators and traits
pub use self::iterators::ToPtrVec;
