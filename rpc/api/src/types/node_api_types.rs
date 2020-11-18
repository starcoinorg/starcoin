use serde::{Deserialize, Serialize};
use starcoin_vm_types::genesis_config::ChainNetworkID;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainId {
    pub name: String,
    pub id: u8,
}

impl From<&ChainNetworkID> for ChainId {
    fn from(id: &ChainNetworkID) -> Self {
        match id {
            ChainNetworkID::Builtin(t) => ChainId {
                name: t.chain_name(),
                id: t.chain_id().id(),
            },
            ChainNetworkID::Custom(t) => ChainId {
                name: t.chain_name().to_string(),
                id: t.chain_id().id(),
            },
        }
    }
}
