use starcoin_account_api::AccountInfo;
use starcoin_chain_api::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_crypto::keygen::KeyGen;
use starcoin_open_block::OpenedBlock;
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::{account::Account, account_address, U256};
use starcoin_vm_types::{account_config, state_view::StateReaderExt};
use std::sync::Arc;

#[stest::test]
pub fn test_force_upgrade_in_openblock() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = chain.current_header().clone();

    let block_gas_limit = 10000000;

    let account_reader = chain.chain_state_reader();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;

    let mut opened_block = {
        let miner_account = AccountInfo::random();
        OpenedBlock::new(
            chain.get_storage(),
            header,
            block_gas_limit,
            miner_account.address,
            config.net().time_service().now_millis(),
            vec![],
            U256::from(0),
            chain.consensus(),
            None,
        )?
    };

    let (_receive_prikey, receive_public_key) = KeyGen::from_os_rng().generate_keypair();
    let receiver = account_address::from_public_key(&receive_public_key);
    let txn1 = build_transfer_from_association(
        receiver,
        association_sequence_num,
        50_000_000,
        config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        config.net(),
    )
    .try_into()?;

    let excluded = opened_block.push_txns(vec![txn1])?;
    assert_eq!(excluded.discarded_txns.len(), 0);
    assert_eq!(excluded.untouched_txns.len(), 0);

    opened_block.finalize()?;

    Ok(())
}
