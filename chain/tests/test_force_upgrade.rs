use starcoin_chain_api::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_force_upgrade::{ForceUpgrade, FORCE_UPGRADE_BLOCK_NUM};
use starcoin_types::vm_error::KeptVMStatus;
use starcoin_vm_types::transaction::{Transaction, TransactionStatus};
use std::sync::Arc;

#[stest::test]
pub fn test_force_upgrade() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;

    let statedb = chain.get_state_view();
    let force_upgrade = ForceUpgrade::new(
        chain.info().chain_id(),
        FORCE_UPGRADE_BLOCK_NUM,
        &chain.get_state_view(),
    );
    force_upgrade.begin()?;

    let signed_txns = force_upgrade.deploy_package_txn()?;
    let txns: Vec<Transaction> = signed_txns.iter().cloned().map(Transaction::UserTransaction).collect();
    let txn_outupts = starcoin_executor::execute_transactions(&statedb, txns.clone(), None)?;
    assert!(
        !txns.is_empty() || !txn_outupts.is_empty(),
        "Failed to execution"
    );
    let txn_output = txn_outupts.get(0).unwrap();
    assert_eq!(
        txn_output.status(),
        &TransactionStatus::Keep(KeptVMStatus::Executed),
        "Execute the deploy failed"
    );
    assert!(
        !txn_output.write_set().is_empty(),
        "Execute the deploy failed"
    );
    Ok(())
}

#[stest::test]
pub fn test_force_upgrade_in_openblock() -> anyhow::Result<()> {
    Ok(())
}
