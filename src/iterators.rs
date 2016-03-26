use std::rc::Rc;
use std::cell::RefCell;

use half_edge_mesh::edge::Edge;
use half_edge_mesh::ptr::{
  Ptr,
  EdgePtr, EdgeRc,
  VertPtr, VertRc,
  FacePtr, /*FaceRc,*/
};

// ToPtrVec
// TODO: rename this, since it's not exactly a vec of "Ptr",
// and that's potentially confusing

// A trait for converting an interator of Ptr<T>
// into a vector of Rc<RefCell<T>>
pub trait ToPtrVec<T> where Self: Iterator<Item=Ptr<T>> {
  fn to_ptr_vec(self) -> Vec<Rc<RefCell<T>>>;
}

// Implement the trait for all iterators over Ptr<T> (all the iterators here)
impl<I, T> ToPtrVec<T> for I where I: Iterator<Item=Ptr<T>> {
  fn to_ptr_vec(self) -> Vec<Rc<RefCell<T>>> {
    self.filter_map(|v| v.upgrade()).collect()
  }
}

// EdgeIterators

enum TwiceIterState {
  First, Second, Done
}

pub struct EdgeAdjacentVertIterator<'a> {
  state: TwiceIterState,
  start: &'a Edge,
}

impl<'a> EdgeAdjacentVertIterator<'a> {
  pub fn new(target: & Edge) -> EdgeAdjacentVertIterator {
    EdgeAdjacentVertIterator {
      state: TwiceIterState::First,
      start: target,
    }
  }
}

impl<'a> Iterator for EdgeAdjacentVertIterator<'a> {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {
    // edge.origin, edge.next.origin
    match self.state {
      TwiceIterState::First => {
        self.state = TwiceIterState::Second;
        Some(self.start.origin.clone())
      },
      TwiceIterState::Second => {
        self.state = TwiceIterState::Done;
        self.start.next.upgrade()
          .map(|next_rc| next_rc.borrow().origin.clone())
      },
      TwiceIterState::Done => None,
    }
  }
}

pub struct EdgeAdjacentEdgeIterator {
  vert_iter_1: Option<VertAdjacentEdgeIterator>,
  vert_iter_2: Option<VertAdjacentEdgeIterator>,
  state: DualIterState,
}

// Implementation here is borrowed from std::iter::Chain
#[derive(Clone)]
enum DualIterState {
  // both iterators running
  Both,
  // only first running
  First,
  // only second running
  Second,
  // neither works
  // (this doesn't exist on the chain iterator,
  // because both must be valid iterators,
  // but it can exist here, in case the weak pointers fail to upgrade)
  Neither
}

impl EdgeAdjacentEdgeIterator {
  pub fn new(target: & Edge) -> EdgeAdjacentEdgeIterator {
    let iter_1_opt: Option<VertAdjacentEdgeIterator> = target.origin.upgrade()
      .map(|vert_ptr: VertRc| vert_ptr.borrow().adjacent_edges());

    let iter_2_opt: Option<VertAdjacentEdgeIterator> = target.next.upgrade()
      .and_then(|edge_next: EdgeRc| edge_next.borrow().origin.upgrade())
      .map(|vert_ptr: VertRc| vert_ptr.borrow().adjacent_edges());

    // Flexible w.r.t. whether either pointer upgrade fails.
    // is this expected behavior? I'm not positive
    let state = match (iter_1_opt.as_ref(), iter_2_opt.as_ref()) {
      (Some(_), Some(_)) => DualIterState::Both,
      (Some(_), None) => DualIterState::First,
      (None, Some(_)) => DualIterState::Second,
      (None, None) => DualIterState::Neither
    }; // <-- because this match is an assignment statement, this semicolon is essential

    EdgeAdjacentEdgeIterator {
      state: state,
      vert_iter_1: iter_1_opt,
      vert_iter_2: iter_2_opt
    }
  }
}

impl Iterator for EdgeAdjacentEdgeIterator {
  type Item = EdgePtr;

