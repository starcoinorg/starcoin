use starcoin_config::NodeConfig;
use starcoin_force_upgrade::ForceUpgrade;
use std::sync::Arc;
use starcoin_chain_api::ChainReader;

#[stest::test]
pub fn test_force_upgrade() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let force_upgrade = ForceUpgrade::new(chain.info().chain_id(), 1);
    let (txns, txn_outputs) = force_upgrade.do_execute(chain.get_state_view())?;
    assert!(
        !txns.is_empty() || !txn_outputs.is_empty(),
        "Failed to execution"
    );
    Ok(())
}

#[stest::test]
pub fn test_force_upgrade_in_openblock() -> anyhow::Result<()> {
    Ok(())
}
