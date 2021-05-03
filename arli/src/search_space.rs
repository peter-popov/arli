use crate::graph::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

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
  resolved: HashMap<N, State<W, N>>,
}

impl<W: Weight, N: Identifier> SearchSpace<W, N> {
  pub fn new() -> Self {
    SearchSpace {
      pq: BinaryHeap::new(),
      resolved: HashMap::new(),
    }
  }

  pub fn min(&self) -> Option<(N, W)> {
    self.pq.peek().map(|state| (state.id, state.cost))
  }

  pub fn init(&mut self, node: N) {
    self.resolve(node, node, Default::default());
  }

  pub fn init_with_cost(&mut self, node: N, cost: W) {
    self.resolve(node, node, cost);
  }

  pub fn unwind(&self, node: N) -> Vec<N> {
    let mut result: Vec<N> = Vec::new();

    let mut current_node = node;
    loop {
      if let Some(state) = self.resolved.get(&current_node) {
        result.push(current_node);
        if current_node == state.id {
          break;
        }
        current_node = state.id;
      } else {
        break;
      }
    }
    result
  }

  pub fn update<G>(&mut self, graph: G) -> bool
  where
    G: Copy + Weighted<Weight = W> + IntoNeighbors<Forward, NodeId = N>,
  {
    if let Some(State { cost, id }) = self.pq.pop() {
      //println!("PQ: {} @ {}", id, cost);

      if let Some(resolved) = self.resolved.get(&id) {
        if cost > resolved.cost {
          //println!("Drop: {},  {} > {}", id, cost, resolved.cost);
          return true;
        }
      }

      for target_id in neighbors_forward(graph, id) {
        let path_cost: W = cost + graph.transition_weight(id, target_id);
        if let Some(target_state) = self.resolved.get(&target_id) {
          if path_cost >= target_state.cost {
            continue;
          }
        }

        //println!("Relax: ({} -> {}) @ {}", id, target_id, path_cost);

        self.resolve(target_id, id, path_cost);
      }

      return true;
    }
    false
  }

  fn resolve(&mut self, node: N, parent_node: N, path_cost: W) {
    self.resolved.insert(
      node,
      State {
        cost: path_cost,
        id: parent_node,
      },
    );
    self.pq.push(State {
      cost: path_cost,
      id: node,
    });
  }
}

#[cfg(test)]
mod tests {
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

    assert!(search_space.update(weighted_graph));
    assert!(search_space.update(weighted_graph));
    assert!(search_space.update(weighted_graph));
    assert!(search_space.update(weighted_graph));
    assert!(search_space.update(weighted_graph));

    assert_ne!(search_space.update(weighted_graph), true);
  }

  // #[test]
  // fn test_backward_update() {
  //   let mut graph = InMemoryGraph::<u32>::new();

  //   let n1 = graph.new_node(1);
  //   let n2 = graph.new_node(2);
  //   let n3 = graph.new_node(3);
  //   let n4 = graph.new_node(4);
  //   let n5 = graph.new_node(5);

  //   graph
  //     .add_edge(n1, n2)
  //     .add_edge(n2, n3)
  //     .add_edge(n3, n4)
  //     .add_edge(n4, n5)
  //     .add_edge(n4, n2)
  //     .add_edge(n3, n5);

  //   let weighted_graph =
  //     to_reverse_graph(
  //       &graph,
  //       |from: &u32, to: &u32| if to > from { to - from + 1 } else { from - to },
  //     );

  //   let mut search_space = SearchSpace::<u32, u32>::new();

  //   search_space.init(n5);

  //   assert!(search_space.update(weighted_graph));
  //   assert!(search_space.update(weighted_graph));
  //   assert!(search_space.update(weighted_graph));
  //   assert!(search_space.update(weighted_graph));
  //   assert!(search_space.update(weighted_graph));

  //   assert_ne!(search_space.update(weighted_graph), true);
  // }
}
