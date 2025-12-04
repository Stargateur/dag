use std::collections::{
  HashMap,
  HashSet,
  VecDeque,
};

use short_uuid::ShortUuid;
use uuid::Uuid;

use crate::{
  generator::Config,
  graph::AcyclicGraph,
};

pub fn validator(graph: &AcyclicGraph, cfg: &Config) -> Result<(), ()> {
  let childs_count: usize = graph.nodes().values().map(|node| node.childs().len()).sum();

  let nodes_with_child_count = graph
    .nodes()
    .values()
    .filter(|node| !node.childs().is_empty())
    .count();

  let average_childs = childs_count as f64 / nodes_with_child_count as f64;

  let parents = graph.parents();
  let max_deepth = max_deepth(graph, &parents);

  let roots = roots(graph, &parents);
  eprintln!("Validation results:");
  let root = if let [root] = &roots[..] {
    eprintln!(" - Found root: {}", ShortUuid::from_uuid(root));
    *root
  } else {
    eprintln!("Validation failed: expected 1 root, found {}", roots.len());
    return Err(());
  };

  let average_width = average_width_without_root(graph, root);
  if have_only_one_path(graph, root) {
    eprintln!(" - Graph have only one path to each node");
  } else {
    eprintln!("Validation failed: graph contains multiple paths to some nodes");
    return Err(());
  }
  eprintln!(
    " - Average childs per node with child: {:.2} (expected {:.2})",
    average_childs, cfg.child_mean
  );
  eprintln!(" - Max deepth expect {} + 1 == {}", max_deepth, cfg.deepth);
  eprintln!(
    " - Average width without root level: {:.2} (max {:.2})",
    average_width, cfg.width_mean
  );

  Ok(())
}

// could be more simple if we assume root, but this way is more general
fn deepths(graph: &AcyclicGraph, parents: &HashMap<Uuid, HashSet<Uuid>>) -> HashMap<Uuid, usize> {
  let mut max_deepth = HashMap::new();
  let mut queue = HashSet::new();

  for (uuid, node) in graph.nodes().iter() {
    if node.childs().is_empty() {
      max_deepth.insert(*uuid, 0);
      queue.insert(*uuid);
    }
  }

  let mut current_deepth = 1;
  while !queue.is_empty() {
    let mut next_queue = HashSet::new();

    for uuid in &queue {
      if let Some(parents) = parents.get(uuid) {
        for parent in parents {
          max_deepth
            .entry(*parent)
            .and_modify(|deepth| *deepth = (*deepth).max(current_deepth))
            .or_insert(current_deepth);
          next_queue.insert(*parent);
        }
      }
    }

    current_deepth += 1;
    std::mem::swap(&mut queue, &mut next_queue);
  }

  max_deepth
}

fn max_deepth(graph: &AcyclicGraph, parents: &HashMap<Uuid, HashSet<Uuid>>) -> usize {
  deepths(graph, parents).values().copied().max().unwrap_or(0)
}

// level existance mean root exist
fn levels(graph: &AcyclicGraph, root: Uuid) -> Vec<Vec<Uuid>> {
  let mut levels = Vec::new();
  let mut current_level = vec![root];

  while !current_level.is_empty() {
    levels.push(current_level.clone());
    let mut next_level = Vec::new();

    for uuid in &current_level {
      if let Some(node) = graph.nodes().get(uuid) {
        for child in node.childs() {
          next_level.push(*child);
        }
      }
    }

    std::mem::swap(&mut current_level, &mut next_level);
  }

  levels
}

fn average_width_without_root(graph: &AcyclicGraph, root: Uuid) -> f64 {
  let levels = levels(graph, root);
  let total_width: usize = levels.iter().skip(1).map(|level| level.len()).sum();
  total_width as f64 / (levels.len() - 1) as f64
}

fn roots(graph: &AcyclicGraph, parents: &HashMap<Uuid, HashSet<Uuid>>) -> Vec<Uuid> {
  graph
    .nodes()
    .keys()
    .filter(|uuid| !parents.contains_key(uuid))
    .copied()
    .collect()
}

// BFS to check if each node is reachable by only one path
fn have_only_one_path(graph: &AcyclicGraph, root: Uuid) -> bool {
  let mut visited = HashSet::new();
  let mut queue = VecDeque::new();
  queue.push_back(root);
  visited.insert(root);
  while let Some(current) = queue.pop_front() {
    if let Some(node) = graph.get_node(current) {
      for &child in node.childs() {
        if visited.insert(child) {
          queue.push_back(child);
        } else {
          return false;
        }
      }
    }
  }

  true
}
