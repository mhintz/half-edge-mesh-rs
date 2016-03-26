use std;

use cgmath::Point;
use cgmath::EuclideanVector;
use cgmath::Vector;

use defs::*;

use half_edge_mesh::ptr::{Ptr, EdgePtr, EdgeRc, VertRc};
use half_edge_mesh::iterators::*;

// TODO: Better way of figuring out when to run compute_attrs
#[derive(Debug)]
pub struct Face {
  pub edge: EdgePtr,
  pub normal: Vec3,
  pub center: Pt,
  pub id: u32,
}

impl Face {
  pub fn empty(id: u32) -> Face {
    Face {
      id: id,
      edge: EdgePtr::empty(),
      // Are these sensible defaults?
      // Are these values even necessary?
      normal: Vec3::unit_z(),
      center: Pt::origin(),
    }
  }

  // Face connected to an existing edge
  pub fn with_edge(id: u32, edge: EdgePtr) -> Face {
    Face {
      id: id,
      edge: edge,
      normal: Vec3::unit_z(),
      center: Pt::origin(),
    }
  }

  pub fn take_edge(&mut self, edge: EdgePtr) { self.edge = edge; }

  pub fn set_edge(&mut self, edge: & EdgePtr) { self.edge = edge.clone(); }

  pub fn set_edge_rc(&mut self, edge: & EdgeRc) { self.edge = Ptr::new(edge); }

  pub fn is_valid(& self) -> bool { self.edge.is_valid() }

  pub fn get_edge(& self) -> Option<EdgeRc> { self.edge.upgrade() }

  pub fn num_vertices(& self) -> usize { self.adjacent_verts().count() }

  // Note: this only works when edges and verts are properly connected
  // So wait for the right time during initialization to run this
  // TODO: Decide what to do here with a degenerate face
  pub fn compute_attrs(&mut self) {
    let mut center = Pt::origin();
    let mut count: f32 = 0.0;

    let vert_list: Vec<VertRc> = self.adjacent_verts().to_ptr_vec();

    debug_assert!(vert_list.len() == 3, "should have 3 adjacent vertices");

    for vert in vert_list.iter() {
      let pos = vert.borrow().get_pos();
      center.x += pos.x;
      center.y += pos.y;
      center.z += pos.z;
      count += 1.0;
    }

    // Average position of the corner points
    self.center = center / count;

    let vert_a = vert_list[0].borrow().get_pos();
    let s1 = vert_list[1].borrow().get_pos() - vert_a;
    let s2 = vert_list[2].borrow().get_pos() - vert_a;
    self.normal = s1.cross(s2).normalize();
  }

  /// Iterates over the vertices which make up the face in *counterclockwise* order
  pub fn adjacent_verts(& self) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator::new(self.edge.clone())
  }

  /// Iterates over the edges which make up the face in *counterclockwise* order
  pub fn adjacent_edges(& self) -> FaceAdjacentEdgeIterator {
    FaceAdjacentEdgeIterator::new(self.edge.clone())
  }

  /// Iterates over the faces adjacent to this face in *counterclockwise* order
  pub fn adjacent_faces(& self) -> FaceAdjacentFaceIterator {
    FaceAdjacentFaceIterator::new(self.edge.clone())
  }

  pub fn distance_to(& self, point: & Pt) -> f32 {
    (point - self.center).length()
  }

  pub fn directed_distance_to(& self, point: & Pt) -> f32 {
    (point - self.center).dot(self.normal)
  }

  pub fn can_see(& self, point: & Pt) -> bool {
    self.directed_distance_to(point) > 0.0000001 // Small epsilon to handle floating-point errors
  }
}

impl PartialEq<Face> for Face {
  fn eq(& self, other: & Face) -> bool { self.id == other.id }
}

impl Eq for Face {}

impl std::hash::Hash for Face {
  fn hash<H>(& self, state: &mut H) where H: std::hash::Hasher {
    state.write_u32(self.id);
    state.finish();
  }
}
