/// Block module provide metadata for generated blocks.
module starcoin_framework::stc_block {
    use std::error;
    use std::signer;
    use std::vector;

    use starcoin_framework::epoch;
    use starcoin_framework::block_reward;
    use starcoin_framework::timestamp;
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::stc_transaction_fee;

    use starcoin_framework::chain_id;
    use starcoin_framework::account;
    use starcoin_framework::system_addresses;
    use starcoin_framework::event;

    const EPROLOGUE_BAD_CHAIN_ID: u64 = 6;

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
        new_block_events: event::EventHandle<Self::NewBlockEvent>,
        /// An Array of the parents hash for a Dag block.
        parents_hash: vector<u8>,
    }

    /// Events emitted when new block generated.
    struct NewBlockEvent has drop, store {
        number: u64,
        author: address,
        timestamp: u64,
        uncles: u64,
        parents_hash: vector<u8>,
    }

    const EBLOCK_NUMBER_MISMATCH: u64 = 17;
    const ERROR_NO_HAVE_CHECKPOINT: u64 = 18;
    const ERROR_NOT_BLOCK_HEADER: u64 = 19;
    const ERROR_INTERVAL_TOO_LITTLE: u64 = 20;

    const CHECKPOINT_LENGTH: u64 = 60;
    const BLOCK_HEADER_LENGTH: u64 = 247;
    const BLOCK_INTERVAL_NUMBER: u64 = 5;

    /// This can only be invoked by the GENESIS_ACCOUNT at genesis
    public fun initialize(account: &signer, parent_hash: vector<u8>) {
        // Timestamp::assert_genesis();
        system_addresses::assert_starcoin_framework(account);

        move_to<BlockMetadata>(
            account,
            BlockMetadata {
                number: 0,
                parent_hash,
                author: system_addresses::get_starcoin_framework(),
                uncles: 0,
                parents_hash: vector::empty<u8>(),
                new_block_events: account::new_event_handle<Self::NewBlockEvent>(account),
            });
    }

    spec initialize {
        use std::signer;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<BlockMetadata>(signer::address_of(account));
    }

    /// Get the current block number
    public fun get_current_block_number(): u64 acquires BlockMetadata {
        borrow_global<BlockMetadata>(system_addresses::get_starcoin_framework()).number
    }

    /// Get the hash of the parent block.
    public fun get_parent_hash(): vector<u8> acquires BlockMetadata {
        *&borrow_global<BlockMetadata>(system_addresses::get_starcoin_framework()).parent_hash
    }

    spec get_parent_hash {
        aborts_if !exists<BlockMetadata>(system_addresses::get_starcoin_framework());
    }

    /// Gets the address of the author of the current block
    public fun get_current_author(): address acquires BlockMetadata {
        borrow_global<BlockMetadata>(system_addresses::get_starcoin_framework()).author
    }

    /// Set the metadata for the current block and distribute transaction fees and block rewards.
    /// The runtime always runs this before executing the transactions in a block.
    public fun block_prologue(
        account: signer,
        parent_hash: vector<u8>,
        timestamp: u64,
        author: address,
        auth_key_vec: vector<u8>,
        uncles: u64,
        number: u64,
        chain_id: u8,
        parent_gas_used: u64,
        parents_hash: vector<u8>,
    ) acquires BlockMetadata {
        debug::print(&std::string::utf8(b"stc_block::block_prologue | Entered"));

        // Can only be invoked by genesis account
        system_addresses::assert_starcoin_framework(&account);

        // Check that the chain ID stored on-chain matches the chain ID
        // specified by the transaction
        assert!(chain_id::get() == chain_id, error::invalid_argument(EPROLOGUE_BAD_CHAIN_ID));

        // deal with previous block first.
        let txn_fee = stc_transaction_fee::distribute_transaction_fees<STC>(&account);

        debug::print(&std::string::utf8(b"stc_block::block_prologue | txn_fee"));
        debug::print(&coin::value(&txn_fee));

        // then deal with current block.
        timestamp::update_global_time(&account, signer::address_of(&account), timestamp * 1000);

        process_block_metadata(
            &account,
            parent_hash,
            author,
            timestamp,
            uncles,
            number,
            parents_hash,
        );

        let reward = epoch::adjust_epoch(&account, number, timestamp, uncles, parent_gas_used);

        debug::print(&std::string::utf8(b"stc_block::block_prologue | timestamp::update_global_time"));
        debug::print(&timestamp);

        // pass in previous block gas fees.
        block_reward::process_block_reward(&account, number, reward, author, auth_key_vec, txn_fee);

        debug::print(&std::string::utf8(b"stc_block::block_prologue | Exited"));
    }

    /// Call at block prologue
    fun process_block_metadata(
        account: &signer,
        parent_hash: vector<u8>,
        author: address,
        timestamp: u64,
        uncles: u64,
        number: u64,
        parents_hash: vector<u8>,
    ) acquires BlockMetadata {

        debug::print(&std::string::utf8(b"stc_block::process_block_metadata | Entered"));

        system_addresses::assert_starcoin_framework(account);

        let block_metadata_ref = borrow_global_mut<BlockMetadata>(system_addresses::get_starcoin_framework());

        debug::print(&std::string::utf8(b"stc_block::process_block_metadata | to check block number"));

        assert!(number == (block_metadata_ref.number + 1), error::invalid_argument(EBLOCK_NUMBER_MISMATCH));
        block_metadata_ref.number = number;
        block_metadata_ref.author = author;
        block_metadata_ref.parent_hash = parent_hash;
        block_metadata_ref.uncles = uncles;
        block_metadata_ref.parents_hash = parents_hash;

        debug::print(&std::string::utf8(b"stc_block::process_block_metadata | to emit NewBlockEvent  "));

        event::emit_event<NewBlockEvent>(
            &mut block_metadata_ref.new_block_events,
            NewBlockEvent {
                number,
                author,
                timestamp,
                parents_hash,
                uncles,
            }
        );
        debug::print(&std::string::utf8(b"stc_block::process_block_metadata | Exited"));
    }

    #[test]
    use starcoin_framework::bcs_util;
    #[test]
    use std::hash;
    #[test]
    use std::option;
    use starcoin_framework::coin;
    use starcoin_std::debug;

    #[test]
    fun test_header() {
        // Block header Unit test
        // Use Main Genesis BlockHeader in Rust
        // BlockHeader {
        //  id: Some(HashValue(0x80848150abee7e9a3bfe9542a019eb0b8b01f124b63b011f9c338fdb935c417d)),
        //  parent_hash: HashValue(0xb82a2c11f2df62bf87c2933d0281e5fe47ea94d5f0049eec1485b682df29529a),
        //  timestamp: 1621311100863,
        //  number: 0,
        //  author: 0x00000000000000000000000000000001,
        //  author_auth_key: None,
        //  txn_accumulator_root: HashValue(0x43609d52fdf8e4a253c62dfe127d33c77e1fb4afdefb306d46ec42e21b9103ae),
        //  block_accumulator_root: HashValue(0x414343554d554c41544f525f504c414345484f4c4445525f4841534800000000),
        //  state_root: HashValue(0x61125a3ab755b993d72accfea741f8537104db8e022098154f3a66d5c23e828d),
        //  gas_used: 0,
        //  difficulty: 11660343,
        //  body_hash: HashValue(0x7564db97ee270a6c1f2f73fbf517dc0777a6119b7460b7eae2890d1ce504537b),
        //  chain_id: ChainId { id: 1 },
        //  nonce: 0,
        //  extra: BlockHeaderExtra([0, 0, 0, 0])
        // }
        // Blockheader BCS : 20b82a2c11f2df62bf87c2933d0281e5fe47ea94d5f0049eec1485b682df29529abf17ac7d79010000000000000000000000000000000000000000000000000001002043609d52fdf8e4a253c62dfe127d33c77e1fb4afdefb306d46ec42e21b9103ae20414343554d554c41544f525f504c414345484f4c4445525f48415348000000002061125a3ab755b993d72accfea741f8537104db8e022098154f3a66d5c23e828d00000000000000000000000000000000000000000000000000000000000000000000000000b1ec37207564db97ee270a6c1f2f73fbf517dc0777a6119b7460b7eae2890d1ce504537b010000000000000000

        let prefix = hash::sha3_256(b"STARCOIN::BlockHeader");
        let header = x"20b82a2c11f2df62bf87c2933d0281e5fe47ea94d5f0049eec1485b682df29529abf17ac7d79010000000000000000000000000000000000000000000000000001002043609d52fdf8e4a253c62dfe127d33c77e1fb4afdefb306d46ec42e21b9103ae20414343554d554c41544f525f504c414345484f4c4445525f48415348000000002061125a3ab755b993d72accfea741f8537104db8e022098154f3a66d5c23e828d00000000000000000000000000000000000000000000000000000000000000000000000000b1ec37207564db97ee270a6c1f2f73fbf517dc0777a6119b7460b7eae2890d1ce504537b010000000000000000";
        let (_parent_hash, new_offset) = bcs_util::deserialize_bytes(&header, 0);
        let (_timestamp, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (number, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (_author, new_offset) = bcs_util::deserialize_address(&header, new_offset);
        let (_author_auth_key, new_offset) = bcs_util::deserialize_option_bytes(&header, new_offset);
        let (_txn_accumulator_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_block_accumulator_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (state_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_gas_used, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (_difficultyfirst, new_offset) = bcs_util::deserialize_u128(&header, new_offset);
        let (_difficultylast, new_offset) = bcs_util::deserialize_u128(&header, new_offset);
        let (_body_hash, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_chain_id, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_nonce, new_offset) = bcs_util::deserialize_u32(&header, new_offset);
        let (_extra1, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra2, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra3, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra4, _new_offset) = bcs_util::deserialize_u8(&header, new_offset);

        vector::append(&mut prefix, header);
        let block_hash = hash::sha3_256(prefix);
        assert!(block_hash == x"80848150abee7e9a3bfe9542a019eb0b8b01f124b63b011f9c338fdb935c417d", 1001);
        assert!(number == 0, 1002);
        assert!(state_root == x"61125a3ab755b993d72accfea741f8537104db8e022098154f3a66d5c23e828d", 1003);
    }

    #[test]
    fun test_header2() {
        // Block header Unit test
        // Use BlockHeader in integration test
        //"number":"2",
        //"block_hash":"0x9433bb7b56333dfc33e012f3b22b67277a3026448eb5043747d59284f648343d"
        //"parent_hash":"0x9be97e678afa8a0a4cf9ca612be6f64810a6f7d5f8b4b4ddf5e4971ef4b5eb48"
        //"state_root":"0xd2df4c8c579f9e05b0adf14b53785379fb245465d703834eb19fba74d9114a9a"
        //"header":"0x209be97e678afa8a0a4cf9ca612be6f64810a6f7d5f8b4b4ddf5e4971ef4b5eb4820aa26050000000002000000000000000000000000000000000000000000000200205c79e9493845327132ab3011c7c6c9d8bddcfde5553abb90cf5ef7fdfb39a4aa20c4d8e6cdb52520794dad5241f51a4eed46a5e5264dd148032cc3bdb8e3bdbe7a20d2df4c8c579f9e05b0adf14b53785379fb245465d703834eb19fba74d9114a9a0000000000000000000000000000000000000000000000000000000000000000000000000000000020c01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97fe0000000000000000"

        let prefix = hash::sha3_256(b"STARCOIN::BlockHeader");
        let header = x"209be97e678afa8a0a4cf9ca612be6f64810a6f7d5f8b4b4ddf5e4971ef4b5eb4820aa26050000000002000000000000000000000000000000000000000000000200205c79e9493845327132ab3011c7c6c9d8bddcfde5553abb90cf5ef7fdfb39a4aa20c4d8e6cdb52520794dad5241f51a4eed46a5e5264dd148032cc3bdb8e3bdbe7a20d2df4c8c579f9e05b0adf14b53785379fb245465d703834eb19fba74d9114a9a0000000000000000000000000000000000000000000000000000000000000000000000000000000020c01e0329de6d899348a8ef4bd51db56175b3fa0988e57c3dcec8eaf13a164d97fe0000000000000000";
        let (_parent_hash, new_offset) = bcs_util::deserialize_bytes(&header, 0);
        let (_timestamp, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (number, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (_author, new_offset) = bcs_util::deserialize_address(&header, new_offset);
        let (_author_auth_key, new_offset) = bcs_util::deserialize_option_bytes(&header, new_offset);
        let (_txn_accumulator_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_block_accumulator_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (state_root, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_gas_used, new_offset) = bcs_util::deserialize_u64(&header, new_offset);
        let (_difficultyfirst, new_offset) = bcs_util::deserialize_u128(&header, new_offset);
        let (_difficultylast, new_offset) = bcs_util::deserialize_u128(&header, new_offset);
        let (_body_hash, new_offset) = bcs_util::deserialize_bytes(&header, new_offset);
        let (_chain_id, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_nonce, new_offset) = bcs_util::deserialize_u32(&header, new_offset);
        let (_extra1, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra2, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra3, new_offset) = bcs_util::deserialize_u8(&header, new_offset);
        let (_extra4, _new_offset) = bcs_util::deserialize_u8(&header, new_offset);

        vector::append(&mut prefix, header);
        let block_hash = hash::sha3_256(prefix);
        assert!(block_hash == x"9433bb7b56333dfc33e012f3b22b67277a3026448eb5043747d59284f648343d", 1001);
        assert!(number == 2, 1002);
        assert!(state_root == x"d2df4c8c579f9e05b0adf14b53785379fb245465d703834eb19fba74d9114a9a", 1003);
    }
}