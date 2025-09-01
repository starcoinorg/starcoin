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

/// StateSet is represent a single state-tree or sub state-tree dump result.
#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct StateSet(Vec<(Vec<u8>, Vec<u8>)>);

impl StateSet {
    pub fn new(states: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Self(states)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> ::std::slice::Iter<(Vec<u8>, Vec<u8>)> {
        self.into_iter()
    }

    fn push(&mut self, key: Vec<u8>, blob: Vec<u8>) {
        self.0.push((key, blob))
    }
}

impl ::std::iter::FromIterator<(Vec<u8>, Vec<u8>)> for StateSet {
    fn from_iter<I: IntoIterator<Item = (Vec<u8>, Vec<u8>)>>(iter: I) -> Self {
        let mut s = Self::default();
        for write in iter {
            s.push(write.0, write.1);
        }
        s
    }
}

impl<'a> IntoIterator for &'a StateSet {
    type Item = &'a (Vec<u8>, Vec<u8>);
    type IntoIter = ::std::slice::Iter<'a, (Vec<u8>, Vec<u8>)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[allow(clippy::from_over_into)]
impl Into<Vec<(Vec<u8>, Vec<u8>)>> for StateSet {
    fn into(self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.0
    }
}
