use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use starcoin_crypto::hash::{CryptoHash, CryptoHasher};
use starcoin_crypto::HashValue;

#[derive(
    Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, CryptoHasher, CryptoHash, JsonSchema,
)]
pub struct MultiState {
    state_root1: HashValue,
    state_root2: HashValue,
}

impl MultiState {
    pub fn new(state_root1: HashValue, state_root2: HashValue) -> Self {
        Self {
            state_root1,
            state_root2,
        }
    }

    pub fn state_root1(&self) -> &HashValue {
        &self.state_root1
    }

    pub fn state_root2(&self) -> &HashValue {
        &self.state_root2
    }
}
