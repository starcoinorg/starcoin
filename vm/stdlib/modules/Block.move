address 0x1 {

module Block {
    use 0x1::Event;
    //use 0x1::System;
    use 0x1::Timestamp;
    //use 0x1::TransactionFee;
    use 0x1::STC::{STC};
    use 0x1::Vector;
    use 0x1::Account;
    use 0x1::RewardConfig;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    spec module {
        pragma verify = false;
    }

    resource struct BlockMetadata {
      // Height of the current block
      // TODO: should we keep the height?
      height: u64,
      // Hash of the current block of transactions.
      id: vector<u8>,
      // TODO rename and add more filed.
      // Proposer of the current block.
      proposer: address,
      // Handle where events with the time of new blocks are emitted
      new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    struct NewBlockEvent {
      round: u64,
      proposer: address,
      previous_block_votes: vector<address>,

      // On-chain time during  he block at the given height
      time_microseconds: u64,
    }

    // This can only be invoked by the Association address, and only a single time.
    // Currently, it is invoked in the genesis transaction
    public fun initialize_block_metadata(account: &signer) {
      assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

      move_to<BlockMetadata>(
          account,
      BlockMetadata {
        height: 0,
        //TODO pass genesis id.
        id: Vector::empty(),
        proposer:0x0,
        new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
      });
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: 1. Make this private, support other metadata
    //       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    //          Resource?
    public fun block_prologue(
        account: &signer,
        round: u64,
        timestamp: u64,
        id: vector<u8>,
        previous_block_votes: vector<address>,
        proposer: address,
        auth_key_prefix: vector<u8>
    ) acquires BlockMetadata,RewardInfo {
        // Can only be invoked by LibraVM privilege.
        //assert(Signer::address_of(account) == 0x0, 33);
        //TODO conform this address.
        assert(Signer::address_of(account) == 0x6d696e74, 33);

        process_block_prologue(account, round, timestamp, id, previous_block_votes, proposer, auth_key_prefix);

        // Currently distribute once per-block.
        // TODO: Once we have a better on-chain representation of epochs we will make this per-epoch.
        // TODO: Need to update this to allow per-currency transaction fee
        // distribution
        //TransactionFee::distribute_transaction_fees();

        // TODO(valerini): call regular reconfiguration here System2::update_all_validator_info()
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        account: &signer,
        round: u64,
        timestamp: u64,
        id: vector<u8>,
        previous_block_votes: vector<address>,
        proposer: address,
        auth_key_prefix: vector<u8>
    ) acquires BlockMetadata, RewardInfo {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT());

        // TODO: Figure out a story for errors in the system transactions.
        //if(proposer != 0x0) assert(System::is_validator(proposer), 5002);
        Timestamp::update_global_time(account, proposer, timestamp);

        let new_height = block_metadata_ref.height + 1;
        block_metadata_ref.height = new_height;
        block_metadata_ref.proposer = proposer;
        block_metadata_ref.id = id;

        do_reward(account, new_height, proposer, auth_key_prefix);
        Event::emit_event<NewBlockEvent>(
          &mut block_metadata_ref.new_block_events,
          NewBlockEvent {
            round: round,
            proposer: proposer,
            previous_block_votes: previous_block_votes,
            time_microseconds: timestamp,
          }
        );
    }

    // Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).height
    }

    // Get the current block id
    public fun get_current_block_id(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).id
    }

    // Gets the address of the proposer of the current block
    public fun get_current_proposer(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).proposer
    }

    resource struct RewardInfo {
            withdrawal_capability: Account::WithdrawCapability,
            reward_height: u64,
            heights: vector<u64>,
            miners: vector<address>,
    }

    public fun initialize_reward_info(account: &signer) {
        //TODO omit this account requirement.
        assert(Signer::address_of(account) == CoreAddresses::MINT_ADDRESS(), 1);

        move_to<RewardInfo>(account, RewardInfo {
            withdrawal_capability: Account::extract_withdraw_capability(account),
            reward_height: 0,
            heights: Vector::empty(),
            miners: Vector::empty(),
        });
    }

    fun do_reward(account: &signer, current_height: u64, current_miner: address, auth_key_prefix: vector<u8>) acquires RewardInfo {

        if (current_height > 0) {
            let reward_info = borrow_global_mut<RewardInfo>(0x6d696e74);
            let len = Vector::length(&reward_info.heights);
            let miner_len = Vector::length(&reward_info.miners);
            assert((current_height == (reward_info.reward_height + len + 1)), 6002);
            assert((len <= RewardConfig::reward_delay()), 6003);
            assert((len == miner_len), 6004);

            if (len == RewardConfig::reward_delay()) {//pay and remove
                let reward_height = *&reward_info.reward_height + 1;
                let first_height = *Vector::borrow(&reward_info.heights, 0);
                assert((reward_height == first_height), 6005);

                let reward_coin = RewardConfig::reward_coin(reward_height);
                let reward_miner = *Vector::borrow(&reward_info.miners, 0);
                reward_info.reward_height = reward_height;
                if (reward_coin > 0) {
                    assert(Account::exists_at(reward_miner), 6006);
                    let libra_coin = Account::withdraw_with_capability<STC>(&reward_info.withdrawal_capability, reward_coin);
                    Account::deposit<STC>(account, reward_miner, libra_coin);
                };
                Vector::remove(&mut reward_info.heights, 0);
                Vector::remove(&mut reward_info.miners, 0);
            };

            Vector::push_back(&mut reward_info.heights, current_height);
            if (!Account::exists_at(current_miner)) {
                assert(!Vector::is_empty(&auth_key_prefix), 6007);
                Account::create_account<STC>(current_miner, auth_key_prefix);
            };
            Vector::push_back(&mut reward_info.miners, current_miner);
        };
    }

}

}
