address 0x1 {
module Oracle {
    use 0x1::Event;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::CoreAddresses;
    use 0x1::Errors;

    struct OracleInfo<OracleT: copy+store+drop, Info: copy+store+drop> has key {
        ///The datasource counter
        counter: u64,
        ///Ext info
        info: Info,
    }

    struct DataRecord<ValueT: copy+store+drop> has copy, store, drop {
        ///The data version
        version: u64,
        ///The price value
        value: ValueT,
        ///Update timestamp millisecond
        updated_at: u64,
    }

    struct OracleFeed<OracleT: copy+store+drop, ValueT: copy+store+drop> has key {
        record: DataRecord<ValueT>,
    }

    struct OracleUpdateEvent<OracleT: copy+store+drop, ValueT: copy+store+drop> has copy,store,drop {
        source_id: u64,
        record: DataRecord<ValueT>,
    }

    struct DataRecordSource<OracleT: copy+store+drop, ValueT: copy+store+drop> has key {
        /// the id of data source of ValueT
        id: u64,
        /// the data version counter.
        counter: u64,
        update_events: Event::EventHandle<OracleUpdateEvent<OracleT, ValueT>>,
    }

    struct UpdateOracleCapability<OracleT: copy+store+drop> has store, key {
        account: address,
    }

    /// No capability to update the oracle value.
    const ERR_NO_UPDATE_CAPABILITY: u64 = 101;
    const ERR_NO_DATA_SOURCE: u64 = 102;

    /// Register `OracleT` as price oracle.
    public fun register_oracle<OracleT: copy+store+drop, Info: copy+store+drop>(signer: &signer, info: Info){
        //TODO implement a global register by contact account.
        CoreAddresses::assert_genesis_address(signer);
        move_to(signer, OracleInfo<OracleT, Info> {
           counter: 0,
            info,
        });
    }

    /// Get the `OracleT` oracle's counter, the counter represent how many `OracleT` datasources
    public fun get_oracle_counter<OracleT: copy + store + drop, Info: copy + store + drop>() : u64  acquires OracleInfo {
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(CoreAddresses::GENESIS_ADDRESS());
        oracle_info.counter
    }

    public fun get_oracle_info<OracleT: copy + store + drop, Info: copy + store + drop>() : Info  acquires OracleInfo {
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(CoreAddresses::GENESIS_ADDRESS());
        *&oracle_info.info
    }

    /// Init a data source for type `OracleT`
    public fun init_data_source<OracleT:  copy+store+drop, Info: copy+store+drop, ValueT: copy+store+drop>(signer: &signer, init_value: ValueT) acquires OracleInfo{
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(CoreAddresses::GENESIS_ADDRESS());
        let now = Timestamp::now_milliseconds();
        move_to(signer, OracleFeed<OracleT, ValueT> {
            record: DataRecord<ValueT> {
                version: 0,
                value: init_value,
                updated_at: now,
            }
        });
        let account = Signer::address_of(signer);
        move_to(signer, DataRecordSource<OracleT, ValueT> {
            id: oracle_info.counter,
            counter: 1,
            update_events: Event::new_event_handle<OracleUpdateEvent<OracleT, ValueT>>(signer),
        });
        move_to(signer, UpdateOracleCapability<OracleT>{account: account});
        oracle_info.counter = oracle_info.counter + 1;
    }

    public fun update<OracleT: copy+store+drop, ValueT: copy+store+drop>(signer: &signer, value: ValueT) acquires UpdateOracleCapability, DataRecordSource, OracleFeed{
        let account = Signer::address_of(signer);
        assert(exists<UpdateOracleCapability<OracleT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        let cap = borrow_global_mut<UpdateOracleCapability<OracleT>>(account);
        update_by_cap(cap,value);
    }

    public fun update_by_cap<OracleT: copy+store+drop, ValueT: copy+store+drop>(cap: &mut UpdateOracleCapability<OracleT>, value: ValueT) acquires DataRecordSource,OracleFeed  {
        let account = cap.account;
        assert(exists<DataRecordSource<OracleT, ValueT>>(account), Errors::requires_capability(ERR_NO_DATA_SOURCE));
        let source = borrow_global_mut<DataRecordSource<OracleT, ValueT>>(account);
        let now = Timestamp::now_milliseconds();
        let oracle_feed = borrow_global_mut<OracleFeed<OracleT, ValueT>>(account);
        oracle_feed.record.version = source.counter;
        oracle_feed.record.value = value;
        oracle_feed.record.updated_at = now;
        source.counter = source.counter + 1;
        Event::emit_event(&mut source.update_events,OracleUpdateEvent<OracleT, ValueT>{
            source_id: source.id,
            record: *&oracle_feed.record
        });
    }

    public fun read<OracleT:copy+store+drop, ValueT: copy+store+drop>(addr: address): ValueT acquires OracleFeed{
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(addr);
        *&oracle_feed.record.value
    }

    public fun read_record<OracleT:copy+store+drop, ValueT: copy+store+drop>(addr: address): DataRecord<ValueT> acquires OracleFeed{
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(addr);
        *&oracle_feed.record
    }

    public fun read_records<OracleT:copy+store+drop,  ValueT: copy+store+drop>(addrs: &vector<address>): vector<DataRecord<ValueT>> acquires OracleFeed{
        let len = Vector::length(addrs);
        let results = Vector::empty();
        let i = 0;
        while (i < len){
            let addr = *Vector::borrow(addrs, i);
            let record = Self::read_record<OracleT, ValueT>(addr);
            Vector::push_back(&mut results, record);
            i = i + 1;
        };
        results
    }

    /// Remove UpdateOracleCapability from current signer.
    public fun remove_update_capability<OracleT:copy+store+drop>(signer: &signer):UpdateOracleCapability<OracleT> acquires UpdateOracleCapability{
        let account = Signer::address_of(signer);
        assert(exists<UpdateOracleCapability<OracleT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdateOracleCapability<OracleT>>(account)
    }

    /// Add UpdateOracleCapability to current signer
    public fun add_update_capability<OracleT:copy+store+drop>(signer: &signer, update_cap: UpdateOracleCapability<OracleT>){
        move_to(signer, update_cap);
    }

    /// Unpack Record to fields: version, oracle, updated_at.
    public fun unpack_record<ValueT: copy+store+drop>(record: DataRecord<ValueT>):(u64, ValueT, u64) {
        (record.version,*&record.value,record.updated_at)
    }
}
module PriceOracle{
    use 0x1::Math;
    use 0x1::Oracle::{Self, DataRecord, UpdateOracleCapability};

    struct PriceOracleInfo has copy,store,drop{
        scaling_factor: u128,
    }

    public fun register_oracle<OracleT: copy+store+drop>(signer: &signer, precision: u8){
        let scaling_factor = Math::pow(10, (precision as u64));
        Oracle::register_oracle<OracleT, PriceOracleInfo>(signer, PriceOracleInfo{
            scaling_factor,
        });
    }

    public fun init_data_source<OracleT: copy+store+drop>(signer: &signer, init_value: u128){
        Oracle::init_data_source<OracleT, PriceOracleInfo, u128>(signer, init_value);
    }

    public fun get_scaling_factor<OracleT: copy + store + drop>() : u128 {
        let info = Oracle::get_oracle_info<OracleT, PriceOracleInfo>();
        info.scaling_factor
    }

    public fun update<OracleT: copy+store+drop>(signer: &signer, value: u128){
        Oracle::update<OracleT, u128>(signer, value);
    }

    public fun update_by_cap<OracleT: copy+store+drop>(cap: &mut UpdateOracleCapability<OracleT>, value: u128) {
        Oracle::update_by_cap<OracleT, u128>(cap, value);
    }

    public fun read<OracleT: copy+store+drop>(addr: address) : u128{
        Oracle::read<OracleT, u128>(addr)
    }

    public fun read_record<OracleT:copy+store+drop>(addr: address): DataRecord<u128>{
        Oracle::read_record<OracleT, u128>(addr)
    }

    public fun read_records<OracleT:copy+store+drop>(addrs: &vector<address>): vector<DataRecord<u128>>{
        Oracle::read_records<OracleT, u128>(addrs)
    }

}

module PriceOracleAggregator{
    use 0x1::Vector;
    use 0x1::Oracle;
    use 0x1::PriceOracle;
    use 0x1::Math;
    use 0x1::Timestamp;
    use 0x1::Errors;

    /// No price data match requirement condition.
    const ERR_NO_PRICE_DATA_AVIABLE:u64 = 101;

    /// Get latest price from datasources and calculate avg.
    /// `addrs` the datasource's addr, `updated_in` the datasource should updated in x millseoconds.
    public fun latest_price_average_aggregator<OracleT: copy+store+drop>(addrs: &vector<address>, updated_in: u64): u128 {
        let len = Vector::length(addrs);
        let price_records = PriceOracle::read_records<OracleT>(addrs);
        let prices = Vector::empty();
        let i = 0;
        let expect_updated_after = Timestamp::now_milliseconds() - updated_in;
        while (i < len){
            let record = Vector::pop_back(&mut price_records);
            let (_version, price, updated_at) = Oracle::unpack_record(record);
            if (updated_at >= expect_updated_after) {
                Vector::push_back(&mut prices, price);
            };
            i = i + 1;
        };
        // if all price data not match the update_in filter, abort.
        assert(!Vector::is_empty(&prices), Errors::invalid_state(ERR_NO_PRICE_DATA_AVIABLE));
        Math::avg(&prices)
    }
}

}