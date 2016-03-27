use std::rc::{Rc, Weak};
use std::cell::RefCell;

use edge::Edge;
use vert::Vert;
use face::Face;

pub type RcRef<T> = Rc<RefCell<T>>;

pub type EdgePtr = Ptr<Edge>;
pub type EdgeRc = RcRef<Edge>;
pub type VertPtr = Ptr<Vert>;
pub type VertRc = RcRef<Vert>;
pub type FacePtr = Ptr<Face>;
pub type FaceRc = RcRef<Face>;


// Ptr is essentially a wrapper around Option<Weak<RefCell<T>>>,
// a.k.a. a nullable ref-counted pointer with interior mutability
// This abstraction is used to get around Rust's
// validity, borrowing, and ownership rules, especially when constructing or
// extending the half-edge mesh.

#[derive(Debug)]
pub struct Ptr<T> {
  val: Option<Weak<RefCell<T>>>
}

impl<T> Ptr<T> {
  // Taken by value, so it moves the value out.
  // Use this for constructing brand new objects.
  // Returns an Rc<RefCell<T>>, not a Ptr<T>,
  // don't get em mixed up
  pub fn new_rc(val: T) -> RcRef<T> { Rc::new(RefCell::new(val)) }

  // Taken by reference to an existing object. Creates a Ptr
  pub fn new(val: & RcRef<T>) -> Ptr<T> {
    Ptr { val: Some(Rc::downgrade(val)) }
  }

  pub fn empty() -> Ptr<T> {
    Ptr { val: None }
  }

  pub fn merge_upgrade(weak_a: & Ptr<T>, weak_b: & Ptr<T>) -> Option<(RcRef<T>, RcRef<T>)> {
    match (weak_a.upgrade(), weak_b.upgrade()) {
      (Some(strong_a), Some(strong_b)) => Some((strong_a, strong_b)),
      _ => None
    }
  }

  // Is it a bad idea to call upgrade() in this function? Too expensive?
  pub fn is_valid(& self) -> bool { self.val.is_some() && self.upgrade().is_some() }

  pub fn upgrade(& self) -> Option<Rc<RefCell<T>>> {
    self.val.as_ref().and_then(|v| v.upgrade())
  }

  pub fn as_ref(& self) -> Option<& Weak<RefCell<T>>> {
    self.val.as_ref()
  }
}

impl<T> Clone for Ptr<T> {
  fn clone(& self) -> Self {
    Ptr { val: self.val.clone() }
  }
}
