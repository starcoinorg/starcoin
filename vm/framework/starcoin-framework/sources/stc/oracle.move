module starcoin_framework::oracle {

    use std::error;
    use std::signer;
    use std::vector;

    use starcoin_framework::account;
    use starcoin_framework::event;
    use starcoin_framework::system_addresses;
    use starcoin_framework::timestamp;

    struct OracleInfo<phantom OracleT: copy+store+drop, Info: copy+store+drop> has key {
        ///The datasource counter
        counter: u64,
        ///Ext info
        info: Info,
    }

    struct DataRecord<ValueT: copy+store+drop> has copy, store, drop {
        ///The data version
        version: u64,
        ///The record value
        value: ValueT,
        ///Update timestamp millisecond
        updated_at: u64,
    }

    struct OracleFeed<phantom OracleT: copy+store+drop, ValueT: copy+store+drop> has key {
        record: DataRecord<ValueT>,
    }

    struct OracleUpdateEvent<phantom OracleT: copy+store+drop, ValueT: copy+store+drop> has copy, store, drop {
        source_id: u64,
        record: DataRecord<ValueT>,
    }

    struct DataSource<phantom OracleT: copy+store+drop, ValueT: copy+store+drop> has key {
        /// the id of data source of ValueT
        id: u64,
        /// the data version counter.
        counter: u64,
        update_events: event::EventHandle<OracleUpdateEvent<OracleT, ValueT>>,
    }

    struct UpdateCapability<phantom OracleT: copy+store+drop> has store, key {
        account: address,
    }

    struct GenesisSignerCapability has key {
        cap: account::SignerCapability,
    }

    /// The oracle type not register.
    const ERR_ORACLE_TYPE_NOT_REGISTER: u64 = 101;
    /// No capability to update the oracle value.
    const ERR_NO_UPDATE_CAPABILITY: u64 = 102;
    const ERR_NO_DATA_SOURCE: u64 = 103;
    const ERR_CAPABILITY_ACCOUNT_MISS_MATCH: u64 = 104;

    /// deprecated.
    public fun initialize(_sender: &signer) {}

    /// Used in v7->v8 upgrade. struct `GenesisSignerCapability` is deprecated, in favor of module `StarcoinFramework::GenesisSignerCapability`.
    public fun extract_signer_cap(sender: &signer): account::SignerCapability acquires GenesisSignerCapability {
        // CoreAddresses::assert_genesis_address(signer);
        system_addresses::assert_starcoin_framework(sender);
        let cap = move_from<GenesisSignerCapability>(signer::address_of(sender));
        let GenesisSignerCapability { cap } = cap;
        cap
    }

    /// Register `OracleT` as an oracle type.
    public fun register_oracle<OracleT: copy+store+drop, Info: copy+store+drop>(sender: &signer, info: Info) {
        // let genesis_account =
        //     reserved_accounts_signer::get_stored_signer(signer::address_of(sender));
        move_to(sender, OracleInfo<OracleT, Info> {
            counter: 0,
            info,
        });
    }

    /// Get the `OracleT` oracle's counter, the counter represent how many `OracleT` datasources
    public fun get_oracle_counter<OracleT: copy + store + drop, Info: copy + store + drop>(): u64 acquires OracleInfo {
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(system_addresses::get_starcoin_framework());
        oracle_info.counter
    }

    public fun get_oracle_info<OracleT: copy + store + drop, Info: copy + store + drop>(): Info acquires OracleInfo {
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(system_addresses::get_starcoin_framework());
        *&oracle_info.info
    }

    /// Init a data source for type `OracleT`
    public fun init_data_source<OracleT: copy+store+drop, Info: copy+store+drop, ValueT: copy+store+drop>(
        sender: &signer,
        init_value: ValueT
    ) acquires OracleInfo {
        assert!(
            exists<OracleInfo<OracleT, Info>>(system_addresses::get_starcoin_framework()),
            error::invalid_state(ERR_ORACLE_TYPE_NOT_REGISTER)
        );
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(system_addresses::get_starcoin_framework());
        let now = timestamp::now_milliseconds();
        move_to(sender, OracleFeed<OracleT, ValueT> {
            record: DataRecord<ValueT> {
                version: 0,
                value: init_value,
                updated_at: now,
            }
        });
        let sender_addr = signer::address_of(sender);
        move_to(sender, DataSource<OracleT, ValueT> {
            id: oracle_info.counter,
            counter: 1,
            update_events: account::new_event_handle<OracleUpdateEvent<OracleT, ValueT>>(sender),
        });
        move_to(sender, UpdateCapability<OracleT> { account: sender_addr });
        oracle_info.counter = oracle_info.counter + 1;
    }

    /// Check the DataSource<OracleT,ValueT> is initiailzed at ds_addr
    public fun is_data_source_initialized<OracleT: copy+store+drop, ValueT: copy+store+drop>(ds_addr: address): bool {
        exists<DataSource<OracleT, ValueT>>(ds_addr)
    }

    /// Update Oracle's record with new value, the `sender` must have UpdateCapability<OracleT>
    public fun update<OracleT: copy+store+drop, ValueT: copy+store+drop>(
        sender: &signer,
        value: ValueT
    ) acquires UpdateCapability, DataSource, OracleFeed {
        let account = signer::address_of(sender);
        assert!(exists<UpdateCapability<OracleT>>(account), error::resource_exhausted(ERR_NO_UPDATE_CAPABILITY));
        let cap = borrow_global_mut<UpdateCapability<OracleT>>(account);
        update_with_cap(cap, value);
    }

    /// Update Oracle's record with new value and UpdateCapability<OracleT>
    public fun update_with_cap<OracleT: copy+store+drop, ValueT: copy+store+drop>(
        cap: &mut UpdateCapability<OracleT>,
        value: ValueT
    ) acquires DataSource, OracleFeed {
        let account = cap.account;
        assert!(exists<DataSource<OracleT, ValueT>>(account), error::resource_exhausted(ERR_NO_DATA_SOURCE));
        let source = borrow_global_mut<DataSource<OracleT, ValueT>>(account);
        let now = timestamp::now_milliseconds();
        let oracle_feed = borrow_global_mut<OracleFeed<OracleT, ValueT>>(account);
        oracle_feed.record.version = source.counter;
        oracle_feed.record.value = value;
        oracle_feed.record.updated_at = now;
        source.counter = source.counter + 1;
        event::emit_event(&mut source.update_events, OracleUpdateEvent<OracleT, ValueT> {
            source_id: source.id,
            record: *&oracle_feed.record
        });
    }

    /// Read the Oracle's value from `ds_addr`
    public fun read<OracleT: copy+store+drop, ValueT: copy+store+drop>(ds_addr: address): ValueT acquires OracleFeed {
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(ds_addr);
        *&oracle_feed.record.value
    }

    /// Read the Oracle's DataRecord from `ds_addr`
    public fun read_record<OracleT: copy+store+drop, ValueT: copy+store+drop>(
        ds_addr: address
    ): DataRecord<ValueT> acquires OracleFeed {
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(ds_addr);
        *&oracle_feed.record
    }

    /// Batch read Oracle's DataRecord from `ds_addrs`
    public fun read_records<OracleT: copy+store+drop, ValueT: copy+store+drop>(
        ds_addrs: &vector<address>
    ): vector<DataRecord<ValueT>> acquires OracleFeed {
        let len = vector::length(ds_addrs);
        let results = vector::empty();
        let i = 0;
        while (i < len) {
            let addr = *vector::borrow(ds_addrs, i);
            let record = Self::read_record<OracleT, ValueT>(addr);
            vector::push_back(&mut results, record);
            i = i + 1;
        };
        results
    }

    /// Remove UpdateCapability from current sender.
    public fun remove_update_capability<OracleT: copy+store+drop>(
        sender: &signer
    ): UpdateCapability<OracleT> acquires UpdateCapability {
        let account = signer::address_of(sender);
        assert!(exists<UpdateCapability<OracleT>>(account), error::resource_exhausted(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdateCapability<OracleT>>(account)
    }

    /// Add UpdateCapability to current sender
    public fun add_update_capability<OracleT: copy+store+drop>(sender: &signer, update_cap: UpdateCapability<OracleT>) {
        assert!(
            signer::address_of(sender) == update_cap.account,
            error::invalid_argument(ERR_CAPABILITY_ACCOUNT_MISS_MATCH)
        );
        move_to(sender, update_cap);
    }

    /// Unpack Record to fields: version, oracle, updated_at.
    public fun unpack_record<ValueT: copy+store+drop>(record: DataRecord<ValueT>): (u64, ValueT, u64) {
        (record.version, *&record.value, record.updated_at)
    }
}
