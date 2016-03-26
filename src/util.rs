use std::collections::HashMap;

use half_edge_mesh::ptr::{Ptr, EdgeRc};
use half_edge_mesh::half_edge_mesh::HalfEdgeMesh;

fn merge_tuple_opt<A, B>(o: (Option<A>, Option<B>)) -> Option<(A, B)> {
  match o {
    (Some(a), Some(b)) => Some((a, b)),
    _ => None
  }
}

fn vert_ab_key(e: & EdgeRc) -> Option<(u32, u32)> {
  let id_origin = e.borrow().origin.upgrade().map(|o| o.borrow().id);
  let id_next_origin = e.borrow().next.upgrade().and_then(|n| n.borrow().origin.upgrade()).map(|o| o.borrow().id);
  merge_tuple_opt((id_origin, id_next_origin))
}

fn vert_ba_key(e: & EdgeRc) -> Option<(u32, u32)> { vert_ab_key(e).map(|tuple| (tuple.1, tuple.0)) }

// Takes what is assumed to be a fully-connected mesh, with no
// pair links, and establishes pair links between adjacent edges
pub fn connect_pairs(mesh: &mut HalfEdgeMesh) -> Result<(), &'static str> {
  // Two-stage algorithm: first collect all edge A -> B relationships,
  // Then go through and look for edges that are B -> A
  let mut edge_hash: HashMap<(u32, u32), & EdgeRc> = HashMap::new();

  for ref edge in mesh.edges.values() {
    // The types returned by match arms must be the same,
    // hence the braces and semicolon used in the first branch
    match vert_ab_key(edge) {
      Some(key) => { edge_hash.insert(key, edge); },
      // This happens if one of the mesh edges doesn't have a valid .origin or .next.origin pointer
      None => { return Err("Could not hash all mesh edges"); }
    }
  }

  for ref edge in mesh.edges.values() {
    // This if statement should skip half the edges, because two
    // edge pairs are set each time it's true
    if !edge.borrow().pair.is_valid() {
      if let Some(key) = vert_ba_key(edge) {
        match edge_hash.get(& key) {
          Some(pair_edge) => {
            // if one edge A -> B matches another edge B -> A, the edges are adjacent
            edge.borrow_mut().take_pair(Ptr::new(pair_edge));
            pair_edge.borrow_mut().take_pair(Ptr::new(edge));
          },
          None => { /* Happens when mesh is not closed */
            return Err("Could not find pair edge");
          }
        }
      } else {
        // Theoretically this shouldn't ever happen
        // because of the early return in the previous match block
        return Err("Could not find reverse hash for mesh edge");
      }
    }
  }

  return Ok(());
}

// Checks if edge pair connections are all valid
pub fn are_edge_pairs_valid(mesh: & HalfEdgeMesh) -> Result<(), &'static str> {
  let mut edge_hash: HashMap<(u32, u32), & EdgeRc> = HashMap::new();

  for ref edge in mesh.edges.values() {
    // The types returned by match arms must be the same,
    // hence the braces and semicolon used in the first branch
    match vert_ab_key(edge) {
      Some(key) => { edge_hash.insert(key, edge); },
      // This happens if one of the mesh edges doesn't have a valid .origin or .next.origin pointer
      None => { return Err("Could not hash all mesh edges"); }
    }
  }

  for ref edge in mesh.edges.values() {
    match vert_ba_key(edge) {
      Some(key) => {
        match edge_hash.get(& key) {
          Some(ref pair) => {
            if (edge.borrow().pair.upgrade().as_ref() != Some(pair)) ||
               (pair.borrow().pair.upgrade().as_ref() != Some(edge)) {
                return Err("Pairs don't match");
            }
          },
          None => { return Err("Could not find a pair edge"); }
        }
      },
      None => { return Err("Could not find reverse hash for mesh edge"); }
    }
  }

  return Ok(());
}
