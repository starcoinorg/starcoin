address 0x1 {
module Oracle {
    use 0x1::Event;
    use 0x1::Timestamp;
    use 0x1::Signer;
    use 0x1::Vector;
    use 0x1::Math;
    use 0x1::CoreAddresses;
    use 0x1::Errors;

    struct PriceOracleInfo<DateT: copy+store+drop> has key {
        ///The datasource counter
        counter: u64,
        /// The scaling factor for the price
        scaling_factor: u128,
        ///Get the description of the data type
        description: vector<u8>,
    }

    struct PriceData has copy, store, drop {
        ///The data version
        version: u64,
        ///The price value
        price: u128,
        ///Update timestamp millisecond
        updated_at: u64,
    }

    struct PriceFeed<DataT: copy+store+drop> has key {
        value: PriceData,
    }

    struct PriceUpdateEvent<DateT: copy+store+drop> has copy,store,drop {
        source_id: u64,
        value: PriceData,
    }

    struct PriceDataSource<DataT: copy+store+drop> has key {
        /// the id of data source of DataT
        id: u64,
        /// the data version counter.
        counter: u64,
        update_events: Event::EventHandle<PriceUpdateEvent<DataT>>,
    }

    struct UpdatePriceCapability<DataT: copy+store+drop> has store, key {
        account: address,
    }

    /// No capability to update the oracle value.
    const ERR_NO_UPDATE_CAPABILITY: u64 = 101;
    const ERR_NO_DATA_SOURCE: u64 = 102;

    /// Register `DataT` as price oracle.
    public fun register_price_oracle<DataT: copy+store+drop>(signer: &signer, precision: u8, description: vector<u8>){
        //TODO implement a global register by contact account.
        CoreAddresses::assert_genesis_address(signer);
        move_to(signer, PriceOracleInfo<DataT> {
           counter: 0,
           scaling_factor: Math::pow(10, (precision as u64)),
           description,
        });
    }

    /// Get the `DataT` oracle's price scaling_factor
    public fun get_oracle_scaling_factor<DataT: copy + store + drop>() : u128  acquires PriceOracleInfo {
        let oracle_info = borrow_global_mut<PriceOracleInfo<DataT>>(CoreAddresses::GENESIS_ADDRESS());
        oracle_info.scaling_factor
    }

    /// Get the `DataT` oracle's counter, the counter represent how many `DataT` datasources
    public fun get_oracle_counter<DataT: copy + store + drop>() : u64  acquires PriceOracleInfo {
        let oracle_info = borrow_global_mut<PriceOracleInfo<DataT>>(CoreAddresses::GENESIS_ADDRESS());
        oracle_info.counter
    }

    /// Init a price data source for type `DataT`
    public fun init_price_data_source<DataT:  copy+store+drop>(signer: &signer, init_price: u128) acquires PriceOracleInfo{
        let oracle_info = borrow_global_mut<PriceOracleInfo<DataT>>(CoreAddresses::GENESIS_ADDRESS());
        let now = Timestamp::now_milliseconds();
        move_to(signer, PriceFeed<DataT> {
            value: PriceData {
                version: 0,
                price: init_price,
                updated_at: now,
            }
        });
        let account = Signer::address_of(signer);
        move_to(signer, PriceDataSource<DataT> {
            id: oracle_info.counter,
            counter: 1,
            update_events: Event::new_event_handle<PriceUpdateEvent<DataT>>(signer),
        });
        move_to(signer, UpdatePriceCapability<DataT>{account: account});
        oracle_info.counter = oracle_info.counter + 1;
    }


    public fun update_price_by_cap<DataT: copy+store+drop>(cap: &mut UpdatePriceCapability<DataT>, price: u128) acquires PriceDataSource,PriceFeed  {
        let account = cap.account;
        assert(exists<PriceDataSource<DataT>>(account), Errors::requires_capability(ERR_NO_DATA_SOURCE));
        let source = borrow_global_mut<PriceDataSource<DataT>>(account);
        let now = Timestamp::now_milliseconds();
        let price_feed = borrow_global_mut<PriceFeed<DataT>>(account);
        price_feed.value.version = source.counter;
        price_feed.value.price = price;
        price_feed.value.updated_at = now;
        source.counter = source.counter + 1;
        Event::emit_event(&mut source.update_events,PriceUpdateEvent<DataT>{
            source_id: source.id,
            value: *&price_feed.value
        });
    }

    public fun update_price<DataT: copy+store+drop>(signer: &signer, price: u128) acquires UpdatePriceCapability, PriceDataSource, PriceFeed{
        let account = Signer::address_of(signer);
        assert(exists<UpdatePriceCapability<DataT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        let cap = borrow_global_mut<UpdatePriceCapability<DataT>>(account);
        update_price_by_cap(cap,price);
    }

    public fun read_price<DataT:copy+store+drop>(addr: address): u128 acquires PriceFeed{
        let price_feed = borrow_global<PriceFeed<DataT>>(addr);
        *&price_feed.value.price
    }

    public fun read_price_data<DataT:copy+store+drop>(addr: address): PriceData acquires PriceFeed{
        let price_feed = borrow_global<PriceFeed<DataT>>(addr);
        *&price_feed.value
    }

    public fun read_price_data_batch<DataT:copy+store+drop>(addrs: &vector<address>): vector<PriceData> acquires PriceFeed{
        let len = Vector::length(addrs);
        let results = Vector::empty();
        let i = 0;
        while (i < len){
            let addr = *Vector::borrow(addrs, i);
            let data = Self::read_price_data<DataT>(addr);
            Vector::push_back(&mut results, data);
            i = i + 1;
        };
        results
    }

    /// Remove UpdatePriceCapability from current signer.
    public fun remove_update_price_capability<DataT:copy+store+drop>(signer: &signer):UpdatePriceCapability<DataT> acquires UpdatePriceCapability{
        let account = Signer::address_of(signer);
        assert(exists<UpdatePriceCapability<DataT>>(account), Errors::requires_capability(ERR_NO_UPDATE_CAPABILITY));
        move_from<UpdatePriceCapability<DataT>>(account)
    }

    /// Add UpdatePriceCapability to current signer
    public fun add_update_price_capability<DataT:copy+store+drop>(signer: &signer, update_cap: UpdatePriceCapability<DataT>){
        move_to(signer, update_cap);
    }

    public fun get_version(data: &PriceData): u64 {
        data.version
    }

    public fun get_price(data: &PriceData): u128 {
        data.price
    }

    public fun get_update_at(data: &PriceData): u64 {
        data.updated_at
    }
    /// Unpack data to fields: version, price, updated_at.
    public fun unpack_data(data: PriceData):(u64, u128, u64) {
        (data.version,data.price,data.updated_at)
    }
}
module OracleAggregator{
    use 0x1::Vector;
    use 0x1::Oracle;
    use 0x1::Math;
    use 0x1::Timestamp;
    use 0x1::Errors;

    /// No price data match requirement condition.
    const ERR_NO_PRICE_DATA_AVIABLE:u64 = 101;

    /// Get latest price from datasources and calculate avg.
    /// `addrs` the datasource's addr, `updated_in` the datasource should updated in x millseoconds.
    public fun latest_price_average_aggregator<DataT: copy+store+drop>(addrs: &vector<address>, updated_in: u64): u128 {
        let len = Vector::length(addrs);
        let price_data_vec = Oracle::read_price_data_batch<DataT>(addrs);
        let prices = Vector::empty();
        let i = 0;
        let expect_updated_after = Timestamp::now_milliseconds() - updated_in;
        while (i < len){
            let data = Vector::pop_back(&mut price_data_vec);
            let (_version, price, updated_at) = Oracle::unpack_data(data);
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