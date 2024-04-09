use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{ensure, format_err};

use starcoin_move_compiler::move_command_line_common::files::MOVE_COMPILED_EXTENSION;
use starcoin_types::{
    account::{Account, DEFAULT_MAX_GAS_AMOUNT},
    transaction::SignedUserTransaction,
};

use starcoin_vm_types::{
    account_config::STC_TOKEN_CODE_STR,
    genesis_config::ChainId,
    transaction::{Module, Package, RawUserTransaction, TransactionPayload},
};

const DEFAULT_PACKAGE_PATH: &str = "stdlib-v12.blob";

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

pub struct ForceUpgrade;

impl ForceUpgrade {
    pub fn force_deploy_txn(
        account: Account,
        sequence_number: u64,
        net: ChainId,
    ) -> anyhow::Result<Vec<SignedUserTransaction>> {
        let package_path = if net.is_test() || net.is_dev() {
            let cur_dir = env::current_dir()?;
            cur_dir.join(format!("scripts/{}", DEFAULT_PACKAGE_PATH))
        } else {
            PathBuf::from(DEFAULT_PACKAGE_PATH)
        };
        let package = load_package_from_file(&package_path)?;

        let signed_transaction = account.sign_txn(RawUserTransaction::new(
            account.address().clone(),
            sequence_number,
            TransactionPayload::Package(package),
            DEFAULT_MAX_GAS_AMOUNT,
            1,
            3600,
            net,
            STC_TOKEN_CODE_STR.to_string(),
        ));
        Ok(vec![signed_transaction])
    }
}
