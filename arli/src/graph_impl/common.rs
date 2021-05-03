use crate::graph::NodesExtension;
use serde::{Deserialize, Serialize};
use std::cell::Cell;

// Node id
pub type Idx = u32;

/// Iterator over elements of the target array defined by RangeRef
pub struct RefIterator<'a, T> {
  items: &'a Vec<T>,
  next: usize,
  last: usize,
}

/// A  [start, end) range of elements in the contiguous array.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct RangeRef(pub Idx, pub Idx);

impl<'a, T: Copy> Iterator for RefIterator<'a, T> {
  type Item = T;

  fn next(&mut self) -> Option<Self::Item> {
    if self.next == self.last {
      return None;
    }
    let index = self.next;
    if self.next < self.last {
      self.next += 1;
    }
    if self.next > self.last {
      self.next -= 1;
    }
    Some(self.items[index])
  }
}

impl<'a, T: Copy> RefIterator<'a, T> {
  pub fn new(items: &'a Vec<T>, first: Idx, last: Idx) -> Self {
    RefIterator {
      items: items,
      next: first as usize,
      last: last as usize,
    }
  }

  pub fn from_range(items: &'a Vec<T>, range: &RangeRef) -> Self {
    RefIterator::new(items, range.0, range.1)
  }
}

pub struct MoreNodes {
  max_id: Idx,
  next: Cell<Idx>
}

impl MoreNodes {
  pub fn new(max_id: Idx) -> Self {
    Self {
      max_id: max_id,
      next: Cell::new(max_id + 1)
    }
  }
}

impl NodesExtension<Idx> for MoreNodes {
  fn new_node_id(&self) -> Option<Idx> {
    Some(self.next.replace(1 + self.next.get()))
  }

  fn contains(&self, id: Idx) -> bool {
    id > self.max_id && id < self.next.get()
  }
}
