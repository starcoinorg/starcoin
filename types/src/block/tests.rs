use super::legacy::{BlockBody, BlockHeader};
use crate::{
    account_address::AccountAddress,
    account_config::CORE_CODE_ADDRESS,
    block::{BlockBody as DagBlockBody, BlockHeaderExtra},
};
use bcs_ext::Sample;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::{ed25519::genesis_key_pair, HashValue};
use starcoin_uint::U256;
use starcoin_vm_types::genesis_config::ChainId;
use starcoin_vm_types::transaction::{
    Package, RawUserTransaction, SignedUserTransaction, TransactionPayload,
};
use std::str::FromStr;

fn this_header() -> BlockHeader {
    let header_id =
        HashValue::from_str("0x85d3b70cbe4c0ccc39d28af77214303d21d2dbae32a8cf8cf8f9da50e1fe4e50")
            .unwrap();
    let parent_hash =
        HashValue::from_str("0x863b7525f5404eae39c0462b572c84eaa23a5fb0728cebfe1924351b7dc54ece")
            .unwrap();
    let timestamp = 1703079047026u64;
    let number = 15780908u64;
    let author = AccountAddress::from_str("0xd9b2d56e8d20a911b2dc5929695f4ec0").unwrap();
    //let author_auth_key = None;
    let txn_accumulator_root =
        HashValue::from_str("0x610e248024614f5c44bc036001809e14e32aa0b922ba2be625cc0d099d49d373")
            .unwrap();
    let block_accumulator_root =
        HashValue::from_str("0xcd70b9a4f3bb71d4228f461d13b9ea438dc6c3c26f7df465ea141f5dd5bca063")
            .unwrap();
    let state_root =
        HashValue::from_str("0xcbcfb2a8bdfd4a4d26ee70068a28f484a819b0220debe5820ff0a5c342f81a83")
            .unwrap();
    let gas_used = 0;
    let difficulty = U256::from(162878673u64);
    let body_hash =
        HashValue::from_str("0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97")
            .unwrap();
    let chain_id = ChainId::new(1);
    let nonce = 83887534u32;
    let extra = BlockHeaderExtra::new([205, 193, 0, 0]);

    let header = BlockHeader::new_with_auth_key(
        parent_hash,
        timestamp,
        number,
        author,
        None,
        txn_accumulator_root,
        block_accumulator_root,
        state_root,
        gas_used,
        difficulty,
        body_hash,
        chain_id,
        nonce,
        extra,
    );

    assert_eq!(header.id.unwrap(), header_id);
    header
}

fn this_signed_txn() -> SignedUserTransaction {
    let txn = RawUserTransaction::new_with_default_gas_token(
        CORE_CODE_ADDRESS,
        0,
        TransactionPayload::Package(Package::sample()),
        0,
        0,
        1, // init to 1 to pass time check
        ChainId::test(),
    );
    let (genesis_private_key, genesis_public_key) = genesis_key_pair();
    let sign_txn = txn.sign(&genesis_private_key, genesis_public_key).unwrap();
    sign_txn.into_inner()
}

#[test]
fn verify_body_hash_with_uncles() {
    let body_hash =
        HashValue::from_str("0x00592ee74f78a848089083febe0621f45d92b70c8f5a0d4b4f6123b6b01a241b")
            .unwrap();

    let body = BlockBody {
        transactions: vec![],
        uncles: Some(vec![this_header()]),
    };
    assert_eq!(body.crypto_hash(), body_hash);

    let dag_body: DagBlockBody = body.clone().into();
    assert_ne!(body_hash, dag_body.crypto_hash());

    let converted_body: BlockBody = dag_body.into();
    assert_eq!(body.crypto_hash(), converted_body.crypto_hash());
}

#[test]
fn verify_empty_body_hash() {
    let empty_hash =
        HashValue::from_str("0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97")
            .unwrap();
    let empty_body = BlockBody {
        transactions: vec![],
        uncles: None,
    };
    assert_eq!(empty_hash, empty_body.crypto_hash());

    let empty_dag_body: DagBlockBody = empty_body.clone().into();
    assert_eq!(empty_hash, empty_dag_body.crypto_hash());

    let converted_empty_body: BlockBody = empty_dag_body.into();
    assert_eq!(empty_body.crypto_hash(), converted_empty_body.crypto_hash());
}

#[test]
fn verify_zero_uncle_body_hash() {
    let empty_hash =
        HashValue::from_str("0xc01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97")
            .unwrap();
    let body = BlockBody {
        transactions: vec![],
        uncles: Some(vec![]),
    };

    assert_ne!(empty_hash, body.crypto_hash());

    let dag_body: DagBlockBody = body.clone().into();
    let converted_body: BlockBody = dag_body.clone().into();

    assert_eq!(body.crypto_hash(), converted_body.crypto_hash());
    assert_eq!(body.crypto_hash(), dag_body.crypto_hash());
}

#[test]
fn verify_empty_uncles_body_hash() {
    let body = BlockBody {
        transactions: vec![this_signed_txn()],
        uncles: None,
    };

    let dag_body: DagBlockBody = body.clone().into();
    let converted_body: BlockBody = dag_body.clone().into();

    assert_eq!(body.crypto_hash(), converted_body.crypto_hash());
    assert_eq!(body.crypto_hash(), dag_body.crypto_hash());
}
