module starcoin_framework::oracle_aggregator {

    use std::error;
    use std::vector;
    use starcoin_std::math_fixed;

    use starcoin_framework::oracle;
    use starcoin_framework::timestamp;
    use starcoin_framework::oracle_price;

    /// No price data match requirement condition.
    const ERR_NO_PRICE_DATA_AVIABLE: u64 = 101;

    /// Get latest price from datasources and calculate avg.
    /// `addrs`: the datasource's addr, `updated_in`: the datasource should updated in x millseoconds.
    public fun latest_price_average_aggregator<OracleT: copy+store+drop>(
        addrs: &vector<address>,
        updated_in: u64
    ): u128 {
        let len = vector::length(addrs);
        let price_records = oracle_price::read_records<OracleT>(addrs);
        let prices = vector::empty();
        let i = 0;
        let expect_updated_after = timestamp::now_milliseconds() - updated_in;
        while (i < len) {
            let record = vector::pop_back(&mut price_records);
            let (_version, price, updated_at) = oracle::unpack_record(record);
            if (updated_at >= expect_updated_after) {
                vector::push_back(&mut prices, price);
            };
            i = i + 1;
        };
        // if all price data not match the update_in filter, abort.
        assert!(!vector::is_empty(&prices), error::invalid_state(ERR_NO_PRICE_DATA_AVIABLE));
        math_fixed::avg(&prices)
    }
}