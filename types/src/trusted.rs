use crate::blockhash::{BlockHashMap, BlueWorkType, KType};
use serde::{Deserialize, Serialize};
use starcoin_crypto::HashValue as Hash;

/// Represents semi-trusted externally provided Ghostdag data (by a network peer)
#[derive(Clone, Serialize, Deserialize)]
pub struct ExternalGhostdagData {
    pub blue_score: u64,
    pub blue_work: BlueWorkType,
    pub selected_parent: Hash,
    pub mergeset_blues: Vec<Hash>,
    pub mergeset_reds: Vec<Hash>,
    pub blues_anticone_sizes: BlockHashMap<KType>,
}

/// Represents externally provided Ghostdag data associated with a block Hash
pub struct TrustedGhostdagData {
    pub hash: Hash,
    pub ghostdag: ExternalGhostdagData,
}

impl TrustedGhostdagData {
    pub fn new(hash: Hash, ghostdag: ExternalGhostdagData) -> Self {
        Self { hash, ghostdag }
    }
}
