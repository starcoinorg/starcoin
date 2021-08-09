//! account: ds1
//! account: ds2
//! account: alice

// check: EXECUTED

//! new-transaction
//! sender: ds1
address default = {{default}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        PriceOracle::init_data_source<STCUSD>(&signer, 100000);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        PriceOracle::init_data_source<STCUSD>(&signer, 110000);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
address alice = {{alice}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::{STCUSD};
    fun test_is_data_source_initiailzed(_signer: signer) {
        assert(!PriceOracle::is_data_source_initialized<STCUSD>(@alice), 997);
        assert(PriceOracle::is_data_source_initialized<STCUSD>(@ds1), 998);
        assert(PriceOracle::is_data_source_initialized<STCUSD>(@ds2), 999);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::{Self,STCUSD};
    fun test_read_price(_signer: signer) {
        assert(PriceOracle::read<STCUSD>(@ds1) == 100000, 1000);
        assert(STCUSDOracle::read(@ds1) == 100000, 2000);
        assert(PriceOracle::read<STCUSD>(@ds2) == 110000, 1001);
        assert(STCUSDOracle::read(@ds2) == 110000, 2001);
}
}

// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 1
//! block-time: 1000

//! new-transaction
//! sender: ds1
address default = {{default}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun update_price(signer: signer) {
        PriceOracle::update<STCUSD>(&signer, 200000);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun test_read_price(_signer: signer) {
        assert(PriceOracle::read<STCUSD>(@ds1) == 200000, 1002);
        assert(PriceOracle::read<STCUSD>(@ds2) == 110000, 1003);
    }
}

// check: EXECUTED

//! block-prologue
//! author: genesis
//! block-number: 2
//! block-time: 2000

//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::PriceOracleAggregator;
    use 0x1::STCUSDOracle::STCUSD;
    use 0x1::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        assert(PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 2000) == ((200000+110000)/2), 1004);
        assert(PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 1000) == 200000, 1005);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::PriceOracleAggregator;
    use 0x1::STCUSDOracle::STCUSD;
    use 0x1::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 100);
    }
}

// abort OracleAggregator::ERR_NO_PRICE_DATA_AVIABLE
// check: "Keep(ABORTED { code: 25857"

//! new-transaction
//! sender: default
address default = {{default}};
module default::DelegateOracleDS{
    use 0x1::Oracle::{Self,UpdateCapability};
    use 0x1::PriceOracle;

    struct DelegateUpdateOracleCapability<OracleT: copy+store+drop> has key{
        cap: UpdateCapability<OracleT>,
    }

    public fun delegate<OracleT: copy+store+drop>(signer: &signer){
        let cap = Oracle::remove_update_capability<OracleT>(signer);
        move_to(signer, DelegateUpdateOracleCapability{
            cap,
        });
    }
    //any one can update the ds in ds_addr
    public fun update_price_any<OracleT: copy+store+drop>(ds_addr: address, price: u128) acquires DelegateUpdateOracleCapability{
        let cap = borrow_global_mut<DelegateUpdateOracleCapability<OracleT>>(ds_addr);
        PriceOracle::update_with_cap(&mut cap.cap, price);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use default::DelegateOracleDS;
    use 0x1::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        DelegateOracleDS::delegate<STCUSD>(&signer);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun update_price(signer: signer) {
        PriceOracle::update<STCUSD>(&signer, 0);
    }
}
// ds2 can not update ds2 datasource price
// check: ABORTED

//! new-transaction
//! sender: alice
address default = {{default}};
script {
    use 0x1::STCUSDOracle::STCUSD;
    use default::DelegateOracleDS;
    fun update_price(_signer: signer) {
        DelegateOracleDS::update_price_any<STCUSD>(@{{ds2}}, 0);
    }
}
// alice can update ds2 datasource price by DelegateOracleDS
// check: EXECUTED

//! new-transaction
//! sender: alice
address default = {{default}};
address ds2 = {{ds2}};
script {
    use 0x1::PriceOracle;
    use 0x1::STCUSDOracle::STCUSD;
    fun test_read_price(_signer: signer) {
        assert(PriceOracle::read<STCUSD>(@ds2) == 0, 1004);
    }
}

// check: EXECUTED