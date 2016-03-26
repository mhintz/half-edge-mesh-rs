use std;

use defs::*;

use half_edge_mesh::ptr::{Ptr, EdgePtr, EdgeRc};
use half_edge_mesh::iterators::*;

#[derive(Debug)]
pub struct Vert {
  pub edge: EdgePtr,
  pub pos: Pt,
  pub id: u32,
}

impl Vert {
  // All structure of the mesh revolves around vertex positions and their connectivity.
  // (Faces are just an abstraction). All vertices must therefore have a concrete position.
  pub fn empty(id: u32, pos: Pt) -> Vert {
    Vert {
      id: id,
      edge: EdgePtr::empty(),
      pos: pos,
    }
  }

  // Vertex connected to an existing edge
  pub fn with_edge(id: u32, pos: Pt, edge: EdgePtr) -> Vert {
    Vert {
      id: id,
      edge: edge,
      pos: pos,
    }
  }

  pub fn take_edge(&mut self, edge: EdgePtr) { self.edge = edge; }

  pub fn set_edge(&mut self, edge: & EdgePtr) { self.edge = edge.clone(); }

  pub fn set_edge_rc(&mut self, edge: & EdgeRc) { self.edge = Ptr::new(edge); }

  pub fn move_to(&mut self, pos: Pt) { self.pos = pos; }

  pub fn get_pos(& self) -> Pt { self.pos }

  pub fn is_valid(& self) -> bool { self.edge.is_valid() }

  pub fn get_edge(& self) -> Option<EdgeRc> { self.edge.upgrade() }

  /// Important: Iterates over the vertices connected to a vertex in *clockwise* order
  pub fn adjacent_verts(& self) -> VertAdjacentVertIterator {
    VertAdjacentVertIterator::new(self.edge.clone())
  }

  /// Important: Iterates over the edges connected to a vertex in *clockwise* order
  pub fn adjacent_edges(& self) -> VertAdjacentEdgeIterator {
    VertAdjacentEdgeIterator::new(self.edge.clone())
  }

  /// Important: Iterates over the faces connected to a vertex in *clockwise* order
  pub fn adjacent_faces(& self) -> VertAdjacentFaceIterator {
    VertAdjacentFaceIterator::new(self.edge.clone())
  }
}

impl PartialEq<Vert> for Vert {
  fn eq(& self, other: & Vert) -> bool { self.id == other.id }
}

impl Eq for Vert {}

impl std::hash::Hash for Vert {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}
