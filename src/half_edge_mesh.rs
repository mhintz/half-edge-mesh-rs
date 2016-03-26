use std::collections::HashMap;
use std::collections::HashSet;

use defs::*;

use half_edge_mesh::edge::Edge;
use half_edge_mesh::vert::Vert;
use half_edge_mesh::face::Face;
use half_edge_mesh::ptr::{Ptr, EdgeRc, VertRc, FaceRc, EdgePtr, VertPtr, FacePtr};
use half_edge_mesh::iterators::ToPtrVec;
use half_edge_mesh::util::*;

/// Half-Edge Mesh data structure
/// While it's possible to create non-triangular faces, this code assumes
/// triangular faces in several locations
// TODO: Better error reporting, using a custom error type
// See also: http://blog.burntsushi.net/rust-error-handling/
// TODO: Better way of updating face-specific data like center and normals
// Probably should do it whenever faces are added or a vertex is modified ?
pub struct HalfEdgeMesh {
  pub edges: HashMap<u32, EdgeRc>,
  pub vertices: HashMap<u32, VertRc>,
  pub faces: HashMap<u32, FaceRc>,
  // Vertex, edge, and face ids are mesh-specific and unique only within a certain mesh
  // Integer overflow is undefined in Rust, but checked in debug builds. I think this means
  // that it's possible to generate the same id twice, after 2^32-1 ids have been made.
  // Try not to make more than 2^32-1 of any one of them, stuff might get messed up.
  cur_edge_id: u32,
  cur_vert_id: u32,
  cur_face_id: u32,
}

impl HalfEdgeMesh {
  pub fn empty() -> HalfEdgeMesh {
    HalfEdgeMesh {
      edges: HashMap::new(),
      vertices: HashMap::new(),
      faces: HashMap::new(),
      cur_edge_id: 0,
      cur_vert_id: 0,
      cur_face_id: 0,
    }
  }

