use petname::Generator;
use rand::{
  SeedableRng,
  rngs::StdRng,
  seq::SliceRandom,
};
use rand_distr::{
  Distribution,
  Normal,
};
use snafu::{
  ResultExt,
  Snafu,
};

use crate::graph::AcyclicGraph;

pub struct Config {
  pub name: Option<String>,
  pub deepth: usize,
  pub width_mean: f64,
  pub width_std: f64,
  pub child_mean: f64,
  pub child_std: f64,
  pub seed: u64,
}

#[derive(Snafu, Debug)]
pub enum Error {
  RandNormalDistribution { source: rand_distr::NormalError },
}

/// This will generate a sinple graph that look like a family tree
pub fn generate(cfg: &Config) -> Result<AcyclicGraph, Error> {
  let mut rng = StdRng::seed_from_u64(cfg.seed);

  let petnames = petname::Petnames::default();

  let width_dist =
    Normal::new(cfg.width_mean, cfg.width_std).context(RandNormalDistributionSnafu)?;
  let child_dist =
    Normal::new(cfg.child_mean, cfg.child_std).context(RandNormalDistributionSnafu)?;

  let name = cfg.name.as_ref().cloned().unwrap_or_else(|| {
    petnames
      .generate(&mut rng, 2, "_")
      .unwrap_or_else(|| "output".to_string())
  });
  let mut graph = AcyclicGraph::new(name);

  let root = graph.add_node_with_rng("Root".to_string(), "I'm the root of all evil", &mut rng);
  let mut current = vec![root.0];
  let mut next = Vec::new();

  for _ in 1..cfg.deepth {
    let n = width_dist.sample(&mut rng).round().max(1.0) as usize;

    next.clear();
    let mut i = 0;
    current.shuffle(&mut rng);
    'outer: for &node in &current {
      let k = child_dist.sample(&mut rng).round().max(0.0) as usize;

      for _ in 0..k {
        // limit total width
        if i >= n {
          break 'outer;
        } else {
          i += 1;
        }
        let name = petnames.generate(&mut rng, 1, "_");
        let (uuid, _) = graph.add_node_with_rng(name, (), &mut rng);
        graph.add_child(node, uuid).unwrap();
        next.push(uuid);
      }
    }

    std::mem::swap(&mut current, &mut next);
  }

  Ok(graph)
}
