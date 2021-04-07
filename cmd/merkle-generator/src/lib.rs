use crypto::digest::Digest;
use crypto::sha3::Sha3;
use merkletree::hash::Algorithm;
use serde::Deserialize;
use serde::Serialize;
use starcoin_vm_types::account_address::AccountAddress;
use std::hash::Hasher;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DataProof {
    pub address: AccountAddress,
    pub index: u64,
    pub amount: u128,
    /// proofs in hex string
    pub proof: Vec<String>,
}
pub fn encode(idx: u64, address: AccountAddress, amount: u128) -> anyhow::Result<Vec<u8>> {
    let mut index = bcs_ext::to_bytes(&idx)?;
    let mut address = bcs_ext::to_bytes(&address)?;
    let mut amount = bcs_ext::to_bytes(&amount)?;
    index.append(&mut address);
    index.append(&mut amount);
    Ok(index)
}

pub struct Sha3Algorithm(Sha3);

impl Default for Sha3Algorithm {
    fn default() -> Self {
        Self(Sha3::sha3_256())
    }
}

impl Hasher for Sha3Algorithm {
    #[inline]
    fn finish(&self) -> u64 {
        unimplemented!()
    }

    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.input(msg)
    }
}

impl Algorithm<[u8; 32]> for Sha3Algorithm {
    #[inline]
    fn hash(&mut self) -> [u8; 32] {
        let mut h = [0u8; 32];
        self.0.result(&mut h);
        h
    }

    #[inline]
    fn reset(&mut self) {
        self.0.reset();
    }
    /// Returns hash value for MT leaf (prefix 0x00).
    #[inline]
    fn leaf(&mut self, leaf: [u8; 32]) -> [u8; 32] {
        leaf
    }

    /// Returns hash value for MT interior node (prefix 0x01).
    #[inline]
    fn node(&mut self, left: [u8; 32], right: [u8; 32], _height: usize) -> [u8; 32] {
        if left <= right {
            self.write(left.as_ref());
            self.write(right.as_ref());
        } else {
            self.write(right.as_ref());
            self.write(left.as_ref());
        }
        self.hash()
    }

    #[inline]
    fn multi_node(&mut self, nodes: &[[u8; 32]], height: usize) -> [u8; 32] {
        if nodes.len() > 2 {
            unimplemented!()
        }

        let node1 = nodes[0];
        let node2 = nodes[1];
        self.node(node1, node2, height)
    }
}

#[cfg(test)]
mod tests {
    use crate::{encode, DataProof};
    use starcoin_crypto::HashValue;

    #[test]
    fn test_proof_verify() -> anyhow::Result<()> {
        let merkle_data = include_str!("../examples/merkle-example.json");
        let merkle_data: serde_json::Value = serde_json::from_str(merkle_data)?;
        let root = merkle_data["root"].as_str().unwrap();
        let root_in_bytes = hex::decode(root.strip_prefix("0x").unwrap_or(root))?;
        let proofs: Vec<DataProof> = serde_json::from_value(merkle_data["proofs"].clone())?;

        let leaf = encode(proofs[0].index, proofs[0].address, proofs[0].amount).unwrap();
        let proof: anyhow::Result<Vec<Vec<u8>>, _> = proofs[0]
            .proof
            .iter()
            .map(|p| hex::decode(p.as_str().strip_prefix("0x").unwrap_or_else(|| p.as_str())))
            .collect();
        let proof = proof?;
        let mut computed_hash = HashValue::sha3_256_of(leaf.as_slice()).to_vec();

        for p in &proof {
            let joined = if &computed_hash <= p {
                let mut joined = computed_hash.clone();
                joined.append(&mut p.clone());
                joined
            } else {
                let mut joined = p.clone();
                joined.append(&mut computed_hash.clone());
                joined
            };
            computed_hash = HashValue::sha3_256_of(joined.as_slice()).to_vec();
        }
        assert_eq!(root_in_bytes, computed_hash);
        Ok(())
    }
}
