use starcoin_vm2_test_helper::executor::prepare_genesis;
use starcoin_vm2_types::genesis_config::ChainNetwork;
use starcoin_vm2_types::state_view::StateReaderExt;

#[stest::test]
pub fn test_prepare_genesis() -> anyhow::Result<()> {
    let (statedb, network) = prepare_genesis()?;
    assert_eq!(network.chain_id(), ChainNetwork::new_test().chain_id());
    assert!(statedb.get_stc_info()?.total_value() > 0);
}
