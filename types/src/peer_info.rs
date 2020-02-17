use crypto::{hash::CryptoHash, HashValue};

#[derive(Eq, PartialEq, Hash, Debug)]
pub struct PeerInfo {
    id: HashValue,
}
