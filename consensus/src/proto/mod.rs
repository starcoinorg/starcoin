use types::block::BlockHeader;
use libra_crypto::HashValue;

struct LatestStateMsg {
    header: BlockHeader,
}

struct GetHashByHeightMsg {
    heights:Vec<u64>,
}

struct HashWithHeight {
    hash:HashValue,
    height:u64,
}

struct BatchHashByHeightMsg {
    hashs:Vec<HashWithHeight>,
}

struct StateNodeHashMsg {
    hash:HashValue,
}

struct BatchStateNodeDataMsg {
    //nodes:
}

enum DataType {
    HEADER,
    BODY,
}

struct GetDataByHashMsg {
    hashs:Vec<HashValue>,
    data_type:DataType
}

struct BatchHeaderMsg {
    headers:Vec<BlockHeader>,
}

struct BlockBody {
    hash:HashValue,
    transactions: Vec<SignedTransaction>,
}

struct BatchBodyMsg {
    bodies:Vec<BlockBody>
}