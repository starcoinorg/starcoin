use starcoin_chain_api::ChainReader;
use starcoin_config::{BuiltinNetworkID, ChainNetwork, NodeConfig};
use starcoin_force_upgrade::{ForceUpgrade, FORCE_UPGRADE_BLOCK_NUM};
use std::sync::Arc;

#[stest::test]
pub fn test_force_upgrade() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let force_upgrade = ForceUpgrade::new(chain.info().chain_id(), FORCE_UPGRADE_BLOCK_NUM);
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
