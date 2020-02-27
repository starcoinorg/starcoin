use starcoin_crypto::{hash::CryptoHash, HashValue};

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct PeerInfo {
    id: HashValue,
}

impl PeerInfo {
    pub fn new(hash: HashValue) -> Self {
        PeerInfo { id: hash }
    }

    pub fn random() -> Self {
        PeerInfo {
            id: HashValue::random(),
        }
    }
}
