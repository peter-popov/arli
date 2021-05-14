use crate::graph::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::collections::hash_map::Entry;

#[derive(Copy, Clone, Eq, PartialEq)]
struct State<W: Weight, N: Identifier> {
  cost: W,
  id: N,
}

impl<W: Weight, N: Identifier> Ord for State<W, N> {
  fn cmp(&self, other: &Self) -> Ordering {
    // Sorted desc
    other.cost.cmp(&self.cost)
    //todo: resolve ties with Node ID; .then_with(|| self.id.cmp(&other.id))
  }
}

// `PartialOrd` needs to be implemented as well.
impl<W: Weight, N: Identifier> PartialOrd for State<W, N> {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

pub struct SearchSpace<W: Weight, N: Identifier> {
  pq: BinaryHeap<State<W, N>>,
  labels: HashMap<N, (W, N, bool)>,
}

impl<W: Weight, N: Identifier> SearchSpace<W, N> {
  pub fn new() -> Self {
    SearchSpace {
      pq: BinaryHeap::new(),
      labels: HashMap::new(),
    }
  }

  pub fn min(&self) -> Option<(N, W)> {
    self.pq.peek().map(|state| (state.id, state.cost))
  }

  pub fn init(&mut self, node: N) {
    self.relax(node, node, &Default::default());
  }

  pub fn init_with_cost(&mut self, node: N, cost: W) {
    self.relax(node, node, &cost);
  }

  pub fn unwind(&self, node: N) -> Vec<N> {
    let mut result: Vec<N> = Vec::new();

    let mut current_node = node;
    loop {
      if let Some((_, parent, _)) = self.labels.get(&current_node) {
        result.push(current_node);
        if current_node == *parent {
          break;
        }
        current_node = *parent;
      } else {
        break;
      }
    }
    result
  }

  pub fn update<Dir:ForwardOrBackward, G>(&mut self, graph: G) -> bool
  where
    G: Copy + Weighted<Weight = W> + IntoNeighbors<Dir, NodeId = N>,
  {
    loop {
      if let Some(State { cost, id }) = self.pq.pop() {
        //println!("--\nPop: {:?} @ {:?}", id, cost);

        // Need to skip if already settled, since we don't decrease priority of existing elements
        if !self.settle(id) {
            continue;
        }

        //println!("Settled: {:?} @ {:?}", id, cost);

        // Relax
        for target_id in graph.neighbors(id) {
          // TODO: we need a way to swap arguments going to the cost depening on the direction
          // For now the workaround is to use different cost-function for the backward search
          let cost = cost + graph.transition_weight(id, target_id);
          self.relax(target_id, id, &cost);
        }
        return true;
      }
      return false;
    }
  }

  pub fn is_settled(&mut self, node: N) -> Option<W> {
    self.labels.get(&node).filter(|t| t.2).map(|t| t.0)
  }

  fn settle(&mut self, node: N) -> bool {
    if let Entry::Occupied(mut entry) = self.labels.entry(node) {
      if !entry.get_mut().2 {
        entry.get_mut().2 = true;    
        return true;
      }
    }
    return false;
  }

  fn relax(&mut self, node: N, new_parent: N, new_cost: &W) {
    match self.labels.entry(node) {
      Entry::Occupied(mut entry) => {
        let (current_cost, _, is_settled) = entry.get_mut();
        if new_cost < current_cost {
          assert!(!*is_settled);
          self.pq.push(State {cost: *new_cost, id: node});
          entry.insert((*new_cost, new_parent, false));
          //println!("Relax: u({:?} -> {:?}) @ {:?}", new_parent, node, new_cost);
        }
      },
      Entry::Vacant(entry) => {
        self.pq.push(State {cost: *new_cost, id: node});
        entry.insert((*new_cost, new_parent, false));
        //println!("Relax: +({:?} -> {:?}) @ {:?}", new_parent, node, new_cost);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::iter::FromIterator;
  use std::array::IntoIter;

  use super::super::test_utils::graph_from_data_and_edges;
  use super::*;

  #[test]
  fn test_forward_update() {
    let graph = graph_from_data_and_edges(
      vec![1, 2, 3, 4, 5],
      vec![(0, 1), (1, 2), (2, 3), (3, 4), (3, 1), (2, 4)],
    );

    let weighted_graph = (
      &graph,
      |from: &u32, to: &u32| if to > from { to - from + 1 } else { from - to },
    );

    let mut search_space = SearchSpace::<u32, u32>::new();

    search_space.init(0);

    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));

    assert_ne!(search_space.update::<Forward, _>(weighted_graph), true);
  }

  #[test]
  fn test_backward_update() {
    let graph = graph_from_data_and_edges(
      vec![1, 2, 3, 4, 5],
      vec![(0, 1), (1, 2), (2, 3), (3, 4), (3, 1), (2, 4)],
    );

    let weighted_graph =(
        &graph,
        |to: &u32, from: &u32| if to > from { to - from + 1 } else { from - to },
      );

    let mut search_space = SearchSpace::<u32, u32>::new();

    search_space.init(4);

    assert!(search_space.update::<Backward, _>(weighted_graph));
    assert!(search_space.update::<Backward, _>(weighted_graph));
    assert!(search_space.update::<Backward, _>(weighted_graph));
    assert!(search_space.update::<Backward, _>(weighted_graph));
    assert!(search_space.update::<Backward, _>(weighted_graph));

    assert_ne!(search_space.update::<Backward, _>(weighted_graph), true);
  }

  #[test]
  fn test_relaxed_properly_second_time() {
    let graph = graph_from_data_and_edges(
      vec![0, 1, 2, 3, 4],
      vec![(0, 1), (0, 2), (2, 3), (1, 3), (3, 4)],
    );

    let costs = HashMap::<_, _>::from_iter(IntoIter::new([((0, 1), 1), ((0, 2), 50), ((2, 3), 50), ((1, 3), 100), ((3, 4), 2)]));

    let weighted_graph = (
      &graph,
      |from: &u32, to: &u32| *costs.get(&(*from, *to)).unwrap()
    );

    let mut search_space = SearchSpace::<u32, u32>::new();

    search_space.init(0);

    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));
    assert!(search_space.update::<Forward, _>(weighted_graph));

    assert_ne!(search_space.update::<Forward, _>(weighted_graph), true);
  }
}
