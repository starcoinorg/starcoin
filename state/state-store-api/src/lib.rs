use anyhow::Result;
use forkable_jellyfish_merkle::node_type::Node;
use forkable_jellyfish_merkle::RawKey;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starcoin_crypto::hash::HashValue;
use std::collections::BTreeMap;
use std::convert::{TryFrom, TryInto};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateNode(pub Vec<u8>);

impl<K> TryFrom<Node<K>> for StateNode
where
    K: RawKey,
{
    type Error = anyhow::Error;

    fn try_from(n: Node<K>) -> Result<Self> {
        Ok(StateNode(n.encode()?))
    }
}

impl<K> TryInto<Node<K>> for StateNode
where
    K: RawKey,
{
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Node<K>, Self::Error> {
        Node::decode(self.0.as_slice())
    }
}

impl<'de> Deserialize<'de> for StateNode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = <&[u8]>::deserialize(deserializer)?;
        Ok(Self(bytes.to_vec()))
    }
}

impl Serialize for StateNode {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

pub trait StateNodeStore: std::marker::Send + std::marker::Sync {
    fn get(&self, hash: &HashValue) -> Result<Option<StateNode>>;
    fn put(&self, key: HashValue, node: StateNode) -> Result<()>;
    fn write_nodes(&self, nodes: BTreeMap<HashValue, StateNode>) -> Result<()>;
}
