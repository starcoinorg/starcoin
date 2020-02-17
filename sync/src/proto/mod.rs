use libra_crypto::HashValue;
use std::cmp::Ordering;
use types::{block::BlockHeader, transaction::SignedUserTransaction};

struct LatestStateMsg {
    header: BlockHeader,
}

struct GetHashByHeightMsg {
    heights: Vec<u64>,
}

#[derive(Eq, PartialEq, PartialOrd, Clone, Debug)]
pub struct HashWithHeight {
    hash: HashValue,
    height: u64,
}

impl Ord for HashWithHeight {
    fn cmp(&self, other: &HashWithHeight) -> Ordering {
        match self.height.cmp(&other.height) {
            Ordering::Equal => {
                return self.hash.cmp(&other.hash);
            }
            ordering => return ordering,
        }
    }
}

struct BatchHashByHeightMsg {
    hashs: Vec<HashWithHeight>,
}

struct StateNodeHashMsg {
    hash: HashValue,
}

struct BatchStateNodeDataMsg {
    //nodes:
}

enum DataType {
    HEADER,
    BODY,
}

struct GetDataByHashMsg {
    hashs: Vec<HashValue>,
    data_type: DataType,
}

struct BatchHeaderMsg {
    headers: Vec<BlockHeader>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct BlockBody {
    hash: HashValue,
    transactions: Vec<SignedUserTransaction>,
}

impl PartialOrd for BlockBody {
    fn partial_cmp(&self, other: &BlockBody) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockBody {
    fn cmp(&self, other: &BlockBody) -> Ordering {
        return self.hash.cmp(&other.hash);
    }
}

struct BatchBodyMsg {
    bodies: Vec<BlockBody>,
}
