use std::{
  collections::{
    HashMap,
    HashSet,
    VecDeque,
  },
  fmt::{
    self,
    Display,
    Formatter,
  },
};

use itertools::Itertools;
use rand::Rng;
use short_uuid::ShortUuid;
use snafu::Snafu;
use uuid::Uuid;

#[derive(Debug, Snafu, PartialEq)]
pub enum Error {
  #[snafu(display("Cycle detected {src} => {dst}"))]
  Cycle { src: Uuid, dst: Uuid },
  #[snafu(display("UUID not found: {uuid}"))]
  UuidNotFound { uuid: Uuid },
  #[snafu(display("Child already exist from {parent} to {child}"))]
  ChildAlreadyExist { parent: Uuid, child: Uuid },
}

#[derive(Debug, Clone)]
pub struct AcyclicGraph {
  name: String,
  nodes: HashMap<Uuid, Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
  name: Option<String>,
  data: NodeData,
  childs: HashSet<Uuid>,
}

impl Node {
  pub fn new(name: impl Into<Option<String>>, data: NodeData) -> Self {
    Self {
      name: name.into(),
      data,
      childs: HashSet::new(),
    }
  }

  pub fn childs(&self) -> &HashSet<Uuid> {
    &self.childs
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
  Number(u64),
  Text(String),
  None,
}

impl From<u64> for NodeData {
  fn from(n: u64) -> Self {
    NodeData::Number(n)
  }
}

impl From<String> for NodeData {
  fn from(s: String) -> Self {
    NodeData::Text(s)
  }
}

impl From<&str> for NodeData {
  fn from(s: &str) -> Self {
    NodeData::Text(s.into())
  }
}

impl From<()> for NodeData {
  fn from(_: ()) -> Self {
    NodeData::None
  }
}

impl AcyclicGraph {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      nodes: HashMap::new(),
    }
  }

  pub fn nodes(&self) -> &HashMap<Uuid, Node> {
    &self.nodes
  }

  fn add_node_uuid(
    &mut self, uuid: Uuid, name: impl Into<Option<String>>, data: impl Into<NodeData>,
  ) -> (Uuid, &Node) {
    match self.nodes.entry(uuid) {
      std::collections::hash_map::Entry::Vacant(vacant) => {
        let node = Node::new(name, data.into());
        (uuid, vacant.insert(node))
      }
      std::collections::hash_map::Entry::Occupied(_) => {
        panic!("UUID collision detected");
      }
    }
  }

  pub fn add_node_with_rng(
    &mut self, name: impl Into<Option<String>>, data: impl Into<NodeData>,
    rng: &mut impl rand::RngCore,
  ) -> (Uuid, &Node) {
    let uuid = uuid::Builder::from_random_bytes(rng.random()).into_uuid();
    self.add_node_uuid(uuid, name, data)
  }

  #[allow(dead_code)]
  pub fn add_node(
    &mut self, name: impl Into<Option<String>>, data: impl Into<NodeData>,
  ) -> (Uuid, &Node) {
    self.add_node_uuid(Uuid::new_v4(), name, data)
  }

  fn check_cycle(&self, parent: Uuid, child: Uuid) -> Result<(), Error> {
    let mut queue = VecDeque::from([child]);
    let mut queued = HashSet::from([child]);

    while let Some(current) = queue.pop_front() {
      if current == parent {
        return Err(Error::Cycle {
          src: parent,
          dst: child,
        });
      }

      if let Some(node) = self.nodes.get(&current) {
        for &child in &node.childs {
          if queued.insert(child) {
            queue.push_back(child);
          }
        }
      } else {
        return Err(Error::UuidNotFound { uuid: current });
      }
    }

    Ok(())
  }

  pub fn get_node(&self, uuid: Uuid) -> Result<&Node, Error> {
    if let Some(node) = self.nodes.get(&uuid) {
      Ok(node)
    } else {
      Err(Error::UuidNotFound { uuid })
    }
  }

  pub fn get_node_mut(&mut self, uuid: Uuid) -> Result<&mut Node, Error> {
    if let Some(node) = self.nodes.get_mut(&uuid) {
      Ok(node)
    } else {
      Err(Error::UuidNotFound { uuid })
    }
  }

  pub fn add_child(&mut self, parent: Uuid, child: Uuid) -> Result<(), Error> {
    self.check_cycle(parent, child)?;
    if self.get_node_mut(parent)?.childs.insert(child) {
      Ok(())
    } else {
      Err(Error::ChildAlreadyExist { parent, child })
    }
  }

  pub fn dot(&self) -> Dot {
    Dot { graph: self }
  }

  pub fn mermaid(&self) -> Mermaid {
    Mermaid { graph: self }
  }

  pub fn parents(&self) -> HashMap<Uuid, HashSet<Uuid>> {
    let mut parents: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();

    for (&uuid, node) in &self.nodes {
      for &child in &node.childs {
        parents.entry(child).or_default().insert(uuid);
      }
    }

    parents
  }
}

pub struct Dot<'a> {
  graph: &'a AcyclicGraph,
}

