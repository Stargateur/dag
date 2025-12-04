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

use short_uuid::ShortUuid;
use snafu::Snafu;
use uuid::Uuid;

#[derive(Debug, Snafu)]
pub enum Error {
  #[snafu(display("Cycle detected {src} => {dst}"))]
  Cycle { src: Uuid, dst: Uuid },
  #[snafu(display("UUID not found: {uuid}"))]
  UuidNotFound { uuid: Uuid },
  #[snafu(display("Child already exist from {src} to {dst}"))]
  ChildAlreadyExist { src: Uuid, dst: Uuid },
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

  pub fn add_node(
    &mut self, name: impl Into<Option<String>>, data: impl Into<NodeData>,
  ) -> (Uuid, &Node) {
    let uuid = Uuid::new_v4();
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

  fn is_cycle(&self, src: Uuid, dst: Uuid) -> Result<(), Error> {
    let mut queue = VecDeque::from([dst]);
    let mut queued = HashSet::from([dst]);

    while let Some(current) = queue.pop_front() {
      if current == src {
        return Err(Error::Cycle { src, dst });
      }

      if let Some(node) = self.nodes.get(&current) {
        for &child in &node.childs {
          if queued.insert(child) {
            queue.push_back(child);
          }
        }
      }
    }

    Ok(())
  }

  pub fn get_node(&self, uuid: Uuid) -> Option<&Node> {
    self.nodes.get(&uuid)
  }

  pub fn get_node_mut(&mut self, uuid: Uuid) -> Option<&mut Node> {
    self.nodes.get_mut(&uuid)
  }

  pub fn check_node_exist(&self, uuid: Uuid) -> Result<(), Error> {
    if self.get_node(uuid).is_none() {
      Err(Error::UuidNotFound { uuid })
    } else {
      Ok(())
    }
  }

  pub fn check_nodes_exist(&self, uuids: &[Uuid]) -> Result<(), Error> {
    for &uuid in uuids {
      self.check_node_exist(uuid)?;
    }
    Ok(())
  }

  pub fn add_child(&mut self, src: Uuid, dst: Uuid) -> Result<(), Error> {
    self.check_nodes_exist(&[src, dst])?;
    self.is_cycle(src, dst)?;
    if self.get_node_mut(src).unwrap().childs.insert(dst) {
      Ok(())
    } else {
      Err(Error::ChildAlreadyExist { src, dst })
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
    write!(f, "  graph [rankdir = \"LR\"]\n\n")?;
    for parent in &self.graph.nodes {
      write!(f, "  \"{}\"", ShortUuid::from_uuid(parent.0))?;

      let mut childrens = parent.1.childs.iter();
      if let Some(first) = childrens.next() {
        write!(f, "  -> {{\"{}\"", ShortUuid::from_uuid(first))?;
        for child in childrens {
          write!(f, " \"{}\"", ShortUuid::from_uuid(child))?;
        }
        write!(f, "}}")?;
      }
      writeln!(f, ";")?;
    }
    writeln!(f, "}}")
  }
}

pub struct Mermaid<'a> {
  graph: &'a AcyclicGraph,
}

impl Display for Mermaid<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "---\ntitle: {}\n---\n", self.graph.name)?;
    writeln!(f, "flowchart LR")?;
    for parent in &self.graph.nodes {
      write!(f, "  {}", ShortUuid::from_uuid(parent.0))?;

      let mut childrens = parent.1.childs.iter();
      if let Some(child) = childrens.next() {
        write!(f, "  --> {}", ShortUuid::from_uuid(child))?;
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
    assert_eq!(graph.get_node(uuid), Some(&node));
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
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let dot_output = format!("{}", graph.dot());
    dot_parser::ast::Graph::try_from(dot_output.as_str()).expect("DOT format is invalid");
  }

  #[test]
  fn test_mermaid_format() {
    let mut graph = AcyclicGraph::new("Test Graph");
    let (parent_uuid, _) = graph.add_node("Parent".to_string(), ());
    let (child_uuid, _) = graph.add_node("Child".to_string(), ());
    assert!(graph.add_child(parent_uuid, child_uuid).is_ok());
    let _mermaid_output = format!("{}", graph.mermaid());
    // No standard parser for Mermaid, so we just ensure it generates without
    // error FIXME
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
