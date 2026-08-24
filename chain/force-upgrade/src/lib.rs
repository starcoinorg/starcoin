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
use starcoin_vm_runtime::force_upgrade_management::{
    get_force_upgrade_block_number, is_legacy_main_force_upgrade,
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
    // block_timestamp: *NOTE* by seconds.
    //
    // This convenience entry point constructs the transaction for the
    // configured force-upgrade height. Call `force_deploy_txn_at` when the
    // caller already has the block number being executed.
    pub fn force_deploy_txn(
        account: Account,
        sequence_number: u64,
        block_timestamp_in_secs: u64,
        chain_id: &ChainId,
    ) -> anyhow::Result<SignedUserTransaction> {
        Self::force_deploy_txn_at(
            account,
            sequence_number,
            block_timestamp_in_secs,
            get_force_upgrade_block_number(chain_id),
            chain_id,
        )
    }

    pub fn force_deploy_txn_at(
        account: Account,
        sequence_number: u64,
        block_timestamp_in_secs: u64,
        block_number: u64,
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

        /* NOTICE also need modify fn  test_package_init_function */
        let init_script = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("StdlibUpgradeScripts").unwrap(),
            ),
            Identifier::new("upgrade_from_v11_to_v12").unwrap(),
            vec![],
            vec![
                bcs_ext::to_bytes(&23182155u64).unwrap(),
                bcs_ext::to_bytes(&16083000u64).unwrap(),
                bcs_ext::to_bytes(&5u64).unwrap(),
                bcs_ext::to_bytes(&1000u64).unwrap(),
            ],
        );

        assert_eq!(package.init_script().unwrap(), &init_script);

        let gas_unit_price = u64::from(is_legacy_main_force_upgrade(chain_id, block_number));

        Ok(account.sign_txn(RawUserTransaction::new(
            *account.address(),
            sequence_number,
            TransactionPayload::Package(package),
            DEFAULT_MAX_GAS_AMOUNT,
            gas_unit_price,
            block_timestamp_in_secs + DEFAULT_EXPIRATION_TIME,
            *chain_id,
            STC_TOKEN_CODE_STR.to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_crypto::HashValue;
    use starcoin_vm_runtime::force_upgrade_management::{
        get_force_upgrade_account, MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER,
    };

    #[test]
    fn main_historical_force_upgrade_txn_matches_canonical_hash() -> anyhow::Result<()> {
        let chain_id = ChainId::new(1);
        let block_timestamp_secs = 1_741_160_423;
        assert_eq!(MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER, 23_009_355);
        let txn = ForceUpgrade::force_deploy_txn_at(
            get_force_upgrade_account(&chain_id)?,
            158,
            block_timestamp_secs + DEFAULT_EXPIRATION_TIME,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER,
            &chain_id,
        )?;

        assert_eq!(txn.gas_unit_price(), 1);
        assert_eq!(txn.expiration_timestamp_secs(), 1_741_240_423);
        assert_eq!(
            txn.id(),
            HashValue::from_hex_literal(
                "0x935a709511febb849d8e396a84a743191518eeee95370332a8446ea2147fd47a",
            )?
        );
        Ok(())
    }

    #[test]
    fn main_force_upgrade_txn_is_gas_free_outside_historical_height() -> anyhow::Result<()> {
        let chain_id = ChainId::new(1);
        let txn = ForceUpgrade::force_deploy_txn_at(
            get_force_upgrade_account(&chain_id)?,
            158,
            1_741_160_423 + DEFAULT_EXPIRATION_TIME,
            MAIN_LEGACY_FORCE_UPGRADE_BLOCK_NUMBER + 1,
            &chain_id,
        )?;

        assert_eq!(txn.gas_unit_price(), 0);

        let halley = ChainId::new(253);
        let halley_txn = ForceUpgrade::force_deploy_txn(
            get_force_upgrade_account(&halley)?,
            0,
            1_741_160_423 + DEFAULT_EXPIRATION_TIME,
            &halley,
        )?;
        assert_eq!(halley_txn.gas_unit_price(), 0);
        Ok(())
    }
}
