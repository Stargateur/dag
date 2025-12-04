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
pub struct Graph {
  name: String,
  pub nodes: HashMap<Uuid, Node>,
}

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
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

impl Graph {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      nodes: HashMap::new(),
    }
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
}

pub struct Dot<'a> {
  graph: &'a Graph,
}

impl Display for Dot<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "digraph \"{}\" {{\n", self.graph.name)?;
    for parent in &self.graph.nodes {
      write!(f, "  {}", ShortUuid::from_uuid(parent.0))?;

      let mut childrens = parent.1.childs.iter();
      if let Some(first) = childrens.next() {
        write!(f, "  -> {{{}", ShortUuid::from_uuid(first))?;
        for child in childrens {
          write!(f, " {}", ShortUuid::from_uuid(child))?;
        }
        write!(f, "}}")?;
      }
      write!(f, ";\n")?;
    }
    write!(f, "}}\n")
  }
}

pub struct Mermaid<'a> {
  graph: &'a Graph,
}

impl Display for Mermaid<'_> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "flowchart \"{}\" {{\n", self.graph.name)?;
    for parent in &self.graph.nodes {
      write!(f, "  {}\n", ShortUuid::from_uuid(parent.0))?;

      let mut childrens = parent.1.childs.iter();
      if let Some(child) = childrens.next() {
        write!(f, "  --> {}", ShortUuid::from_uuid(child))?;
        for child in childrens {
          write!(f, " & {}", ShortUuid::from_uuid(child))?;
        }
      }
    }
    write!(f, "\n")
  }
}
