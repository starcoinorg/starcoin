//! Network RPC request parameters, as decoded from peers by the generated server delegate
//! (`from_bytes::<Param>`), followed by the cheap validation the server runs before any IO.
#![no_main]

use bcs_ext::BCSCodec;
use forkable_jellyfish_merkle::RawKey;
use libfuzzer_sys::fuzz_target;
use serde::de::DeserializeOwned;
use serde::Serialize;
use starcoin_crypto::HashValue;
use starcoin_network_rpc_api::{
    GetAccountState, GetAccumulatorNodeByNodeHash, GetBlockHeadersByNumber, GetBlockIds,
    GetStateWithProof, GetStateWithTableItemProof, GetTableInfo, GetTxnsWithHash, GetTxnsWithSize,
    Ping, RpcRequest,
};
use starcoin_types::block::BlockNumber;
use starcoin_vm_types::access_path::DataPath;
use starcoin_vm_types::state_store::state_key::StateKey;

/// Decodes like the server delegate, then re-encodes: a decoded request must encode again,
/// since request params are logged and some are forwarded.
fn decode<T: Serialize + DeserializeOwned>(bytes: &[u8]) -> Option<T> {
    let value = bcs_ext::from_bytes::<T>(bytes).ok()?;
    bcs_ext::to_bytes(&value).expect("a decoded request must encode again");
    Some(value)
}

fn decode_verified<T: Serialize + DeserializeOwned + RpcRequest>(bytes: &[u8]) -> Option<T> {
    let value = decode::<T>(bytes)?;
    value.verify().ok()?;
    Some(value)
}

fuzz_target!(|data: &[u8]| {
    let Some((selector, bytes)) = data.split_first() else {
        return;
    };
    match selector % 12 {
        0 => {
            decode::<Ping>(bytes);
        }
        1 => {
            decode_verified::<GetTxnsWithSize>(bytes);
        }
        2 => {
            decode_verified::<GetTxnsWithHash>(bytes);
        }
        3 => {
            if let Some(req) = decode_verified::<GetBlockHeadersByNumber>(bytes) {
                let max_size = req.max_size;
                let numbers: Vec<BlockNumber> = req.into();
                assert!(numbers.len() as u64 <= max_size);
            }
        }
        4 => {
            decode_verified::<GetBlockIds>(bytes);
        }
        5 => {
            // get_block_infos, get_bodies_by_hash, get_headers_by_hash and get_blocks.
            decode::<Vec<HashValue>>(bytes);
        }
        6 => {
            // get_txn_infos and get_state_node_by_node_hash.
            decode::<HashValue>(bytes);
        }
        7 => {
            decode::<GetAccumulatorNodeByNodeHash>(bytes);
        }
        8 => {
            // The state service hashes the resource struct tag as a state tree key.
            if let Some(req) = decode::<GetStateWithProof>(bytes) {
                if let DataPath::Resource(struct_tag) = &req.access_path.path {
                    struct_tag.key_hash();
                }
                StateKey::AccessPath(req.access_path)
                    .encode()
                    .expect("a decoded access path must encode");
            }
        }
        9 => {
            decode::<GetAccountState>(bytes);
        }
        10 => {
            decode::<GetStateWithTableItemProof>(bytes);
        }
        _ => {
            decode::<GetTableInfo>(bytes);
        }
    }
});
