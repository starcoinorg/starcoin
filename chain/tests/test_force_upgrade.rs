use starcoin_account_api::AccountInfo;
use starcoin_chain_api::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_crypto::keygen::KeyGen;
use starcoin_force_upgrade::ForceUpgrade;
use starcoin_open_block::OpenedBlock;
use starcoin_transaction_builder::{build_transfer_from_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account::Account;
use starcoin_types::{account_address, vm_error::KeptVMStatus, U256};
use starcoin_vm_runtime::force_upgrade_data_cache::FORCE_UPGRADE_BLOCK_NUMBER;
use starcoin_vm_types::{
    account_config,
    state_view::StateReaderExt,
    transaction::{Transaction, TransactionStatus},
};
use std::sync::Arc;

#[stest::test]
pub fn test_force_upgrade() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;

    let statedb = chain.get_state_view();
    let account = Account::new_association();
    let sequence_number = statedb.get_sequence_number(account.address().clone())?;

    let signed_txns = ForceUpgrade::begin(
        account,
        sequence_number,
        chain.info().chain_id(),
        FORCE_UPGRADE_BLOCK_NUMBER,
        statedb,
        statedb,
    )?;

    let txns: Vec<Transaction> = signed_txns
        .iter()
        .cloned()
        .map(Transaction::UserTransaction)
        .collect();

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

    ForceUpgrade::finish(statedb)?;

    Ok(())
}

#[stest::test]
pub fn test_force_upgrade_in_openblock() -> anyhow::Result<()> {
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test(config.net())?;
    let header = chain.current_header();
    let block_gas_limit = 10000000;

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

    let account_reader = chain.chain_state_reader();
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;
    let (receive_prikey, receive_public_key) = KeyGen::from_os_rng().generate_keypair();
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

    let txns = {
        let account = Account::new_association();
        ForceUpgrade::begin(
            account,
            association_sequence_num + 1,
            opened_block.chain_id(),
            FORCE_UPGRADE_BLOCK_NUM,
            opened_block.state_writer(),
            opened_block.state_reader(),
        )?
    };

    if !txns.is_empty() {
        let exclude_txns = opened_block.push_txns(txns)?;
        assert_eq!(exclude_txns.discarded_txns.len(), 0);
        assert_eq!(exclude_txns.untouched_txns.len(), 0);
    }

    {
        // Finished force upgrade
        ForceUpgrade::finish(opened_block.state_writer())?;
    }

    opened_block.finalize()?;

    Ok(())
}
