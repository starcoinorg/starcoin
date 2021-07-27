use anyhow::Result;
use starcoin_crypto::HashValue;
use starcoin_types::block_metadata::BlockMetadata;
use starcoin_vm_types::account_config::BlockRewardEvent;
use test_helper::executor::expect_decode_event;
use test_helper::{
    dao::empty_txn_payload,
    executor::{
        account_execute_with_output, blockmeta_execute, current_block_number, prepare_genesis,
    },
    Account,
};

#[stest::test]
fn test_block_reward() -> Result<()> {
    // NOTICE: in test network, reward delay is 1.
    let alice = Account::new();
    let bob = Account::new();
    let (chain_state, net) = prepare_genesis();
    let one_day: u64 = 60 * 60 * 24 * 1000;
    // alice mint a block
    {
        let block_number = current_block_number(&chain_state) + 1;
        let block_timestamp = net.time_service().now_millis() + one_day * block_number - 1;
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
    }

    // bob mint a block
    let gas_fees = {
        let block_number = current_block_number(&chain_state) + 1;
        let block_timestamp = net.time_service().now_millis() + one_day * block_number - 1;

        let output = blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *bob.address(),
                Some(bob.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;

        let block_reward_event = expect_decode_event::<BlockRewardEvent>(&output);
        assert_eq!(&block_reward_event.miner, alice.address());
        assert_eq!(block_reward_event.block_number, 1);
        assert_eq!(block_reward_event.gas_fees, 0);

        let output = account_execute_with_output(&alice, &chain_state, empty_txn_payload());
        output.gas_used()
    };
    // alice mint another block
    {
        let block_number = current_block_number(&chain_state) + 1;
        let block_timestamp = net.time_service().now_millis() + one_day * block_number - 1;
        let output = blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.auth_key()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let block_reward_event = expect_decode_event::<BlockRewardEvent>(&output);
        assert_eq!(&block_reward_event.miner, bob.address());
        assert_eq!(block_reward_event.block_number, 2);
        assert_eq!(block_reward_event.gas_fees, gas_fees as u128);
    }
    Ok(())
}
