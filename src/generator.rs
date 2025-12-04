use rand::{
  SeedableRng,
  rngs::StdRng,
  seq::IndexedRandom,
};
use rand_distr::{
  Distribution,
  Normal,
};
use snafu::{
  ResultExt,
  Snafu,
};
use uuid::Uuid;

use crate::graph::Graph;

pub struct Config {
  pub name: String,
  pub depth: usize,
  pub width_mean: f64,
  pub width_std: f64,
  pub child_mean: f64,
  pub child_std: f64,
  pub seed: Option<u64>,
}

#[derive(Snafu, Debug)]
pub enum Error {
  RandNormalDistribution { source: rand_distr::NormalError },
}

pub fn generate(cfg: Config) -> Result<Graph, Error> {
  let mut rng = if let Some(seed) = cfg.seed {
    StdRng::seed_from_u64(seed)
  } else {
    StdRng::from_rng(&mut rand::rng())
  };

  let width_dist =
    Normal::new(cfg.width_mean, cfg.width_std).context(RandNormalDistributionSnafu)?;
  let conn_dist =
    Normal::new(cfg.child_mean, cfg.child_std).context(RandNormalDistributionSnafu)?;

  let mut graph = Graph::new(cfg.name);
  let mut levels: Vec<Vec<Uuid>> = vec![];

  let root = graph.add_node("Root".to_string(), "I'm the root of all evil");
  levels.push(vec![root.0]);

  for _ in 1..cfg.depth {
    let n = width_dist.sample(&mut rng).round().max(1.0) as usize;
    let mut nodes = Vec::with_capacity(n);

    for _ in 0..n {
      let node = graph.add_node(None, ());
      nodes.push(node.0);
    }

    levels.push(nodes);
  }

  for level in 0..cfg.depth - 1 {
    let potencial_child: Vec<Uuid> = levels[level + 1].iter().copied().collect();

    for &node in &levels[level] {
      let k = conn_dist.sample(&mut rng).round().max(0.0) as usize;

      for &child in potencial_child.choose_multiple(&mut rng, k) {
        graph.add_child(node, child).unwrap();
      }
    }
  }

  Ok(graph)
}
