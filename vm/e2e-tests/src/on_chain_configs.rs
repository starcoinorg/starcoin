// Copyright (c) Starcoin
// SPDX-License-Identifier: Apache-2.0

use crate::account::Account;
use crate::executor::FakeExecutor;
use move_core_types::transaction_argument::{convert_txn_args, TransactionArgument};
use move_ir_compiler::Compiler;
use starcoin_vm_runtime::starcoin_vm::StarcoinVM;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::transaction::Script;
use stdlib::{stdlib_compiled_modules, StdLibOptions};

pub fn set_starcoin_version(executor: &mut FakeExecutor, version: Version) {
    let account =
        Account::new_genesis_account(starcoin_vm_types::account_config::genesis_address());
    // let txn = account
    //     .transaction()
    //     .script(
    //         LegacyStdlibScript::UpdateDiemVersion
    //             .compiled_bytes()
    //             .into_vec(),
    //         vec![],
    //         vec![
    //             TransactionArgument::U64(0),
    //             TransactionArgument::U64(version.major),
    //         ],)
    //     .payload(starcoin_stdlib::encode_version_set_version(version.major))
    //     .sequence_number(0)
    //     .sign();
    let script_body = {
        let code = r#"
import 0x1.Config;
import 0x1.LanguageVersion;

main(account: signer, language_version: u8) {
    // initialize the language version config.
    Config::publish_new_config(sender, LanguageVersion::new(language_version));
}
"#;

        let modules = stdlib_compiled_modules(StdLibOptions::Compiled(StdlibVersion::Latest));
        let compiler = Compiler {
            deps: modules.iter().collect(),
        };
        compiler.into_script_blob(code).expect("Failed to compile")
    };
    // let account = Account::new_starcoin_root();
    let txn = account
        .transaction()
        .script(Script::new(
            script_body,
            vec![],
            convert_txn_args(&vec![TransactionArgument::U64(version.major)]),
        ))
        .sequence_number(0)
        .sign();

    executor.new_block();
    executor.execute_and_apply(txn);

    let new_vm = StarcoinVM::new(None);
    assert_eq!(new_vm.get_version().unwrap(), version);
}
