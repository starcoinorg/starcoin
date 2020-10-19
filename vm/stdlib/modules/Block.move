address 0x1 {

module Block {
    use 0x1::Event;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::ConsensusConfig;
    use 0x1::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict = true;
    }

     resource struct BlockMetadata {
          // number of the current block
          number: u64,
          // Hash of the parent block.
          parent_hash: vector<u8>,
          // Author of the current block.
          author: address,
          // Handle where events with the time of new blocks are emitted
          new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    struct NewBlockEvent {
          number: u64,
          author: address,
          timestamp: u64,
    }

    const EBLOCK_NUMBER_MISMATCH: u64 = 17; // do not change

    // This can only be invoked by the GENESIS_ACCOUNT at genesis
    public fun initialize(account: &signer, parent_hash: vector<u8>) {
      Timestamp::assert_genesis();
      CoreAddresses::assert_genesis_address(account);

      move_to<BlockMetadata>(
          account,
      BlockMetadata {
        number: 0,
        parent_hash: parent_hash,
        author: CoreAddresses::GENESIS_ADDRESS(),
        new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
      });
    }

    spec fun initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<BlockMetadata>(Signer::spec_address_of(account));
    }

    // Get the current block number
    public fun get_current_block_number(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number
    }

    spec fun get_current_block_number {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    // Get the hash of the parent block.
    public fun get_parent_hash(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).parent_hash
    }

    spec fun get_parent_hash {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    // Gets the address of the author of the current block
    public fun get_current_author(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).author
    }

    spec fun get_current_author {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    // Call at block prologue
    public fun process_block_metadata(account: &signer, parent_hash: vector<u8>,author: address, timestamp: u64, uncles:u64, number:u64, parent_gas_used:u64): u128 acquires BlockMetadata{
        CoreAddresses::assert_genesis_address(account);

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
        assert(number == (block_metadata_ref.number + 1), Errors::invalid_argument(EBLOCK_NUMBER_MISMATCH));
        block_metadata_ref.number = number;
        block_metadata_ref.author= author;
        block_metadata_ref.parent_hash = parent_hash;

        let reward = ConsensusConfig::adjust_epoch(account, number, timestamp, uncles, parent_gas_used);

        Event::emit_event<NewBlockEvent>(
          &mut block_metadata_ref.new_block_events,
          NewBlockEvent {
            number: number,
            author: author,
            timestamp: timestamp,
          }
        );
        reward
    }

    spec fun process_block_metadata {
        pragma verify = false;

        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if number != global<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS()).number + 1;
    }
}
}