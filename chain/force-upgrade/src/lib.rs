use anyhow::format_err;

use starcoin_types::{
    account::{Account, DEFAULT_MAX_GAS_AMOUNT},
    transaction::SignedUserTransaction,
};

use include_dir::{include_dir, Dir};
use starcoin_vm_types::{
    account_config::STC_TOKEN_CODE_STR,
    genesis_config::ChainId,
    transaction::{Package, RawUserTransaction, TransactionPayload},
};

pub const FORCE_UPGRADE_PACKAGE: Dir = include_dir!("package");

pub struct ForceUpgrade;

impl ForceUpgrade {
    pub fn force_deploy_txn(
        account: Account,
        sequence_number: u64,
        net: ChainId,
    ) -> anyhow::Result<Vec<SignedUserTransaction>> {
        let package_file = "stdlib.blob".to_string();
        let package = FORCE_UPGRADE_PACKAGE
            .get_file(package_file.clone())
            .map(|file| {
                bcs_ext::from_bytes::<Package>(file.contents())
                    .expect("Decode package should success")
            })
            .ok_or_else(|| format_err!("Can not find upgrade package {}", package_file))?;

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