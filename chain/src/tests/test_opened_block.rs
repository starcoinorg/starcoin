use crate::test_helper;
use anyhow::Result;
use config::NodeConfig;
use consensus::dev::DevConsensus;
use crypto::keygen::KeyGen;
use logger::prelude::*;
use starcoin_open_block::OpenedBlock;
use starcoin_state_api::AccountStateReader;
use starcoin_wallet_api::WalletAccount;
use std::{convert::TryInto, sync::Arc};
use traits::ChainReader;
use types::{account_address, account_config, transaction::authenticator::AuthenticationKey};

#[stest::test]
pub fn test_open_block() -> Result<()> {
    // uncomment this to see debug message.
    // let _ = logger::init_for_test();
    let config = Arc::new(NodeConfig::random_for_test());
    let chain = test_helper::gen_blockchain_for_test::<DevConsensus>(config)?;
    let header = chain.current_header();
    let block_gas_limit = 10000;

    let mut opened_block = {
        let miner_account = WalletAccount::random();
        OpenedBlock::new(
            chain.get_storage(),
            header,
            block_gas_limit,
            miner_account.address,
            Some(miner_account.get_auth_key().prefix().to_vec()),
            vec![],
        )?
    };

    let account_reader = AccountStateReader::new(chain.chain_state_reader());
    let association_sequence_num =
        account_reader.get_sequence_number(account_config::association_address())?;
    let (sender_prikey, sender_pubkey) = KeyGen::from_os_rng().generate_keypair();
    let sender = account_address::from_public_key(&sender_pubkey);
    let txn1 = executor::build_transfer_from_association(
        sender,
        AuthenticationKey::ed25519(&sender_pubkey).prefix().to_vec(),
        association_sequence_num,
        50_000_000,
    )
    .try_into()?;
    let excluded = opened_block.push_txns(vec![txn1])?;
    assert_eq!(excluded.discarded_txns.len(), 0);
    assert_eq!(excluded.untouched_txns.len(), 0);

    // check state changed
    {
        let account_reader = AccountStateReader::new(opened_block.state_reader());
        let account_balance = account_reader.get_balance(&sender)?;
        assert_eq!(account_balance, Some(50_000_000));

        let account_resource = account_reader.get_account_resource(&sender)?.unwrap();
        assert_eq!(account_resource.sequence_number(), 0);
    }

    debug!("init gas_used: {}", opened_block.gas_used());
    let initial_gas_used = opened_block.gas_used();

    let build_transfer_txn = |seq_number: u64| {
        let (_prikey, pubkey) = KeyGen::from_os_rng().generate_keypair();
        let address = account_address::from_public_key(&pubkey);
        executor::build_transfer_txn(
            sender,
            address,
            AuthenticationKey::ed25519(&pubkey).prefix().to_vec(),
            seq_number,
            10_000,
            1,
            1_000_000,
        )
        .sign(&sender_prikey, sender_pubkey.clone())
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
