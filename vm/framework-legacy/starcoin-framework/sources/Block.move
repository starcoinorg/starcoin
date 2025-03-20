address StarcoinFramework {
/// Block module provide metadata for generated blocks.
module Block {
    use StarcoinFramework::Event;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Signer;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Errors;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict = true;
    }

    /// Block metadata struct.
    struct BlockMetadata has key {
        /// number of the current block
        number: u64,
        /// Hash of the parent block.
        parent_hash: vector<u8>,
        /// Author of the current block.
        author: address,
        /// number of uncles.
        uncles: u64,
        /// Handle of events when new blocks are emitted
        new_block_events: Event::EventHandle<Self::NewBlockEvent>,
    }

    /// Events emitted when new block generated.
    struct NewBlockEvent has drop, store {
        number: u64,
        author: address,
        timestamp: u64,
        uncles: u64,
    }

    const EBLOCK_NUMBER_MISMATCH: u64 = 17;

    /// This can only be invoked by the GENESIS_ACCOUNT at genesis
    public fun initialize(account: &signer, parent_hash: vector<u8>) {
        Timestamp::assert_genesis();
        CoreAddresses::assert_genesis_address(account);

        move_to<BlockMetadata>(
            account,
            BlockMetadata {
                number: 0,
                parent_hash: parent_hash,
                author: CoreAddresses::GENESIS_ADDRESS(),
                uncles: 0,
                new_block_events: Event::new_event_handle<Self::NewBlockEvent>(account),
            });
    }

    spec initialize {
        aborts_if !Timestamp::is_genesis();
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<BlockMetadata>(Signer::address_of(account));
    }

    /// Get the current block number
    public fun get_current_block_number(): u64 acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).number
    }

    spec get_current_block_number {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Get the hash of the parent block.
    public fun get_parent_hash(): vector<u8> acquires BlockMetadata {
      *&borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).parent_hash
    }

    spec get_parent_hash {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Gets the address of the author of the current block
    public fun get_current_author(): address acquires BlockMetadata {
      borrow_global<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS()).author
    }

    spec get_current_author {
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
    }

    /// Call at block prologue
    public fun process_block_metadata(account: &signer, parent_hash: vector<u8>,author: address, timestamp: u64, uncles:u64, number:u64) acquires BlockMetadata{
        CoreAddresses::assert_genesis_address(account);

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
        assert!(number == (block_metadata_ref.number + 1), Errors::invalid_argument(EBLOCK_NUMBER_MISMATCH));
        block_metadata_ref.number = number;
        block_metadata_ref.author= author;
        block_metadata_ref.parent_hash = parent_hash;
        block_metadata_ref.uncles = uncles;

        Event::emit_event<NewBlockEvent>(
          &mut block_metadata_ref.new_block_events,
          NewBlockEvent {
              number: number,
              author: author,
              timestamp: timestamp,
              uncles: uncles,
          }
        );
    }

    spec process_block_metadata {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if !exists<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS());
        aborts_if number != global<BlockMetadata>(CoreAddresses::SPEC_GENESIS_ADDRESS()).number + 1;
    }

    spec schema AbortsIfBlockMetadataNotExist {
        aborts_if !exists<BlockMetadata>(CoreAddresses::GENESIS_ADDRESS());
    }
}
}