  fn next(&mut self) -> Option<EdgePtr> {
    // edge.origin.adjacent_edges(), edge.next.origin.adjacent_edges()
    // unwraps are only OK here because of the nature of the constructor
    match self.state {
      DualIterState::Both => {
        match self.vert_iter_1.as_mut().unwrap().next() {
          // val @ *pattern* binds val to the entire object, doesn't destructure the Option
          val @ Some(..) => val,
          None => {
            self.state = DualIterState::Second;
            self.vert_iter_2.as_mut().unwrap().next()
          }
        }
      },
      DualIterState::First => self.vert_iter_1.as_mut().unwrap().next(),
      DualIterState::Second => self.vert_iter_2.as_mut().unwrap().next(),
      DualIterState::Neither => None,
    }
  }
}

pub struct EdgeAdjacentFaceIterator<'a> {
  start: &'a Edge,
  state: TwiceIterState
}

impl<'a> EdgeAdjacentFaceIterator<'a> {
  pub fn new(target: &'a Edge) -> EdgeAdjacentFaceIterator<'a> {
    EdgeAdjacentFaceIterator {
      start: target,
      state: TwiceIterState::First
    }
  }
}

impl<'a> Iterator for EdgeAdjacentFaceIterator<'a> {
  type Item = FacePtr;

  fn next(&mut self) -> Option<FacePtr> {
    // edge.face, edge.pair.face
    match self.state {
      TwiceIterState::First => {
        self.state = TwiceIterState::Second;
        Some(self.start.face.clone())
      },
      TwiceIterState::Second => {
        self.state = TwiceIterState::Done;
        self.start.pair.upgrade()
          .map(|pair_rc: EdgeRc| pair_rc.borrow().face.clone())
      },
      TwiceIterState::Done => None
    }
  }
}

// VertIterators

pub struct VertAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl VertAdjacentVertIterator {
  pub fn new(edge: EdgePtr) -> VertAdjacentVertIterator {
    VertAdjacentVertIterator {
      start: edge,
      current: None,
    }
  }
}

impl Iterator for VertAdjacentVertIterator {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {
    // edge.pair.origin
    // edge -> edge.pair.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .and_then(|cur_rc: EdgeRc| cur_rc.borrow().pair.upgrade())
        .and_then(|pair_rc: EdgeRc| {
          let next_weak: EdgePtr = pair_rc.borrow().next.clone();
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                self.current = Some(next_weak);
                Some(pair_rc.borrow().origin.clone())
              } else { None }
            });
        }),
      None => self.start.upgrade()
        .and_then(|cur_rc: EdgeRc| cur_rc.borrow().pair.upgrade())
        .map(|pair_rc: EdgeRc| {
          self.current = Some(self.start.clone());
          pair_rc.borrow().origin.clone()
        }),
    }
  }
}

pub struct VertAdjacentEdgeIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl VertAdjacentEdgeIterator {
  pub fn new(edge: EdgePtr) -> VertAdjacentEdgeIterator {
    VertAdjacentEdgeIterator {
      start: edge,
      current: None
    }
  }
}

impl Iterator for VertAdjacentEdgeIterator {
  type Item = EdgePtr;

  fn next(&mut self) -> Option<EdgePtr> {
    // edge
    // edge -> edge.pair.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .and_then(|cur_rc: EdgeRc| cur_rc.borrow().pair.upgrade())
        .map(|pair_rc: EdgeRc| pair_rc.borrow().next.clone())
        .and_then(|next_weak: EdgePtr| {
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                self.current = Some(next_weak.clone());
                Some(next_weak)
              } else { None }
            });
        }),
      None => self.start.upgrade()
        .map(|_: EdgeRc| {
          self.current = Some(self.start.clone());
          self.start.clone()
        }),
    }
  }
}

pub struct VertAdjacentFaceIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl VertAdjacentFaceIterator {
  pub fn new(edge: EdgePtr) -> VertAdjacentFaceIterator {
    VertAdjacentFaceIterator {
      start: edge,
      current: None,
    }
  }
}

