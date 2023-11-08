// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0
use diem_types::{
    access_path::AccessPath,
    on_chain_config::DiemVersion,
    transaction::{ChangeSet, Script, TransactionStatus, WriteSetPayload},
    vm_status::KeptVMStatus,
    write_set::WriteOp,
};
use diem_vm::DiemVM;
use diem_writeset_generator::build_changeset;
use move_ir_compiler::Compiler;
use starcoin_language_e2e_tests::{
    account::Account, compile::compile_module, current_function_name, executor::FakeExecutor,
};

#[test]
fn build_upgrade_writeset() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    // create a transaction trying to publish a new module.
    let genesis_account = Account::new_diem_root();

    let program = String::from(
        "
        module 0x1.M {
            public magic(): u64 { label b0: return 42; }
        }
        ",
    );

    let module = compile_module(&program).0;
    let module_bytes = {
        let mut v = vec![];
        module.serialize(&mut v).unwrap();
        v
    };
    let change_set = {
        let (version_writes, events) = build_changeset(executor.get_state_view(), |session| {
            session.set_diem_version(11);
        })
        .into_inner();
        let mut writeset = version_writes.into_mut();
        writeset.push((
            AccessPath::code_access_path(module.self_id()),
            WriteOp::Value(module_bytes),
        ));
        ChangeSet::new(writeset.freeze().unwrap(), events)
    };

    let writeset_txn = genesis_account
        .transaction()
        .write_set(WriteSetPayload::Direct(change_set))
        .sequence_number(0)
        .sign();

    let output = executor.execute_transaction(writeset_txn.clone());
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
    assert!(executor.verify_transaction(writeset_txn).status().is_none());

    executor.apply_write_set(output.write_set());

    let new_vm = DiemVM::new(executor.get_state_view());
    assert_eq!(
        new_vm.internals().diem_version().unwrap(),
        DiemVersion { major: 11 }
    );

    let script_body = {
        let code = r#"
import 0x1.M;

main(lr_account: signer) {
label b0:
  assert(M.magic() == 42, 100);
  return;
}
"#;

        let compiler = Compiler {
            deps: vec![&module],
        };
        compiler.into_script_blob(code).expect("Failed to compile")
    };

    let txn = genesis_account
        .transaction()
        .script(Script::new(script_body, vec![], vec![]))
        .sequence_number(1)
        .sign();

    let output = executor.execute_transaction(txn);
    assert_eq!(
        output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed)
    );
}
