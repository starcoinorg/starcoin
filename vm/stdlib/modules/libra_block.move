address 0x0 {

module LibraBlock {
    use 0x0::Event;
    //use 0x0::LibraSystem;
    use 0x0::LibraTimestamp;
    use 0x0::Transaction;
    //use 0x0::TransactionFee;
    use 0x0::STC;
    use 0x0::Vector;
    use 0x0::LibraAccount;
    use 0x0::SubsidyConfig;

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
    public fun initialize_block_metadata() {
      // Only callable by the Association address
      Transaction::assert(Transaction::sender() == 0xA550C18, 1);

      move_to_sender<BlockMetadata>(BlockMetadata {
        height: 0,
        //TODO pass genesis id.
        id: Vector::empty(),
        proposer:0x0,
        new_block_events: Event::new_event_handle<Self::NewBlockEvent>(),
      });
    }

    // Set the metadata for the current block.
    // The runtime always runs this before executing the transactions in a block.
    // TODO: 1. Make this private, support other metadata
    //       2. Should the previous block votes be provided from BlockMetadata or should it come from the ValidatorSet
    //          Resource?
    public fun block_prologue(
        round: u64,
        timestamp: u64,
        id: vector<u8>,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        // Can only be invoked by LibraVM privilege.
        //Transaction::assert(Transaction::sender() == 0x0, 33);
        //TODO conform this address.
        Transaction::assert(Transaction::sender() == 0x6d696e74, 33);

        process_block_prologue(round, timestamp, id, previous_block_votes, proposer);

        // Currently distribute once per-block.
        // TODO: Once we have a better on-chain representation of epochs we will make this per-epoch.
        // TODO: Need to update this to allow per-currency transaction fee
        // distribution
        //TransactionFee::distribute_transaction_fees<LBR::T>();

        // TODO(valerini): call regular reconfiguration here LibraSystem2::update_all_validator_info()
    }

    // Update the BlockMetadata resource with the new blockmetada coming from the consensus.
    fun process_block_prologue(
        round: u64,
        timestamp: u64,
        id: vector<u8>,
        previous_block_votes: vector<address>,
        proposer: address
    ) acquires BlockMetadata {
        let block_metadata_ref = borrow_global_mut<BlockMetadata>(0xA550C18);

        // TODO: Figure out a story for errors in the system transactions.
        //if(proposer != 0x0) Transaction::assert(LibraSystem::is_validator(proposer), 5002);
        LibraTimestamp::update_global_time(proposer, timestamp);
        block_metadata_ref.height = block_metadata_ref.height + 1;
        block_metadata_ref.proposer = proposer;
        block_metadata_ref.id = id;
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
      borrow_global<BlockMetadata>(0xA550C18).height
    }

    // Get the current block id
    public fun get_current_block_id(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(0xA550C18).id
    }

    // Gets the address of the proposer of the current block
    public fun get_current_proposer(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(0xA550C18).proposer
    }

    resource struct SubsidyInfo {
            withdrawal_capability: LibraAccount::WithdrawalCapability,
            subsidy_height: u64,
            heights: vector<u64>,
            miners: vector<address>,
    }

    public fun initialize_subsidy_info() {
        Transaction::assert(Transaction::sender() == 0x6d696e74, 1);

        move_to_sender<SubsidyInfo>(SubsidyInfo {
            withdrawal_capability: LibraAccount::extract_sender_withdrawal_capability(),
            subsidy_height: 0,
            heights: Vector::empty(),
            miners: Vector::empty(),
        });
    }

    fun do_subsidy(auth_key_prefix: vector<u8>) acquires BlockMetadata, SubsidyInfo {
        let current_height = get_current_block_height();

        if (current_height > 0) {
            Transaction::assert(SubsidyConfig::right_conf(), 6001);
            let subsidy_info = borrow_global_mut<SubsidyInfo>(0x6d696e74);
            let len = Vector::length(&subsidy_info.heights);
            let miner_len = Vector::length(&subsidy_info.miners);
            Transaction::assert((current_height == (subsidy_info.subsidy_height + len + 1)), 6002);
            Transaction::assert((len <= SubsidyConfig::subsidy_delay()), 6003);
            Transaction::assert((len == miner_len), 6004);

            if (len == SubsidyConfig::subsidy_delay()) {//pay and remove
                let subsidy_height = *&subsidy_info.subsidy_height + 1;
                let first_height = *Vector::borrow(&subsidy_info.heights, 0);
                Transaction::assert((subsidy_height == first_height), 6005);

                let subsidy_coin = SubsidyConfig::subsidy_coin(subsidy_height);
                let subsidy_miner = *Vector::borrow(&subsidy_info.miners, 0);
                subsidy_info.subsidy_height = subsidy_height;
                if (subsidy_coin > 0) {
                    Transaction::assert(LibraAccount::exists(subsidy_miner), 6006);
                    let libra_coin = LibraAccount::withdraw_with_capability<STC::T>(&subsidy_info.withdrawal_capability, subsidy_coin);
                    LibraAccount::deposit<STC::T>(subsidy_miner, libra_coin);
                };
                Vector::remove(&mut subsidy_info.heights, 0);
                Vector::remove(&mut subsidy_info.miners, 0);
            };

            Vector::push_back(&mut subsidy_info.heights, current_height);
            let current_miner = get_current_proposer();
            if (!LibraAccount::exists(current_miner)) {
                Transaction::assert(!Vector::is_empty(&auth_key_prefix), 6007);
                LibraAccount::create_account<STC::T>(current_miner, auth_key_prefix);
            };
            Vector::push_back(&mut subsidy_info.miners, current_miner);
        };
    }

}

}
