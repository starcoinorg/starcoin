use crate::transaction::Script;
use crate::transaction_argument::convert_txn_args;
use move_core_types::account_address::AccountAddress;
use move_core_types::transaction_argument::TransactionArgument;

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
        ]),
    );
    let json_str = serde_json::to_string(&script).expect("json to_string should success.");
    let script2 = serde_json::from_str(json_str.as_str()).expect("json from_str should success.");
    assert_eq!(script, script2);
    let json_value = serde_json::to_value(&script).expect("json to_value should success.");
    let script3 = serde_json::from_value(json_value).expect("json from_value should success.");
    assert_eq!(script, script3);
}
