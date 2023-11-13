// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::transaction_argument::TransactionArgument;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::StatusCode;
use move_ir_compiler::Compiler;
use starcoin_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};

use starcoin_language_e2e_tests::compile::{compile_script};
use starcoin_language_e2e_tests::{
    account::Account,  current_function_name,
    executor::FakeExecutor,
};

use starcoin_transaction_builder::{stdlib_compiled_modules, StdLibOptions};
use starcoin_vm_types::{
    genesis_config::StdlibVersion::Latest,
    on_chain_config,
    transaction::{authenticator::AuthenticationKey, Script},
};

#[test]
fn admin_script_rotate_key_single_signer_no_epoch() {
    let mut executor = FakeExecutor::from_test_genesis();
    //executor.set_golden_file(current_function_name!());
    let new_account_data = executor.create_raw_account_data(100_000, 0);
    executor.add_account_data(&new_account_data);

    // Generate new key pair
    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

//     let code = r#"
// import 0x1.Account;
//
// main(account: signer, auth_key_prefix: vector<u8>) {
//   let rotate_cap: Account.KeyRotationCapability;
// label b0:
//   rotate_cap = Account.extract_key_rotation_capability(&account);
//   Account.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
//   Account.restore_key_rotation_capability(move(rotate_cap));
//
//   return;
// }
// "#;

    let code = r#"
        script {
            use 0x1::Account;
            fun main(account: signer, auth_key_prefix: vector<u8>) {
                Account::do_rotate_authentication_key(&account, auth_key_prefix);
            }
        }
"#;
    let script_body = compile_script(code).expect("Compile script error!");

    let new_account = new_account_data.account();

    // Read KeyRotationCapability resource, make sure it is none
    assert!(!executor.read_account_resource(new_account).unwrap().has_delegated_key_rotation_capability());

    let txn = new_account
        .transaction()
        .script(Script::new(
            script_body,
            vec![],
            vec![
                MoveValue::from(TransactionArgument::U8Vector(new_key_hash.to_vec()))
                    .simple_serialize()
                    .expect("transaction arguments must serialize"),
            ],
        ))
        .sequence_number(0)
        .sign();

    // executor.new_block();
    let output = executor.execute_and_apply(txn);

    // // The transaction should not trigger a reconfiguration.
    let new_epoch_event_key = on_chain_config::new_epoch_event_key();
    assert!(!output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key));

    let updated_sender = executor
        .read_account_resource(new_account)
        .expect("sender must exist");

    assert_eq!(updated_sender.authentication_key(), new_key_hash.as_slice());
}

#[test]
fn admin_script_rotate_key_single_signer_new_epoch() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());
    let new_account = executor.create_raw_account_data(100_000, 0);
    executor.add_account_data(&new_account);

    let privkey = Ed25519PrivateKey::generate_for_testing();
    let pubkey = privkey.public_key();
    let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();

    let script_body = {
        let code = r#"
import 0x1.Account;
import 0x1.Config;

main(dr_account: signer, account: signer, auth_key_prefix: vector<u8>) {
  let rotate_cap: Account.KeyRotationCapability;
label b0:
  rotate_cap = Account.extract_key_rotation_capability(&account);
  Account.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
  Account.restore_key_rotation_capability(move(rotate_cap));

  Config.reconfigure(&dr_account);
  return;
}
"#;

        let deps = stdlib_compiled_modules(StdLibOptions::Compiled(Latest));
        let compiler = Compiler {
            deps: deps.iter().collect(),
        };
        compiler.into_script_blob(code).expect("Failed to compile")
    };
    let account = Account::new_starcoin_root();
    let txn = account
        .transaction()
        .script(Script::new(script_body, vec![], vec![new_key_hash.clone()]))
        .sequence_number(0)
        .sign();
    executor.new_block();
    let output = executor.execute_and_apply(txn);

    // The transaction should trigger a reconfiguration.
    let new_epoch_event_key = on_chain_config::new_epoch_event_key();
    assert!(output
        .events()
        .iter()
        .any(|event| *event.key() == new_epoch_event_key));

    let updated_sender = executor
        .read_account_resource(new_account.account())
        .expect("sender must exist");

    assert_eq!(updated_sender.authentication_key(), new_key_hash.as_slice());
}

// TODO(BobOng): testsuite
// #[test]
// fn admin_script_rotate_key_multi_signer() {
//     let mut executor = FakeExecutor::from_genesis_file();
//     executor.set_golden_file(current_function_name!());
//     let new_account = executor.create_raw_account_data(100_000, 0);
//     executor.add_account_data(&new_account);
//
//     let privkey = Ed25519PrivateKey::generate_for_testing();
//     let pubkey = privkey.public_key();
//     let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
//
//     let script_body = {
//         let code = r#"
// import 0x1.Account;
//
// main(account: signer, auth_key_prefix: vector<u8>) {
//   let rotate_cap: Account.KeyRotationCapability;
// label b0:
//   rotate_cap = Account.extract_key_rotation_capability(&account);
//   Account.rotate_authentication_key(&rotate_cap, move(auth_key_prefix));
//   Account.restore_key_rotation_capability(move(rotate_cap));
//
//   return;
// }
// "#;
//
//         let deps = stdlib_compiled_modules(StdLibOptions::Compiled(Latest));
//         let compiler = Compiler {
//             deps: deps.iter().collect(),
//         };
//         compiler.into_script_blob(code).expect("Failed to compile")
//     };
//     let account = Account::new_starcoin_root();
//     let txn = account
//         .transaction()
//         .script(Script::new(script_body, vec![], vec![new_key_hash]))
//         .sequence_number(0)
//         .sign();
//     executor.new_block();
//     let output = executor.execute_transaction(txn);
//     assert_eq!(output.status().status(), Err(StatusCode::INVALID_WRITE_SET));
// }