impl Display for Dot<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    writeln!(f, "digraph \"{}\" {{", self.graph.name)?;
    writeln!(f, "  node [shape = box]")?;
    writeln!(f, "  graph [rankdir = TB]")?;
    writeln!(f)?;
    for parent in self.graph.nodes.iter().sorted_by_key(|node| node.0) {
      // Node
      write!(f, "  \"{}\"", ShortUuid::from_uuid(parent.0))?;
      if let Some(name) = &parent.1.name {
        write!(f, " [label = \"{name}\"]")?;
      }
      writeln!(f, ";")?;

      // Childs
      let mut childrens = parent.1.childs.iter().sorted();
      if let Some(first) = childrens.next() {
        write!(
          f,
          "  \"{}\" -> {{\"{}\"",
          ShortUuid::from_uuid(parent.0),
          ShortUuid::from_uuid(first)
        )?;
        for child in childrens {
          write!(f, " \"{}\"", ShortUuid::from_uuid(child))?;
        }
        writeln!(f, "}};")?;
      }
    }

    writeln!(f, "}}")
  }
}

pub struct Mermaid<'a> {
  graph: &'a AcyclicGraph,
}

impl Display for Mermaid<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    writeln!(f, "---")?;
    writeln!(f, "title: {}", self.graph.name)?;

    writeln!(f, "---")?;
    writeln!(f, "flowchart TB")?;
    for parent in self.graph.nodes.iter().sorted_by_key(|node| node.0) {
      // Node
      write!(f, "  {}", ShortUuid::from_uuid(parent.0))?;
      if let Some(name) = &parent.1.name {
        write!(f, "[{name}]")?;
      }

      // Childrens
      let mut childrens = parent.1.childs.iter().sorted();
      if let Some(child) = childrens.next() {
        write!(f, " --> {}", ShortUuid::from_uuid(child))?;
        for child in childrens {
          write!(f, " & {}", ShortUuid::from_uuid(child))?;
        }
      }
      writeln!(f)?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use rand::{
    SeedableRng,
    rngs::StdRng,
  };

  use super::*;

  #[test]
  fn test_add_node() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (uuid, node) = graph.add_node("Node".to_string(), "This is some data");
    let node = node.clone();
    assert_eq!(node.name.as_deref(), Some("Node"));
    match &node.data {
      NodeData::Text(s) => assert_eq!(s, "This is some data"),
      _ => panic!("Expected NodeData::Text"),
    }
    assert_eq!(graph.get_node(uuid), Ok(&node));
  }

  #[test]
  fn test_add_child() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let parent_node = graph.get_node(parent_uuid).unwrap();
    assert!(parent_node.childs.contains(&child_uuid));
  }

  #[test]
  fn test_cycle_detection() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let result = graph.add_child(child_uuid, parent_uuid);
    assert!(matches!(result, Err(Error::Cycle { .. })));
  }

  #[test]
  fn test_child_already_exist() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let result = graph.add_child(parent_uuid, child_uuid);
    assert!(matches!(result, Err(Error::ChildAlreadyExist { .. })));
  }

  #[test]
  fn test_uuid_not_found() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (node_uuid, _) = graph.add_node("Node".to_string(), ());
    let fake_uuid = Uuid::new_v4();
    let result = graph.add_child(node_uuid, fake_uuid);
    assert!(matches!(result, Err(Error::UuidNotFound { .. })));
  }

  #[test]
  fn test_dot_format() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node_with_rng("Parent".to_string(), (), &mut rng);
    let (child_uuid, _) = graph.add_node_with_rng("Child".to_string(), (), &mut rng);
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let dot_output = format!("{}", graph.dot());
    dot_parser::ast::Graph::try_from(dot_output.as_str()).expect("DOT format is invalid");

    let expected_output = r###"digraph "Test Graph" {
  node [shape = box]
  graph [rankdir = TB]

  "cDe6M3HmMtiJnhL4ihtnyx" [label = "Child"];
  "m43pF1xXxnZvhCY1VeAnMV" [label = "Parent"];
  "m43pF1xXxnZvhCY1VeAnMV" -> {"cDe6M3HmMtiJnhL4ihtnyx"};
}
"###;
    pretty_assertions::assert_eq!(dot_output, expected_output);
  }

  #[test]
  fn test_mermaid_format() {
    let mut rng = StdRng::seed_from_u64(42);
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node_with_rng("Parent".to_string(), (), &mut rng);
    let (child_uuid, _) = graph.add_node_with_rng("Child".to_string(), (), &mut rng);
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let mermaid_output = format!("{}", graph.mermaid());

    let expected_output = r###"---
title: Test Graph
---
flowchart TB
  cDe6M3HmMtiJnhL4ihtnyx[Child]
  m43pF1xXxnZvhCY1VeAnMV[Parent] --> cDe6M3HmMtiJnhL4ihtnyx
"###;
    pretty_assertions::assert_eq!(mermaid_output, expected_output);
  }

  #[test]
  fn test_parents() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let parents = graph.parents();
    assert!(parents.get(&child_uuid).unwrap().contains(&parent_uuid));
  }
}
