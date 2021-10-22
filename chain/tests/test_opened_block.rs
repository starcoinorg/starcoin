use anyhow::Result;
use crypto::keygen::KeyGen;
use logger::prelude::*;
use starcoin_account_api::AccountInfo;
use starcoin_chain::ChainReader;
use starcoin_config::NodeConfig;
use starcoin_executor::{
    build_transfer_from_association, build_transfer_txn, DEFAULT_EXPIRATION_TIME,
};
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::StateReaderExt;
use starcoin_types::{account_address, account_config, U256};
use std::{convert::TryInto, sync::Arc};

#[stest::test]
pub fn test_open_block() -> Result<()> {
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

    // check state changed
    {
        let account_reader = opened_block.state_reader();
        let account_balance = account_reader.get_balance(receiver)?;
        assert_eq!(account_balance, Some(50_000_000));

        let account_resource = account_reader.get_account_resource(receiver)?.unwrap();
        assert_eq!(account_resource.sequence_number(), 0);
    }

    debug!("init gas_used: {}", opened_block.gas_used());
    let initial_gas_used = opened_block.gas_used();

    let build_transfer_txn = |seq_number: u64| {
        let (_prikey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        let address = account_address::from_public_key(&pubkey);
        build_transfer_txn(
            receiver,
            address,
            seq_number,
            10_000,
            1,
            1_000_000,
            config.net().time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
            config.net().chain_id(),
        )
        .sign(&receive_prikey, receive_public_key.clone())
        .unwrap()
        .into_inner()
    };

    // pre-run a txn to get gas_used
    // transferring to an non-exists account uses about 30w gas.
    let transfer_txn_gas = {
        let txn = build_transfer_txn(0);
        let excluded = opened_block.push_txns(vec![txn])?;
        assert_eq!(excluded.discarded_txns.len(), 0);
        assert_eq!(excluded.untouched_txns.len(), 0);
        opened_block.gas_used() - initial_gas_used
    };

    // check include txns
    let gas_left = opened_block.gas_left();
    let max_include_txn_num: u64 = gas_left / transfer_txn_gas;
    {
        let user_txns = (0u64..(max_include_txn_num + 1))
            .map(|idx| build_transfer_txn(idx + 1))
            .collect::<Vec<_>>();

        assert_eq!(max_include_txn_num + 1, user_txns.len() as u64);

        let excluded_txns = opened_block.push_txns(user_txns)?;
        assert_eq!(excluded_txns.untouched_txns.len(), 1);
        assert_eq!(excluded_txns.discarded_txns.len(), 0);
    }

    Ok(())
}