  // A half-edge mesh requires at least a tetrahedron to be valid
  // p1: apex, p2: bottom left front, p3: bottom right front, p4: bottom rear
  pub fn from_tetrahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt) -> HalfEdgeMesh {
    // In progress
    let mut mesh = HalfEdgeMesh::empty();

    let v1 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p1));
    let v2 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p2));
    let v3 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p3));
    let v4 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p4));

    let mut tri;

    tri = mesh.make_triangle(& v1, & v2, & v3);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v2, & v1, & v4);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v3, & v4, & v1);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v4, & v3, & v2);
    mesh.add_triangle(tri);

    mesh.move_verts(vec![v1, v2, v3, v4]);

    report_connect_err(connect_pairs(&mut mesh));

    return mesh;
  }

  // p1: top apex, p2: mid left front, p3: mid right front, p4: mid left back, p5: mid right back, p6: bottom apex
  pub fn from_octahedron_pts(p1: Pt, p2: Pt, p3: Pt, p4: Pt, p5: Pt, p6: Pt) -> HalfEdgeMesh {
    let mut mesh = HalfEdgeMesh::empty();

    let v1 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p1));
    let v2 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p2));
    let v3 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p3));
    let v4 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p4));
    let v5 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p5));
    let v6 = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), p6));

    let mut tri;

    tri = mesh.make_triangle(& v1, & v2, & v3);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v1, & v4, & v2);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v1, & v3, & v5);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v1, & v5, & v4);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v6, & v3, & v2);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v6, & v2, & v4);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v6, & v5, & v3);
    mesh.add_triangle(tri);
    tri = mesh.make_triangle(& v6, & v4, & v5);
    mesh.add_triangle(tri);

    mesh.move_verts(vec![v1, v2, v3, v4, v5, v6]);

    report_connect_err(connect_pairs(&mut mesh));

    return mesh;
  }

  pub fn from_face_vertex_mesh(vertices: & Vec<Pt>, indices: & Vec<Tri>) -> HalfEdgeMesh {
    let mut mesh = HalfEdgeMesh::empty();
    let mut id_map: HashMap<usize, u32> = HashMap::new(); // Maps indices to ids

    for (idx, pos) in vertices.iter().enumerate() {
      let vert = Ptr::new_rc(Vert::empty(mesh.new_vert_id(), pos.clone()));
      id_map.insert(idx, vert.borrow().id);
      mesh.push_vert(vert);
    }

    for tri in indices.iter() {
      let face = Ptr::new_rc(Face::empty(mesh.new_face_id()));
      let mut new_edges: Vec<EdgeRc> = Vec::new();

      for idx in tri {
        match id_map.get(idx) {
          Some(vert_id) => {
            if mesh.vertices.contains_key(vert_id) {
              let new_edge_id = mesh.new_edge_id();
              if let Some(vert) = mesh.vertices.get(vert_id) {
                let edge = Ptr::new_rc(Edge::with_origin(new_edge_id, Ptr::new(vert)));
                edge.borrow_mut().set_face_rc(& face);
                vert.borrow_mut().set_edge_rc(& edge);
                new_edges.push(edge);
              }
            }
          },
          None => (),
        }
      }

      let n_edge_len = new_edges.len();
      for (idx, edge) in new_edges.iter().enumerate() {
        edge.borrow_mut().set_next_rc(& new_edges[(idx + 1) % n_edge_len]);
      }

      if let Some(ref edge) = new_edges.get(0) {
        face.borrow_mut().set_edge_rc(edge);
      }

      for edge in new_edges {
        mesh.push_edge(edge);
      }

      mesh.push_face(face);
    }

    report_connect_err(connect_pairs(&mut mesh));

    return mesh;
  }

  pub fn new_edge_id(&mut self) -> u32 {
    self.cur_edge_id += 1; self.cur_edge_id
  }

  pub fn new_vert_id(&mut self) -> u32 {
    self.cur_vert_id += 1; self.cur_vert_id
  }

  pub fn new_face_id(&mut self) -> u32 {
    self.cur_face_id += 1; self.cur_face_id
  }

  pub fn push_edge(&mut self, edge: EdgeRc) {
    let key = edge.borrow().id;
    self.edges.insert(key, edge);
  }

  pub fn extend_edges(&mut self, edges: & [EdgeRc]) {
    for edge in edges {
      let key = edge.borrow().id;
      self.edges.insert(key, edge.clone());
    }
  }

  pub fn move_edges(&mut self, edges: Vec<EdgeRc>) {
    for edge in edges {
      let key = edge.borrow().id;
      self.edges.insert(key, edge);
    }
  }

  pub fn push_vert(&mut self, vert: VertRc) {
    let key = vert.borrow().id;
    self.vertices.insert(key, vert);
  }

  pub fn extend_verts(&mut self, verts: & [VertRc]) {
    for vert in verts {
      let key = vert.borrow().id;
      self.vertices.insert(key, vert.clone());
    }
  }

  pub fn move_verts(&mut self, verts: Vec<VertRc>) {
    for vert in verts {
      let key = vert.borrow().id;
      self.vertices.insert(key, vert);
    }
  }

  pub fn push_face(&mut self, face: FaceRc) {
    // Ensuring that the attributes are correct before the face gets added here is essential
    face.borrow_mut().compute_attrs();
    let key = face.borrow().id;
    self.faces.insert(key, face);
  }

  pub fn extend_faces(&mut self, faces: & [FaceRc]) {
    for face in faces {
      let key = face.borrow().id;
      self.faces.insert(key, face.clone());
    }
  }

  pub fn move_faces(&mut self, faces: Vec<FaceRc>) {
    for face in faces {
      let key = face.borrow().id;
      self.faces.insert(key, face);
    }
  }

  pub fn add_triangle(&mut self, triangle: (FaceRc, EdgeRc, EdgeRc, EdgeRc)) {
    let mut key: u32;

    key = triangle.0.borrow().id;
    self.faces.insert(key, triangle.0);

    key = triangle.1.borrow().id;
    self.edges.insert(key, triangle.1);

    key = triangle.2.borrow().id;
    self.edges.insert(key, triangle.2);

    key = triangle.3.borrow().id;
    self.edges.insert(key, triangle.3);
  }

  // Takes three Rc<RefCell<Vert>>,
  // creates three edges and one face, and connects them as well as it can
  // Note: since this creates a lone triangle, edge.pair links are
  // still empty after this function
  pub fn make_triangle(&mut self, p1: & VertRc, p2: & VertRc, p3: & VertRc) -> (FaceRc, EdgeRc, EdgeRc, EdgeRc) {
    // Create triangle edges
    let e1 = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& p1)));
    let e2 = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& p2)));
    let e3 = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& p3)));

    // Be sure to set up vertex connectivity with the new edges
    // It doesn't matter which edge a vertex points to,
    // so long as it points back to the vertex
    p1.borrow_mut().take_edge(Ptr::new(& e1));
    p2.borrow_mut().take_edge(Ptr::new(& e2));
    p3.borrow_mut().take_edge(Ptr::new(& e3));

    // Set up edge cycle
    e1.borrow_mut().take_next(Ptr::new(& e2));
    e2.borrow_mut().take_next(Ptr::new(& e3));
    e3.borrow_mut().take_next(Ptr::new(& e1));

    // Create triangle face
    let f1 = Ptr::new_rc(Face::with_edge(self.new_face_id(), Ptr::new(& e1)));

    // Set up face links
    e1.borrow_mut().take_face(Ptr::new(& f1));
    e2.borrow_mut().take_face(Ptr::new(& f1));
    e3.borrow_mut().take_face(Ptr::new(& f1));

    // Now is the right time to run this, since vertices and edges are connected
    f1.borrow_mut().compute_attrs();

    (f1, e1, e2, e3)
  }

  // Checks if two faces are adjacent by looking for a shared edge
  pub fn are_faces_adjacent(& self, face_l: & FaceRc, face_r: & FaceRc) -> bool {
    face_l.borrow().adjacent_edges()
      .any(|edge| {
        edge.upgrade()
          .and_then(|e| e.borrow().pair.upgrade())
          .and_then(|e| e.borrow().face.upgrade())
          .map(|f| f == * face_r) == Some(true)
      })
  }

  pub fn are_face_ptrs_adjacent(& self, face_l: & FacePtr, face_r: & FacePtr) -> bool {
    match Ptr::merge_upgrade(face_l, face_r) {
      Some((l_rc, r_rc)) => self.are_faces_adjacent(& l_rc, & r_rc),
      None => false,
    }
  }

  // Replace a face with three faces, each connected to the new point
  // And one of the face's previous vertices
  // TODO: Make all of these mesh-manipulation functions return a Result<(), &str> to check that manipulation was completed
  pub fn triangulate_face(&mut self, point: Pt, target_face: & FaceRc) {
    // get face edges
    let face_edges = target_face.borrow().adjacent_edges().to_ptr_vec();
    // get face vertexes, assumed to be counter-clockwise
    let face_vertices = target_face.borrow().adjacent_verts().to_ptr_vec();
    let vertices_len = face_vertices.len();

    debug_assert!(face_edges.len() == 3, "should be 3 adjacent edges");
    debug_assert!(vertices_len == 3, "should be 3 adjacent vertices"); // should be 3, or else your faces aren't triangles

    let apex_vert = Ptr::new_rc(Vert::empty(self.new_vert_id(), point));

    // Add the three new faces - one attached to each of the original face's edges,
    // plus two new edges attached to the point
    let mut new_lead_edges: Vec<EdgeRc> = Vec::new();
    let mut new_trail_edges: Vec<EdgeRc> = Vec::new();
    for (i, base_edge) in face_edges.iter().enumerate() {
      // Might not be necessary
      base_edge.borrow_mut().take_origin(Ptr::new(& face_vertices[i]));
      base_edge.borrow().origin.upgrade().map(|o| o.borrow_mut().take_edge(Ptr::new(base_edge)));

      let new_face = Ptr::new_rc(Face::with_edge(self.new_face_id(), Ptr::new(base_edge)));
      let leading_edge = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& face_vertices[(i + 1) % vertices_len])));
      let trailing_edge = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& apex_vert)));

      base_edge.borrow_mut().take_face(Ptr::new(& new_face));
      leading_edge.borrow_mut().take_face(Ptr::new(& new_face));
      trailing_edge.borrow_mut().take_face(Ptr::new(& new_face));

      base_edge.borrow_mut().take_next(Ptr::new(& leading_edge));
      leading_edge.borrow_mut().take_next(Ptr::new(& trailing_edge));
      trailing_edge.borrow_mut().take_next(Ptr::new(base_edge));

      apex_vert.borrow_mut().take_edge(Ptr::new(& trailing_edge));

      new_lead_edges.push(leading_edge.clone());
      new_trail_edges.push(trailing_edge.clone());

      self.push_edge(leading_edge);
      self.push_edge(trailing_edge);
      self.push_face(new_face);
    }

    // This step is pretty crucial
    self.push_vert(apex_vert);

    let trail_edge_len = new_trail_edges.len();

    // Should be 3, or else the faces are not triangular, or not enough edges were created
    debug_assert!(trail_edge_len == 3, "should be 3 new trailing edges");
    debug_assert!(new_lead_edges.len() == 3, "should be 3 new leading edges");

    // Connect pairs
    for (i, leading_edge) in new_lead_edges.iter().enumerate() {
      let trailing_edge = & new_trail_edges[(i + 1) % trail_edge_len];
      leading_edge.borrow_mut().take_pair(Ptr::new(& trailing_edge));
      trailing_edge.borrow_mut().take_pair(Ptr::new(& leading_edge));
    }

    // Remove the face and the edges from the mesh.
    // When the local pointer to this falls out of scope, it should be deallocated
    self.faces.remove(& target_face.borrow().id);
  }

  pub fn triangulate_face_ptr(&mut self, point: Pt, face: & FacePtr) {
    match face.upgrade() {
      Some(face_rc) => self.triangulate_face(point, & face_rc),
      None => (),
    }
  }

  /// Attach a point to a mesh, replacing many faces (used for the convex hull algorithm)
  /// The faces should be a continuously connected group, each adjacent pair of vertices
  /// in the border of this group are connected to the point in a new triangular face.
  /// The programmer is responsible for ensuring that there are no holes in the passed
  /// set of faces. Returns Pointers to the new faces in the result, if successful
  pub fn attach_point_for_faces(&mut self, point: Pt, remove_faces: & Vec<FaceRc>) -> Result<Vec<FaceRc>, &'static str> {
    // collect a set of face ids to be removed, for later reference
    let outgoing_face_ids: HashSet<u32> = remove_faces.iter().map(|f| f.borrow().id).collect();
    let mut horizon_edges: HashMap<u32, EdgeRc> = HashMap::new();
    let mut remove_edges: Vec<u32> = Vec::new();
    let mut remove_verts: Vec<u32> = Vec::new();
    let mut horizon_next_map: HashMap<u32, u32> = HashMap::new();
    let mut iter_edge: Option<EdgeRc> = None;

    // for each face in faces
    for out_face in remove_faces.iter() {
      // iterate over the edges of the face
      for face_edge in out_face.borrow().adjacent_edges().to_ptr_vec() {
        // check if the opposite face bordered by the edge should also be removed (edge.pair.face)
        // any edges which border a face to be removed, should also be removed.
        // Any edges which border a face which won't be removed, are part of the "horizon".
        let remove_edge = face_edge.borrow().pair.upgrade()
          .and_then(|p| p.borrow().face.upgrade())
          .map(|f| outgoing_face_ids.contains(& f.borrow().id))
          .unwrap_or(true); // Remove edges where pointer upgrades don't work

        if remove_edge {
          // Removed edges are saved for later in a vec
          remove_edges.push(face_edge.borrow().id);
        } else {
          // The origin vertex of each horizon edge should have it's edge pointer set to the horizon edge
          // This is important in case the edge pointer was already set to one of the removed edges
          face_edge.borrow().get_origin().map(|o| o.borrow_mut().set_edge_rc(& face_edge));
          // The first horizon edge discovered should be saved as an "iteration" edge
          if iter_edge.is_none() { iter_edge = Some(face_edge.clone()); }
          // Horizon edges are saved for later in a HashMap (id -> edge)
          horizon_edges.insert(face_edge.borrow().id, face_edge.clone());
        }
      }

      // likewise, iterate over the vertices of the face
      // any vertex which is surrounded by only faces to be removed, should also be removed.
      // any vertex which has at least one non-removed face adjacent to it should not be removed.
      // Save the removed vertices in a list, to be dealt with later
      for face_vert in out_face.borrow().adjacent_verts().to_ptr_vec() {
        let remove_vert = face_vert.borrow().adjacent_faces()
          .all(|face_ptr| {
            face_ptr.upgrade()
              .map(|f| outgoing_face_ids.contains(& f.borrow().id))
              .unwrap_or(true)
          });

        if remove_vert {
          remove_verts.push(face_vert.borrow().id);
        }
      }
    }

    // If no iteration edge was saved, then no horizon edges were found and the faces list is invalid.
    if iter_edge.is_none() { return Err("No horizon edges found"); }

    // iterate over the horizon edges
    for h_edge in horizon_edges.values() {
      // Iterate over the edges at the target end of each horizon edge (edge.next.origin.adjacent_edges())
      if let Some(target_vert) = h_edge.borrow().get_target() {
        // find the next horizon edge connected to it.
        // If the edge is actually a horizon edge (i.e. it is actually adjacent to a face which won't be removed),
        // then this adjacent horizon edge must exist.
        for adj_edge in target_vert.borrow().adjacent_edges() {
          if let Some(adj_edge_rc) = adj_edge.upgrade() {
            let adj_id = adj_edge_rc.borrow().id;
            if horizon_edges.contains_key(& adj_id) {
              // Save the correspondences between each horizon edge and the next one
              horizon_next_map.insert(h_edge.borrow().id, adj_id);
              break;
            }
          }
        }
      }
    }

    // check the horizon edge next correspondences: each next value should itself have a next value.
    // In addition, each key value should have some other key's next pointing to it.
    // Because of the way the hashmap is constructed, no edge will point to itself (good!)
    let horizon_next_keys: HashSet<u32> = horizon_next_map.keys().map(|e| e.clone()).collect();
    let horizon_next_values: HashSet<u32> = horizon_next_map.values().map(|e| e.clone()).collect();

    // Test that the set of keys and values are equal, i.e. keys are a subset of values and vice versa
    if horizon_next_keys != horizon_next_values { return Err("Horizon is malformed - it does not form a connected loop"); }

    // Create a vec which iterates over the horizon edges, with adjacent horizon edges adjacent in the vec.
    // This will be used twice later
    let start_edge = iter_edge.unwrap();
    let start_id = start_edge.borrow().id;
    let mut iter_id = start_id;
    let mut horizon_vec: Vec<EdgeRc> = Vec::new();
    // Note: after testing the invariant above that keys and values are equal,
    // we know this loop will finish
    loop {
      horizon_vec.push(self.edges[& iter_id].clone());
      iter_id = horizon_next_map[& iter_id];
      if iter_id == start_id { break; }
    }

    // Remove the faces, the edges, and the vertices that were marked for removal
    // Do this after all other data structures have been set up, because a valid mesh is required
    // for some steps, for example finding a horizon edge's next edge
    for out_face in remove_faces.iter() {
      self.faces.remove(& out_face.borrow().id);
    }

    for out_vert_id in remove_verts.iter() {
      self.vertices.remove(out_vert_id);
    }

    for out_edge_id in remove_edges.iter() {
      self.edges.remove(out_edge_id);
    }

    // create a new vertex for the point
    let apex_vert = Ptr::new_rc(Vert::empty(self.new_vert_id(), point));

    // Going to iterate twice through the ordered list of horizon edges created earlier
    // And set up new mesh entities and their linkage
    let horizon_len = horizon_vec.len();

    let mut return_faces: Vec<FaceRc> = Vec::new();

    // the iterating edge is the 'base edge'
    for (idx, base_edge) in horizon_vec.iter().enumerate() {
      // the iterating edge's next edge is edges[(i + 1) % edges.len()]
      let next_edge = & horizon_vec[(idx + 1) % horizon_len];
      if let Some(next_origin) = next_edge.borrow().origin.upgrade() {
        // create a new face, connected to the base edge
        let new_face = Ptr::new_rc(Face::with_edge(self.new_face_id(), Ptr::new(base_edge)));
        // create two new edges, one leading and one trailing.
        // The leading edge connects to the next horizon edge's origin vertex
        let new_leading = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& next_origin)));
        // the trailing edge connects to the new vertex
        let new_trailing = Ptr::new_rc(Edge::with_origin(self.new_edge_id(), Ptr::new(& apex_vert)));
        // connect the new vertex to the trailing edge (this is repeated many times but is necessary for mesh validity)
        apex_vert.borrow_mut().set_edge_rc(& new_trailing);
        // connect next ptrs: the horizon edge to the leading edge, the leading to the trailing, and the trailing to the horizon
        base_edge.borrow_mut().set_next_rc(& new_leading);
        new_leading.borrow_mut().set_next_rc(& new_trailing);
        new_trailing.borrow_mut().set_next_rc(& base_edge);
        // connect all three to the face
        base_edge.borrow_mut().set_face_rc(& new_face);
        new_leading.borrow_mut().set_face_rc(& new_face);
        new_trailing.borrow_mut().set_face_rc(& new_face);
        // move the two new edges into the mesh
        self.push_edge(new_leading);
        self.push_edge(new_trailing);
        // move the face into the mesh
        return_faces.push(new_face.clone());
        self.push_face(new_face);
      } else {
        return Err("Could not set up horizon faces correctly");
      }
    }

    // move the point vertex into the mesh
    self.push_vert(apex_vert);

    // iterate over the horizon edges again.
    for (idx, base_edge) in horizon_vec.iter().enumerate() {
      // Connect pairs: edge.next to (edge + 1).next.next and vice versa
      let next_edge = & horizon_vec[(idx + 1) % horizon_len];
      if let (Some(next_rc), Some(pair_rc)) = (base_edge.borrow().get_next(), next_edge.borrow().get_next_next()) {
        next_rc.borrow_mut().set_pair_rc(& pair_rc);
        pair_rc.borrow_mut().set_pair_rc(& next_rc);
      } else {
        return Err("Could not connect pair edges");
      }
    }

    return Ok(return_faces);
  }

  pub fn attach_point_for_face_ptrs(&mut self, point: Pt, faces: & Vec<FacePtr>) -> Result<Vec<FaceRc>, &'static str> {
    let face_ptrs = faces.iter().filter_map(|f| f.upgrade()).collect::<Vec<FaceRc>>();
    self.attach_point_for_faces(point, & face_ptrs)
  }

  // This function should only work if the vertex has exactly three adjacent edges.
  // Therefore, it has three adjacent faces.
  // The vertices connected to those edges form a new face, and the faces and edges connected
  // to the removed vertex are also removed
  pub fn remove_vert(&mut self, vert: & VertRc) -> Result<(), &'static str> {
    let vert_b = vert.borrow();
    let mut edges = vert_b.adjacent_edges().to_ptr_vec(); // get e for e in v.edges
    // Edges are iterated in clockwise order, but we need counter-clockwise order
    // to establish correct .next links
    edges.reverse();

    // Must have 3 edges, so that the surrounding faces can be combined to a triangle
    if edges.len() != 3 { return Err("Vertex must have exactly 3 connecting edges"); }

    let new_face = Ptr::new_rc(Face::empty(self.new_face_id())); // n_f

    for (idx, edge) in edges.iter().enumerate() {
      let edge_b = edge.borrow();
      edge_b.next.upgrade()
        .map(|next: EdgeRc| {
          let mut next_bm = next.borrow_mut();
          next_bm.set_face_rc(& new_face); // e.n.f = n_f
          next_bm.set_next(& edges[(idx + 1) % edges.len()].borrow().next); // e.n.n = (e + 1).n
          new_face.borrow_mut().set_edge_rc(& next); // n_f.e = e.n
          next_bm.origin.upgrade()
            .map(|o: VertRc| o.borrow_mut().set_edge_rc(& next)); // e.n.o.e = e.n
        });

      edge_b.pair.upgrade()
        .map(|p: EdgeRc| self.edges.remove(& p.borrow().id)); // del e.p
      self.edges.remove(& edge_b.id); // del e
    }

    self.push_face(new_face); // add n_f

    for face in vert_b.adjacent_faces() {
      face.upgrade().map(|f: FaceRc| self.faces.remove(& f.borrow().id)); // del f for f in v.faces
    }

    self.vertices.remove(& vert_b.id); // del v

    return Ok(());
  }

  pub fn remove_vert_ptr(&mut self, point: & VertPtr) -> Result<(), &'static str> {
    match point.upgrade() {
      Some(point_rc) => self.remove_vert(& point_rc),
      None => Err("Provided pointer was invalid"),
    }
  }

  // flips an edge between two faces so that the faces are each split by
  // the other diagonal of the parallelogram they form.
  pub fn flip_edge(&mut self, edge: & EdgeRc) {
    unimplemented!();
  }

  pub fn flip_edge_ptr(&mut self, edge: & EdgePtr) {
    match edge.upgrade() {
      Some(edge_rc) => self.flip_edge(& edge_rc),
      None => (),
    }
  }

  // Inserts a vertex at the position, specified by tval, along edge.origin -> edge.next.origin
  // The edge's two neighboring faces are each split into two faces.
  // All four new faces include the new vertex
  pub fn split_edge(&mut self, edge: & EdgeRc, tval: f32) {
    unimplemented!();
  }

  pub fn split_edge_rc(&mut self, edge: & EdgePtr, tval: f32) {
    match edge.upgrade() {
      Some(edge_rc) => self.split_edge(& edge_rc, tval),
      None => (),
    }
  }
}

fn report_connect_err(res: Result<(), &str>) {
  match res {
    Err(e) => println!("Error connecting mesh pairs! Mesh is not valid! {}", e),
    _ => {},
  }
}
