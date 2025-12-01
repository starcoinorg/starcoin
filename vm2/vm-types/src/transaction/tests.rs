use crate::account_address::AccountAddress;
use crate::account_config::genesis_address;
use crate::block_metadata::BlockMetadata;
use crate::on_chain_resource::ChainId;
use crate::transaction::{Script, Transaction};
use crate::transaction_argument::convert_txn_args;
use move_core_types::transaction_argument::TransactionArgument;
use move_core_types::u256;
use starcoin_crypto::HashValue;
use std::collections::BTreeSet;

#[test]
fn test_transaction_argument_to_json() {
    let script = Script::new(
        vec![],
        vec![],
        convert_txn_args(&[
            TransactionArgument::U8(u8::max_value()),
            TransactionArgument::U64(u64::max_value()),
            TransactionArgument::U128(u128::max_value()),
            TransactionArgument::Bool(true),
            TransactionArgument::Address(AccountAddress::random()),
            TransactionArgument::U8Vector(vec![0u8]),
            TransactionArgument::U16(u16::max_value()),
            TransactionArgument::U32(u32::max_value()),
            TransactionArgument::U256(u256::U256::max_value()),
        ]),
    );
    let raw_json = serde_json::to_string(&script).expect("json to_string should success.");
    let value = serde_json::from_str(raw_json.as_str()).expect("json from_str should success.");
    assert_eq!(script, value);
    let serialized = serde_json::to_value(&script).expect("json to_value should success.");
    let deserialized = serde_json::from_value(serialized).expect("json from_value should success.");
    assert_eq!(script, deserialized);
}

#[test]
fn block_epilogue_id_depends_on_senders() {
    let metadata = BlockMetadata::new(
        HashValue::zero(),
        1,
        genesis_address(),
        0,
        1,
        ChainId::test(),
        0,
        vec![HashValue::zero()],
        0,
    );

    let mut senders_one = BTreeSet::new();
    senders_one.insert(AccountAddress::from_hex_literal("0x1").unwrap());

    let mut senders_two = senders_one.clone();
    senders_two.insert(AccountAddress::from_hex_literal("0x2").unwrap());

    let epilogue_one = Transaction::BlockEpilogue(metadata.clone(), senders_one);
    let epilogue_two = Transaction::BlockEpilogue(metadata, senders_two);

    assert_ne!(epilogue_one.id(), epilogue_two.id());
}
