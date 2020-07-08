address 0x1 {

module Block {
    use 0x1::Event;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

     resource struct BlockMetadata {
          // Height of the current block
          height: u64,
          // Hash of the parent block.
          parent_hash: vector<u8>,
          // Author of the current block.
          author: address,
          // Handle where events with the time of new blocks are emitted
          new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    struct NewBlockEvent {
          height: u64,
          author: address,
          timestamp: u64,
    }

    // This can only be invoked by the GENESIS_ACCOUNT at genesis
    public fun initialize(account: &signer, parent_hash: vector<u8>) {
      assert(Timestamp::is_genesis(), 1);
      assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 1);

      move_to<BlockMetadata>(
          account,
      BlockMetadata {
        height: 0,
        parent_hash: parent_hash,
        author: CoreAddresses::GENESIS_ACCOUNT(),
        new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
      });
    }

    // Get the current block height
    public fun get_current_block_height(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).height
    }

    // Get the hash of the parent block.
    public fun get_parent_hash(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).parent_hash
    }

    // Gets the address of the author of the current block
    public fun get_current_author(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT()).author
    }

    // Call at block prologue
    public fun process_block_metadata(account: &signer, parent_hash: vector<u8>,author: address, timestamp: u64): u64 acquires BlockMetadata{
        assert(Signer::address_of(account) == CoreAddresses::GENESIS_ACCOUNT(), 33);

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::GENESIS_ACCOUNT());
        let new_height = block_metadata_ref.height + 1;
        block_metadata_ref.height = new_height;
        block_metadata_ref.author= author;
        block_metadata_ref.parent_hash = parent_hash;

        Event::emit_event<NewBlockEvent>(
          &mut block_metadata_ref.new_block_events,
          NewBlockEvent {
            height: new_height,
            author: author,
            timestamp: timestamp,
          }
        );
        new_height
    }
}
}