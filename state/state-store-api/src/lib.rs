use anyhow::Result;
use forkable_jellyfish_merkle::node_type::Node;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::hash::HashValue;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNode(pub Node);

impl StateNode {
    pub fn inner(&self) -> &Node {
        &self.0
    }
}

impl From<Node> for StateNode {
    fn from(n: Node) -> Self {
        StateNode(n)
    }
}

impl<'de> Deserialize<'de> for StateNode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <&[u8]>::deserialize(deserializer)?;
        let node = Node::decode(bytes).unwrap();
        Ok(StateNode::from(node))
    }
}

impl Serialize for StateNode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = Node::encode(self.inner()).unwrap();
        bytes.serialize(serializer)
    }
}

pub trait StateNodeStore: std::marker::Send + std::marker::Sync {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>>;
    fn put(&self, key: HashValue, node: StateNode) -> Result<()>;
    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()>;
}
