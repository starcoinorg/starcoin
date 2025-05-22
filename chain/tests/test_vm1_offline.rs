use starcoin_chain::verifier::FullVerifier;
use starcoin_chain::BlockChain;
use starcoin_chain_api::{ChainReader, ChainWriter};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_config::{BuiltinNetworkID, ChainNetwork};
use starcoin_consensus::Consensus;
use starcoin_transaction_builder::{peer_to_peer_txn_sent_as_association, DEFAULT_EXPIRATION_TIME};
use starcoin_types::account_address::AccountAddress;
use starcoin_vm2_test_helper::executor::get_sequence_number;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::state_view::StateReaderExt;
use starcoin_vm_types::transaction::SignedUserTransaction;
use std::str::FromStr;
use test_helper::gen_blockchain_for_test;

#[stest::test]
fn test_vm2_only_chain() -> anyhow::Result<()> {
    let chain_name = "vm2-only-testnet".to_string();
    let net = ChainNetwork::new_custom(
        chain_name,
        123.into(),
        BuiltinNetworkID::Test.genesis_config().clone(),
        BuiltinNetworkID::Test.genesis_config2().clone(),
    )
    .unwrap();
    let miner_account = AccountAddress::random();

    let vm1_offline_height = vm1_offline_height(123.into());
    assert_eq!(vm1_offline_height, 2);

    let mut chain = gen_blockchain_for_test(&net)?;

    let receiver = "0xd0c5a06ae6100ce115cad1600fe59e96";

    // For block 1, it's ok to mix vm1 and vm2 txns
    {
        let vm1_txn = build_a_vm1_txn(&mut chain, &net, receiver)?;
        let vm2_txn = build_a_vm2_txn(&mut chain, &net, receiver)?;
        let block_1 = {
            let res = chain.create_block_template_simple_with_txns(
                miner_account,
                vec![vm1_txn.into(), vm2_txn.into()],
            )?;
            chain
                .consensus()
                .create_block(res.0, net.time_service().as_ref())?
        };
        chain.apply_with_verifier::<FullVerifier>(block_1)?;
    }

    // For block 2, vm1 txns are not allowed because vm1 is offline
    {
        let vm1_txn = build_a_vm1_txn(&mut chain, &net, receiver)?;
        let res = chain.create_block_template_simple_with_txns(miner_account, vec![vm1_txn.into()]);
        let Err(e) = res else {
            panic!("vm1 offline chain should not be able to create block with vm1 txn");
        };
        assert_eq!(
            e.to_string(),
            "vm2 already initialized, can not push vm1 txns any more"
        );
    }

    // For block 2, vm2 txns are allowed
    {
        let vm2_txn = build_a_vm2_txn(&mut chain, &net, receiver)?;
        let res =
            chain.create_block_template_simple_with_txns(miner_account, vec![vm2_txn.into()])?;
        let block_2 = chain
            .consensus()
            .create_block(res.0, net.time_service().as_ref())?;
        assert_eq!(block_2.header().number(), vm1_offline_height);
        chain.apply_with_verifier::<FullVerifier>(block_2)?;
    }

    // For block 3, only vm2 txns are allowed
    {
        let vm1_txn = build_a_vm1_txn(&mut chain, &net, receiver)?;
        let vm2_txn = build_a_vm2_txn(&mut chain, &net, receiver)?;
        let res = chain.create_block_template_simple_with_txns(
            miner_account,
            vec![vm1_txn.into(), vm2_txn.clone().into()],
        );
        let Err(e) = res else {
            panic!("vm1 offline chain should not be able to create block with vm1 txn");
        };
        assert_eq!(
            e.to_string(),
            "vm2 already initialized, can not push vm1 txns any more"
        );

        let res =
            chain.create_block_template_simple_with_txns(miner_account, vec![vm2_txn.into()])?;
        let block_3 = chain
            .consensus()
            .create_block(res.0, net.time_service().as_ref())?;
        assert_eq!(block_3.header().number(), vm1_offline_height + 1);
        chain.apply_with_verifier::<FullVerifier>(block_3)?;
    }

    Ok(())
}

fn build_a_vm1_txn(
    chain: &mut BlockChain,
    net: &ChainNetwork,
    receiver: &str,
) -> anyhow::Result<SignedUserTransaction> {
    let association_sequence_num = chain
        .chain_state_reader()
        .get_sequence_number(association_address())?;
    let receiver = AccountAddress::from_str(receiver)?;
    let txn = peer_to_peer_txn_sent_as_association(
        receiver,
        association_sequence_num,
        1,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net,
    );

    assert!(
        starcoin_executor::validate_transaction(chain.chain_state(), txn.clone(), None).is_none()
    );
    Ok(txn)
}

fn build_a_vm2_txn(
    chain: &mut BlockChain,
    net: &ChainNetwork,
    receiver: &str,
) -> anyhow::Result<SignedUserTransaction2> {
    use starcoin_vm2_test_helper::build_transfer_from_association;
    use starcoin_vm2_types::account_address::AccountAddress;
    use starcoin_vm2_vm_types::account_config::association_address;
    let association_sequence_num = get_sequence_number(association_address(), chain.chain_state2());
    let receiver = AccountAddress::from_str(receiver)?;
    build_transfer_from_association(
        receiver,
        association_sequence_num,
        1,
        net.time_service().now_secs() + DEFAULT_EXPIRATION_TIME,
        net,
    )
    .try_into()
}
