use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{ensure, format_err};

use starcoin_move_compiler::move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use starcoin_state_api::{ChainStateWriter, StateView};
use starcoin_statedb::ChainStateDB;
use starcoin_types::{
    access_path::AccessPath,
    account::{Account, DEFAULT_MAX_GAS_AMOUNT},
    account_config::{genesis_address, ModuleUpgradeStrategy},
    transaction::{SignedUserTransaction, TransactionOutput},
};
use starcoin_vm_types::{
    account_config::STC_TOKEN_CODE_STR,
    genesis_config::ChainId,
    move_resource::MoveResource,
    state_store::state_key::StateKey,
    transaction::{Module, Package, RawUserTransaction, Transaction, TransactionPayload},
};

pub const FORCE_UPGRADE_BLOCK_NUM: u64 = 16000000;

const DEFAULT_PACKAGE_PATH: &str = "/Users/bobong/Codes/westar_labs/starcoin/StarcoinFramework.v0.1.0.blob";

fn load_package_from_file(mv_or_package_file: &Path) -> anyhow::Result<Package> {
    ensure!(
        mv_or_package_file.exists(),
        "file {:?} not exist",
        mv_or_package_file
    );
    let mut bytes = vec![];
    File::open(mv_or_package_file)?.read_to_end(&mut bytes)?;

    let package = if mv_or_package_file.extension().unwrap_or_default() == MOVE_COMPILED_EXTENSION {
        Package::new_with_module(Module::new(bytes))?
    } else {
        bcs_ext::from_bytes(&bytes).map_err(|e| {
            format_err!(
                "Decode Package failed {:?}, please ensure the file is a Package binary file.",
                e
            )
        })?
    };
    anyhow::Ok(package)
}

pub struct ForceUpgrade {
    net: ChainId,
    block_number: u64,
}

impl ForceUpgrade {
    pub fn new(net: ChainId, block_number: u64) -> Self {
        ForceUpgrade { net, block_number }
    }

    pub fn do_execute(
        &self,
        state_db: &ChainStateDB,
    ) -> anyhow::Result<(Vec<SignedUserTransaction>, Vec<TransactionOutput>)> {
        if self.block_number != FORCE_UPGRADE_BLOCK_NUM {
            return Ok((vec![], vec![]));
        };

        // Write upgrade strategy resource to 0
        let upgrade_strategy_path = AccessPath::resource_access_path(
            genesis_address(),
            ModuleUpgradeStrategy::struct_tag(),
        );

        let upgraded_strategy = 100;

        let before_strategy = state_db
            .get_state_value(&StateKey::AccessPath(upgrade_strategy_path.clone()))?
            .unwrap();
        assert_eq!(before_strategy[0], 1, "Checking the strategy not 1");

        state_db.set(&upgrade_strategy_path, vec![100])?;

        // Check state is OK
        let after_ret = state_db
            .get_state_value(&StateKey::AccessPath(upgrade_strategy_path.clone()))?
            .unwrap();
        assert_eq!(after_ret[0], upgraded_strategy, "Set to upgrade strategy failed!");

        let ret = self.deploy_package(state_db, &Account::new_association());

        // Revert to origin value
        state_db.set(&upgrade_strategy_path, before_strategy)?;

        ret
    }

    fn deploy_package(
        &self,
        state_view: &ChainStateDB,
        account: &Account,
    ) -> anyhow::Result<(Vec<SignedUserTransaction>, Vec<TransactionOutput>)> {
        let package = load_package_from_file(&PathBuf::from(DEFAULT_PACKAGE_PATH))?;
        let signed_transaction = account.sign_txn(RawUserTransaction::new(
            account.address().clone(),
            0,
            TransactionPayload::Package(package),
            DEFAULT_MAX_GAS_AMOUNT,
            1,
            3600,
            self.net,
            STC_TOKEN_CODE_STR.to_string(),
        ));
        let ret = starcoin_executor::execute_transactions(
            state_view,
            vec![Transaction::UserTransaction(signed_transaction.clone())],
            None,
        )?;
        assert_eq!(ret.len(), 1, "There is incorrect execution result");

        Ok((vec![signed_transaction], ret))
    }
}