impl Iterator for VertAdjacentFaceIterator {
  type Item = FacePtr;

  fn next(&mut self) -> Option<FacePtr> {
    // edge.face
    // edge -> edge.pair.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .and_then(|cur_rc: EdgeRc| cur_rc.borrow().pair.upgrade())
        .map(|pair_rc: EdgeRc| pair_rc.borrow().next.clone())
        .and_then(|next_weak: EdgePtr| {
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                self.current = Some(next_weak);
                Some(next_rc.borrow().face.clone())
              } else { None }
            })
        }),
      None => self.start.upgrade()
        .map(|cur_rc: EdgeRc| {
          self.current = Some(self.start.clone());
          cur_rc.borrow().face.clone()
        }),
    }
  }
}

// FaceIterators

pub struct FaceAdjacentVertIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl FaceAdjacentVertIterator {
  pub fn new(edge: EdgePtr) -> FaceAdjacentVertIterator {
    FaceAdjacentVertIterator {
      start: edge,
      current: None,
    }
  }
}

impl Iterator for FaceAdjacentVertIterator {
  type Item = VertPtr;

  fn next(&mut self) -> Option<VertPtr> {
    // edge.origin
    // edge -> edge.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .map(|cur_rc: EdgeRc| cur_rc.borrow().next.clone())
        .and_then(|next_weak: EdgePtr| {
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                self.current = Some(next_weak);
                Some(next_rc.borrow().origin.clone())
              } else { None }
            });
        }),
      None => self.start.upgrade()
        .map(|cur_rc: EdgeRc| {
          self.current = Some(self.start.clone());
          cur_rc.borrow().origin.clone()
        }),
    }
  }
}

pub struct FaceAdjacentEdgeIterator {
  start: EdgePtr,
  current: Option<EdgePtr>
}

impl FaceAdjacentEdgeIterator {
  pub fn new(edge: EdgePtr) -> FaceAdjacentEdgeIterator {
    FaceAdjacentEdgeIterator {
      start: edge,
      current: None
    }
  }
}

impl Iterator for FaceAdjacentEdgeIterator {
  type Item = EdgePtr;

  fn next(&mut self) -> Option<EdgePtr> {
    // edge
    // edge -> edge.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .map(|cur_rc: EdgeRc| cur_rc.borrow().next.clone())
        .and_then(|next_weak: EdgePtr| {
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                self.current = Some(next_weak.clone());
                Some(next_weak)
              } else { None }
            });
        }),
      None => {
        self.current = Some(self.start.clone());
        Some(self.start.clone())
      },
    }
  }
}

pub struct FaceAdjacentFaceIterator {
  start: EdgePtr,
  current: Option<EdgePtr>,
}

impl FaceAdjacentFaceIterator {
  pub fn new(edge: EdgePtr) -> FaceAdjacentFaceIterator {
    FaceAdjacentFaceIterator {
      start: edge,
      current: None
    }
  }
}

impl Iterator for FaceAdjacentFaceIterator {
  type Item = FacePtr;

  fn next(&mut self) -> Option<FacePtr> {
    // edge.pair.face
    // edge -> edge.next
    match self.current.clone() {
      Some(cur_weak) => cur_weak.upgrade()
        .map(|cur_rc: EdgeRc| cur_rc.borrow().next.clone())
        .and_then(|next_weak: EdgePtr| {
          return Ptr::merge_upgrade(& next_weak, & self.start)
            .and_then(|(next_rc, start_rc)| {
              if next_rc != start_rc {
                next_rc.borrow().pair.upgrade()
                  .map(|pair_rc| {
                    self.current = Some(next_weak);
                    pair_rc.borrow().face.clone()
                  })
              } else { None }
            });
        }),
      None => self.start.upgrade()
        .and_then(|edge_rc: EdgeRc| edge_rc.borrow().pair.upgrade())
        .map(|pair_rc: EdgeRc| {
          self.current = Some(self.start.clone());
          pair_rc.borrow().face.clone()
        }),
    }
  }
}
