mod generator;
mod graph;
mod validator;

use std::num::NonZeroUsize;

use clap::Parser;
use rand::Rng;

#[derive(Parser, Debug)]
pub struct Args {
  #[arg(long, default_value_t = NonZeroUsize::new(5).unwrap())]
  #[arg(alias = "profondeur_max")]
  deepth: NonZeroUsize,

  #[arg(long, default_value_t = 10.0)]
  #[arg(alias = "largeur_moyenne")]
  width_mean: f64,

  #[arg(long, default_value_t = 0.5)]
  width_std_dev: f64,

  #[arg(long, default_value_t = 3.0)]
  #[arg(alias = "connexions_moyennes")]
  child_mean: f64,

  #[arg(long, default_value_t = 1.0)]
  #[arg(alias = "ecart_type_connexions")]
  child_std_dev: f64,

  #[arg(long, default_value = "mermaid")]
  format: Format,

  #[arg(long)]
  seed: Option<u64>,

  #[arg(long)]
  name: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Format {
  Dot,
  Mermaid,
  Both,
}

fn main() {
  let args = Args::parse();
  let seed = args.seed.unwrap_or_else(|| rand::rng().random());

  let config = generator::Config {
    deepth: args.deepth.into(),
    width_mean: args.width_mean,
    width_std_dev: args.child_std_dev,
    child_mean: args.child_mean,
    child_std_dev: args.child_std_dev,
    seed,
    name: args.name,
  };

  let graph = generator::generate(&config).unwrap();

  match args.format {
    Format::Dot => print!("{}", graph.dot()),
    Format::Mermaid => print!("{}", graph.mermaid()),
    Format::Both => {
      print!("{}", graph.dot());
      print!("{}", graph.mermaid());
    }
  }

  match validator::validator(&graph, &config) {
    Ok(_) => eprintln!("OK"),
    Err(_) => eprintln!("FAIL"),
  }

  eprintln!("Seed used: {seed}");
}
