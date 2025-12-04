use rand::{
  SeedableRng,
  rngs::StdRng,
  seq::{IndexedRandom, SliceRandom},
};
use rand_distr::{Distribution, Normal};
use snafu::{ResultExt, Snafu};
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
  let child_dist =
    Normal::new(cfg.child_mean, cfg.child_std).context(RandNormalDistributionSnafu)?;

  let mut graph = Graph::new(cfg.name);

  let root = graph.add_node("Root".to_string(), "I'm the root of all evil");
  let mut todo = vec![root.0];

  for _ in 1..cfg.depth {
    let n = width_dist.sample(&mut rng).round().max(1.0) as usize;

    let mut next_todo = Vec::with_capacity(n);
    let mut i = 0;
    todo.shuffle(&mut rng);
    for node in todo {
      let k = child_dist.sample(&mut rng).round().max(0.0) as usize;

      for _ in 0..k {
        if i >= n {
          break;
        } else {
          i += 1;
        }
        let (uuid, _) = graph.add_node(None, ());
        graph.add_child(node, uuid).unwrap();
        next_todo.push(uuid);
      }
    }

    todo = next_todo;
  }

  Ok(graph)
}
