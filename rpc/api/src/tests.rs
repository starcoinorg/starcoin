use crate::types::{ContractCall, TransactionArgumentView, TypeTagView};
use starcoin_vm_types::token::stc::stc_type_tag;
use starcoin_vm_types::transaction_argument::TransactionArgument;
use std::path::PathBuf;
use std::process::Command;
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
  "function_id": "0x0CC02653F9D7A62D07754D859B066BDE::T::A",
  "type_args": [ "0x42C4DDA17CC39AF459C20D09F6A82EDF::T::T"],
  "args": ["0xD6F8FAF8FA976104B8BA8C6F85DCF9E4"]
}        
        "#;
    let v = serde_json::from_str::<ContractCall>(s).unwrap();
    println!("{:?}", v);
}
fn assert_that_version_control_has_no_unstaged_changes() {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .unwrap();
    if !output.stdout.is_empty() {
        println!(
            "git status output: {:?}",
            String::from_utf8(output.stdout.clone()).unwrap()
        )
    }
    assert!(
        output.stdout.is_empty(),
        "Git repository should be in a clean state"
    );
    assert!(output.status.success());
}

#[test]
fn test_generated_schema_are_up_to_date_in_git() {
    // Better not run the `stdlib` tool when the repository is not in a clean state.
    assert_that_version_control_has_no_unstaged_changes();
    let path = PathBuf::from("../../target/debug/starcoin-rpc-schema-generate")
        .canonicalize()
        .unwrap();
    assert!(Command::new(path)
        .current_dir("../")
        .status()
        .unwrap()
        .success());

    // Running the stdlib tool should not create unstaged changes.
    assert_that_version_control_has_no_unstaged_changes();
}
