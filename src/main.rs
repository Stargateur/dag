mod generator;
mod graph;

use std::num::NonZeroUsize;

use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
  #[arg(long, default_value_t = NonZeroUsize::new(3).unwrap())]
  depth_max: NonZeroUsize,

  #[arg(long, default_value_t = 10.0)]
  width_mean: f64,

  #[arg(long, default_value_t = 5.0)]
  width_std: f64,

  #[arg(long, default_value_t = 3.0)]
  child_mean: f64,

  #[arg(long, default_value_t = 2.0)]
  child_dev: f64,

  #[arg(long, default_value = "dot")]
  format: Format,

  #[arg(long)]
  seed: Option<u64>,

  #[arg(long, default_value = "output")]
  name: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
  Dot,
  Mermaid,
}

fn main() {
  let args = Args::parse();

  let config = generator::Config {
    depth: args.depth_max.into(),
    width_mean: args.width_mean,
    width_std: args.child_dev,
    child_mean: args.child_mean,
    child_std: args.child_dev,
    seed: args.seed,
    name: args.name,
  };

  let graph = generator::generate(config).unwrap();

  match args.format {
    Format::Dot => println!("{}", graph.dot()),
    Format::Mermaid => println!("{}", graph.mermaid()),
  }

  // match validate::validate_graph(&graph) {
  //     Ok(_) => println!("Validation : OK (DAG)"),
  //     Err(err) => println!("Validation : ÉCHEC -> {}", err),
  // }
}
