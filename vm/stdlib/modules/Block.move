address 0x1 {

module Block {
    use 0x1::Event;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::Consensus;
    use 0x1::ErrorCode;

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

    // This can only be invoked by the GENESIS_ACCOUNT at genesis
    public fun initialize(account: &signer, parent_hash: vector<u8>) {
      assert(Timestamp::is_genesis(), ErrorCode::ENOT_GENESIS());
      assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

      move_to<BlockMetadata>(
          account,
      BlockMetadata {
        number: 0,
        parent_hash: parent_hash,
        author: CoreAddresses::GENESIS_ADDRESS(),
        new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
      });
    }

    // Get the current block number
    public fun get_current_block_number(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number
    }

    // Get the hash of the parent block.
    public fun get_parent_hash(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).parent_hash
    }

    // Gets the address of the author of the current block
    public fun get_current_author(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).author
    }

    // Call at block prologue
    public fun process_block_metadata(account: &signer, parent_hash: vector<u8>,author: address, timestamp: u64, uncles:u64, number:u64): u128 acquires BlockMetadata{
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ADDRESS(), ErrorCode::ENOT_GENESIS_ACCOUNT());

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
        assert(number == (block_metadata_ref.number + 1), ErrorCode::EBLOCK_NUMBER_MISMATCH());
        block_metadata_ref.number = number;
        block_metadata_ref.author= author;
        block_metadata_ref.parent_hash = parent_hash;

        let reward = Consensus::adjust_epoch(account, number, timestamp, uncles);

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
}
}