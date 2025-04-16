use crate::transaction::Script;
use crate::transaction_argument::convert_txn_args;
use move_core_types::account_address::AccountAddress;
use move_core_types::transaction_argument::TransactionArgument;
use move_core_types::u256;

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
