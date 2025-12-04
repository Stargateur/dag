use std::collections::{HashMap, HashSet};

use crate::{generator::Config, graph::Graph};

fn validator(graph: &Graph, cfg: Config) -> Result<(), ()> {
  let childs_count: usize = graph
    .nodes()
    .iter()
    .map(|(_, node)| node.childs().len())
    .sum();

  let nodes_count = graph.nodes().len();

  let average_childs = childs_count as f64 / nodes_count as f64;

  Ok(())
}

fn max_deepth(graph: &Graph) -> usize {
  let mut max_deepth = HashMap::new();
  let mut todo : HashSet<_> = graph.nodes().keys().cloned().collect();

  for (uuid, node) in graph.nodes().iter() {
      if node.childs().is_empty() {
        max_deepth.insert(*uuid, 0);
        todo.remove(uuid);
      }
  }

  while !todo.is_empty() {
    todo.retain(|uuid|{
      let node = graph.nodes().get(uuid).unwrap();

      if node.childs().iter().all(|child| max_deepth.contains_key(child)) {
        let depth = node
          .childs()
          .iter()
          .map(|child| max_deepth.get(child).unwrap())
          .max()
          .unwrap()
          + 1;
        max_deepth.insert(*uuid, depth);
        false
      } else {
        true
      }
    });
  }

  max_deepth.values().copied().max().unwrap_or(0)
}
