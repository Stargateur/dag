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

  let roots = roots(graph, &parents);
  eprintln!("Validation results:");
  let root = if let [root] = &roots[..] {
    eprintln!(" - Found root: {}", ShortUuid::from_uuid(root));
    *root
  } else {
    eprintln!("Validation failed: expected 1 root, found {}", roots.len());
    return Err(());
  };

  let levels = levels(graph, root);
  let deepths = deepths(&levels);
  let max_deepth = deepths.values().copied().max().unwrap_or(0);
  let average_deepth = deepths.values().copied().sum::<usize>() as f64 / deepths.len() as f64;

  let average_width = average_width_without_root(&levels);
  if have_only_one_path(graph, root) {
    eprintln!(" - Graph have only one path to each node");
  } else {
    eprintln!("Validation failed: graph contains multiple paths to some nodes");
    return Err(());
  }
  eprintln!(
    " - Average childs per node with child: {:.2} (expected average {:.2})",
    average_childs, cfg.child_mean
  );
  eprintln!(" - Max deepth expect {} + 1 <= {}", max_deepth, cfg.deepth);
  eprintln!(" - Average deepth {:.2}", average_deepth);
  eprintln!(
    " - Average width without root level: {:.2} (expected average {:.2})",
    average_width, cfg.width_mean
  );

  Ok(())
}

// could be more simple if we assume root, but this way is more general
fn deepths(levels: &Vec<Vec<Uuid>>) -> HashMap<Uuid, usize> {
  levels
    .iter()
    .enumerate()
    .flat_map(|(i, level)| level.iter().map(move |&uuid| (uuid, i)))
    .collect()
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

fn average_width_without_root(levels: &Vec<Vec<Uuid>>) -> f64 {
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
