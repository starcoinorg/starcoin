// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2

use anyhow::format_err;
use starcoin_types::account::DEFAULT_EXPIRATION_TIME;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_types::{
    account::{Account, DEFAULT_MAX_GAS_AMOUNT},
    transaction::SignedUserTransaction,
};
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::transaction::ScriptFunction;
use starcoin_vm_types::{
    account_config::STC_TOKEN_CODE_STR,
    genesis_config::ChainId,
    transaction::{Package, RawUserTransaction, TransactionPayload},
};
use stdlib::COMPILED_MOVE_CODE_DIR;

pub struct ForceUpgrade;

impl ForceUpgrade {
    // block_timestamp: *NOTE* by seconds,
    pub fn force_deploy_txn(
        account: Account,
        sequence_number: u64,
        block_timestamp_in_secs: u64,
        chain_id: &ChainId,
    ) -> anyhow::Result<SignedUserTransaction> {
        let package_file = "12/11-12/stdlib.blob".to_string();
        let package = COMPILED_MOVE_CODE_DIR
            .get_file(package_file.clone())
            .map(|file| {
                bcs_ext::from_bytes::<Package>(file.contents())
                    .expect("Decode package should success")
            })
            .ok_or_else(|| format_err!("Can not find upgrade package {}", package_file))?;

        let init_script = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("StdlibUpgradeScripts").unwrap(),
            ),
            Identifier::new("upgrade_from_v11_to_v12").unwrap(),
            vec![],
            vec![],
        );

        assert_eq!(package.init_script().unwrap(), &init_script);

        Ok(account.sign_txn(RawUserTransaction::new(
            *account.address(),
            sequence_number,
            TransactionPayload::Package(package),
            DEFAULT_MAX_GAS_AMOUNT,
            1,
            block_timestamp_in_secs + DEFAULT_EXPIRATION_TIME,
            *chain_id,
            STC_TOKEN_CODE_STR.to_string(),
        )))
    }
}
