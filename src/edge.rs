use std;

use half_edge_mesh::ptr::{Ptr, EdgePtr, VertPtr, FacePtr, EdgeRc, VertRc, FaceRc};
use half_edge_mesh::iterators::*;

#[derive(Debug)]
pub struct Edge {
  pub next: EdgePtr,
  pub pair: EdgePtr,
  pub origin: VertPtr,
  pub face: FacePtr,
  pub id: u32,
}

// TODO: change the name of set_*_rc to just set_*, and change the current set_* to set_*_ptr
// because set_*_rc is used way more than set_* at the moment.
impl Edge {
  pub fn empty(id: u32) -> Edge {
    Edge {
      id: id,
      next: EdgePtr::empty(),
      pair: EdgePtr::empty(),
      origin: VertPtr::empty(),
      face: FacePtr::empty(),
    }
  }

  pub fn with_origin(id: u32, origin: VertPtr) -> Edge {
    Edge {
      id: id,
      next: EdgePtr::empty(),
      pair: EdgePtr::empty(),
      origin: origin,
      face: FacePtr::empty(),
    }
  }

  pub fn take_next(&mut self, next: EdgePtr) { self.next = next; }

  pub fn set_next(&mut self, next: & EdgePtr) { self.next = next.clone(); }

  pub fn set_next_rc(&mut self, next: & EdgeRc) { self.next = Ptr::new(next); }

  pub fn take_pair(&mut self, pair: EdgePtr) { self.pair = pair; }

  pub fn set_pair(&mut self, pair: & EdgePtr) { self.pair = pair.clone(); }

  pub fn set_pair_rc(&mut self, pair: & EdgeRc) { self.pair = Ptr::new(pair); }

  pub fn take_origin(&mut self, origin: VertPtr) { self.origin = origin; }

  pub fn set_origin(&mut self, origin: & VertPtr) { self.origin = origin.clone(); }

  pub fn set_origin_rc(&mut self, origin: & VertRc) { self.origin = Ptr::new(origin); }

  pub fn set_face(&mut self, face: & FacePtr) { self.face = face.clone(); }

  pub fn take_face(&mut self, face: FacePtr) { self.face = face; }

  pub fn set_face_rc(&mut self, face: & FaceRc) { self.face = Ptr::new(face); }

  // The tests in this function are in order of "subjective likeliness of being invalid"
  pub fn is_valid(& self) -> bool { self.pair.is_valid() && self.face.is_valid() && self.origin.is_valid() && self.next.is_valid() }

  pub fn get_next(& self) -> Option<EdgeRc> { self.next.upgrade() }

  pub fn get_pair(& self) -> Option<EdgeRc> { self.pair.upgrade() }

  pub fn get_origin(& self) -> Option<VertRc> { self.origin.upgrade() }

  pub fn get_face(& self) -> Option<FaceRc> { self.face.upgrade() }

  pub fn get_next_next(& self) -> Option<EdgeRc> { self.get_next().and_then(|n| n.borrow().get_next()) }

  pub fn get_next_pair(& self) -> Option<EdgeRc> { self.get_next().and_then(|n| n.borrow().get_pair()) }

  pub fn get_target(& self) -> Option<VertRc> { self.get_next().and_then(|n| n.borrow().get_origin()) }

  pub fn get_pair_face(& self) -> Option<FaceRc> { self.get_pair().and_then(|p| p.borrow().get_face()) }

  /// Yields edge.origin, then edge.next.origin
  /// Gives you first the source of the half-edge, and then its target
  pub fn adjacent_verts<'a> (&'a self) -> EdgeAdjacentVertIterator<'a> {
    EdgeAdjacentVertIterator::new(self)
  }

  /// Gives you the edges connected to the source of the half-edge first (in *clockwise* order)
  /// and then the edges connected to the target of the half-edge (also *clockwise* order)
  pub fn adjacent_edges(& self) -> EdgeAdjacentEdgeIterator {
    EdgeAdjacentEdgeIterator::new(self)
  }

  /// Yields edge.face, then edge.pair.face
  /// Gives you the "left" face to the half edge, and then the "right" face
  /// Note that the "right" face is not connected to this edge, but to its pair
  pub fn adjacent_faces<'a>(&'a self) -> EdgeAdjacentFaceIterator<'a> {
    EdgeAdjacentFaceIterator::new(self)
  }
}

impl PartialEq<Edge> for Edge {
  fn eq(& self, other: & Edge) -> bool { self.id == other.id }
}

impl Eq for Edge {}

impl std::hash::Hash for Edge {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}
