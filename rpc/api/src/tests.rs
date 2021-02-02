use crate::types::{ContractCall, TransactionArgumentView, TypeTagView};
use starcoin_vm_types::token::stc::stc_type_tag;
use starcoin_vm_types::transaction_argument::TransactionArgument;

#[test]
fn test_view_of_type_tag() {
    let ty_tag = stc_type_tag();
    let s = serde_json::to_string(&TypeTagView::from(ty_tag.clone())).unwrap();
    println!("{}", &s);
    let ty_tag_view: TypeTagView = serde_json::from_str(s.as_str()).unwrap();
    assert_eq!(ty_tag_view.0, ty_tag);
}

#[test]
fn test_view_of_transaction_arg() {
    let arg = TransactionArgument::U8(1);
    let s = serde_json::to_string(&TransactionArgumentView::from(arg.clone())).unwrap();
    println!("{}", &s);
    let view: TransactionArgumentView = serde_json::from_str(s.as_str()).unwrap();
    assert_eq!(view.0, arg);
}

#[test]
fn test_deserialize() {
    let s = r#"
{
  "module_address": "0x0CC02653F9D7A62D07754D859B066BDE",
  "module_name": "T",
  "func": "A",
  "type_args": [ "0x42C4DDA17CC39AF459C20D09F6A82EDF::T::T"],
  "args": ["0xD6F8FAF8FA976104B8BA8C6F85DCF9E4"]
}        
        "#;
    let v = serde_json::from_str::<ContractCall>(s).unwrap();
    println!("{:?}", v);
}
