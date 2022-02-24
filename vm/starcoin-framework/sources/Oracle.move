address StarcoinFramework {
module Oracle {
    use StarcoinFramework::Event;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Signer;
    use StarcoinFramework::Vector;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Errors;
    use StarcoinFramework::Account;
    use StarcoinFramework::GenesisSignerCapability;

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

    struct OracleUpdateEvent<phantom OracleT: copy+store+drop, ValueT: copy+store+drop> has copy,store,drop {
        source_id: u64,
        record: DataRecord<ValueT>,
    }

    struct DataSource<phantom OracleT: copy+store+drop, ValueT: copy+store+drop> has key {
        /// the id of data source of ValueT
        id: u64,
        /// the data version counter.
        counter: u64,
        update_events: Event::EventHandle<OracleUpdateEvent<OracleT, ValueT>>,
    }

    struct UpdateCapability<phantom OracleT: copy+store+drop> has store, key {
        account: address,
    }

    struct GenesisSignerCapability has key{
        cap: Account::SignerCapability,
    }

    /// The oracle type not register.
    const ERR_ORACLE_TYPE_NOT_REGISTER:u64 = 101;
    /// No capability to update the oracle value.
    const ERR_NO_UPDATE_CAPABILITY: u64 = 102;
    const ERR_NO_DATA_SOURCE: u64 = 103;
    const ERR_CAPABILITY_ACCOUNT_MISS_MATCH: u64 = 104;

    /// deprecated.
    public fun initialize(_sender: &signer) {
    }

    /// Used in v7->v8 upgrade. struct `GenesisSignerCapability` is deprecated, in favor of module `StarcoinFramework::GenesisSignerCapability`.
    public fun extract_signer_cap(signer: &signer): Account::SignerCapability acquires GenesisSignerCapability{
        CoreAddresses::assert_genesis_address(signer);
        let cap = move_from<GenesisSignerCapability>(Signer::address_of(signer));
        let GenesisSignerCapability {cap} = cap;
        cap
    }

    /// Register `OracleT` as an oracle type.
    public fun register_oracle<OracleT: copy+store+drop, Info: copy+store+drop>(_sender: &signer, info: Info) {
        let genesis_account = GenesisSignerCapability::get_genesis_signer();
        move_to(&genesis_account, OracleInfo<OracleT, Info> {
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
    public fun init_data_source<OracleT:  copy+store+drop, Info: copy+store+drop, ValueT: copy+store+drop>(sender: &signer, init_value: ValueT) acquires OracleInfo{
        assert!(exists<OracleInfo<OracleT, Info>>(CoreAddresses::GENESIS_ADDRESS()), Errors::not_published(ERR_ORACLE_TYPE_NOT_REGISTER));
        let oracle_info = borrow_global_mut<OracleInfo<OracleT, Info>>(CoreAddresses::GENESIS_ADDRESS());
        let now = Timestamp::now_milliseconds();
        move_to(sender, OracleFeed<OracleT, ValueT> {
            record: DataRecord<ValueT> {
                version: 0,
                value: init_value,
                updated_at: now,
            }
        });
        let sender_addr = Signer::address_of(sender);
        move_to(sender, DataSource<OracleT, ValueT> {
            id: oracle_info.counter,
            counter: 1,
            update_events: Event::new_event_handle<OracleUpdateEvent<OracleT, ValueT>>(sender),
        });
        move_to(sender, UpdateCapability<OracleT>{account: sender_addr});
        oracle_info.counter = oracle_info.counter + 1;
    }

    /// Check the DataSource<OracleT,ValueT> is initiailzed at ds_addr
    public fun is_data_source_initialized<OracleT:  copy+store+drop, ValueT: copy+store+drop>(ds_addr: address): bool {
        exists<DataSource<OracleT, ValueT>>(ds_addr)
    }

    /// Update Oracle's record with new value, the `sender` must have UpdateCapability<OracleT>
    public fun update<OracleT: copy+store+drop, ValueT: copy+store+drop>(sender: &signer, value: ValueT) acquires UpdateCapability, DataSource, OracleFeed{
        let account = Signer::address_of(sender);
        assert!(exists<UpdateCapability<OracleT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        let cap = borrow_global_mut<UpdateCapability<OracleT>>(account);
        update_with_cap(cap,value);
    }

    /// Update Oracle's record with new value and UpdateCapability<OracleT>
    public fun update_with_cap<OracleT: copy+store+drop, ValueT: copy+store+drop>(cap: &mut UpdateCapability<OracleT>, value: ValueT) acquires DataSource,OracleFeed  {
        let account = cap.account;
        assert!(exists<DataSource<OracleT, ValueT>>(account), Errors::requires_capability(ERR_NO_DATA_SOURCE));
        let source = borrow_global_mut<DataSource<OracleT, ValueT>>(account);
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

    /// Read the Oracle's value from `ds_addr`
    public fun read<OracleT:copy+store+drop, ValueT: copy+store+drop>(ds_addr: address): ValueT acquires OracleFeed{
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(ds_addr);
        *&oracle_feed.record.value
    }

    /// Read the Oracle's DataRecord from `ds_addr`
    public fun read_record<OracleT:copy+store+drop, ValueT: copy+store+drop>(ds_addr: address): DataRecord<ValueT> acquires OracleFeed{
        let oracle_feed = borrow_global<OracleFeed<OracleT, ValueT>>(ds_addr);
        *&oracle_feed.record
    }

    /// Batch read Oracle's DataRecord from `ds_addrs`
    public fun read_records<OracleT:copy+store+drop,  ValueT: copy+store+drop>(ds_addrs: &vector<address>): vector<DataRecord<ValueT>> acquires OracleFeed{
        let len = Vector::length(ds_addrs);
        let results = Vector::empty();
        let i = 0;
        while (i < len){
            let addr = *Vector::borrow(ds_addrs, i);
            let record = Self::read_record<OracleT, ValueT>(addr);
            Vector::push_back(&mut results, record);
            i = i + 1;
        };
        results
    }

    /// Remove UpdateCapability from current sender.
    public fun remove_update_capability<OracleT:copy+store+drop>(sender: &signer):UpdateCapability<OracleT> acquires UpdateCapability{
        let account = Signer::address_of(sender);
        assert!(exists<UpdateCapability<OracleT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdateCapability<OracleT>>(account)
    }

    /// Add UpdateCapability to current sender
    public fun add_update_capability<OracleT:copy+store+drop>(sender: &signer, update_cap: UpdateCapability<OracleT>){
        assert!(Signer::address_of(sender) == update_cap.account, Errors::invalid_argument(ERR_CAPABILITY_ACCOUNT_MISS_MATCH));
        move_to(sender, update_cap);
    }

    /// Unpack Record to fields: version, oracle, updated_at.
    public fun unpack_record<ValueT: copy+store+drop>(record: DataRecord<ValueT>):(u64, ValueT, u64) {
        (record.version,*&record.value,record.updated_at)
    }
}
module PriceOracle{
    use StarcoinFramework::Math;
    use StarcoinFramework::Oracle::{Self, DataRecord, UpdateCapability};

    struct PriceOracleInfo has copy,store,drop{
        scaling_factor: u128,
    }

    public fun register_oracle<OracleT: copy+store+drop>(sender: &signer, precision: u8){
        let scaling_factor = Math::pow(10, (precision as u64));
        Oracle::register_oracle<OracleT, PriceOracleInfo>(sender, PriceOracleInfo{
            scaling_factor,
        });
    }

    public fun init_data_source<OracleT: copy+store+drop>(sender: &signer, init_value: u128){
        Oracle::init_data_source<OracleT, PriceOracleInfo, u128>(sender, init_value);
    }

    public fun is_data_source_initialized<OracleT:  copy+store+drop>(ds_addr: address): bool{
        Oracle::is_data_source_initialized<OracleT, u128>(ds_addr)
    }

    public fun get_scaling_factor<OracleT: copy + store + drop>() : u128 {
        let info = Oracle::get_oracle_info<OracleT, PriceOracleInfo>();
        info.scaling_factor
    }

    public fun update<OracleT: copy+store+drop>(sender: &signer, value: u128){
        Oracle::update<OracleT, u128>(sender, value);
    }

    public fun update_with_cap<OracleT: copy+store+drop>(cap: &mut UpdateCapability<OracleT>, value: u128) {
        Oracle::update_with_cap<OracleT, u128>(cap, value);
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

module STCUSDOracle{
    use StarcoinFramework::Oracle::{DataRecord};
    use StarcoinFramework::PriceOracle::{Self};

    /// The STC to USD price oracle
    struct STCUSD has copy,store,drop {}

    public fun register(sender: &signer){
        PriceOracle::register_oracle<STCUSD>(sender, 6);
    }

    public fun read(ds_addr: address) : u128{
        PriceOracle::read<STCUSD>(ds_addr)
    }

    public fun read_record(ds_addr: address): DataRecord<u128>{
        PriceOracle::read_record<STCUSD>(ds_addr)
    }

    public fun read_records(ds_addrs: &vector<address>): vector<DataRecord<u128>>{
        PriceOracle::read_records<STCUSD>(ds_addrs)
    }
}

module PriceOracleScripts{
    use StarcoinFramework::PriceOracle;

    public(script) fun register_oracle<OracleT: copy+store+drop>(sender: signer, precision: u8){
        PriceOracle::register_oracle<OracleT>(&sender, precision)
    }

    public(script) fun init_data_source<OracleT: copy+store+drop>(sender: signer, init_value: u128){
        PriceOracle::init_data_source<OracleT>(&sender, init_value);
    }

    public(script) fun update<OracleT: copy+store+drop>(sender: signer, value: u128){
        PriceOracle::update<OracleT>(&sender, value);
    }
}

module PriceOracleAggregator{
    use StarcoinFramework::Vector;
    use StarcoinFramework::Oracle;
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::Math;
    use StarcoinFramework::Timestamp;
    use StarcoinFramework::Errors;

    /// No price data match requirement condition.
    const ERR_NO_PRICE_DATA_AVIABLE:u64 = 101;

    /// Get latest price from datasources and calculate avg.
    /// `addrs`: the datasource's addr, `updated_in`: the datasource should updated in x millseoconds.
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
        assert!(!Vector::is_empty(&prices), Errors::invalid_state(ERR_NO_PRICE_DATA_AVIABLE));
        Math::avg(&prices)
    }
}


}