//! account: ds1
//! account: ds2
//! account: alice

address default = {{default}};
module default::STCUSD {
    use 0x1::Oracle;
    struct STCUSD has copy,store,drop{}

    public fun init(account: &signer){
        Oracle::register_price_oracle<STCUSD>(account, 6, b"STC USD price");
    }
}
// check: EXECUTED

//! new-transaction
//! sender: genesis
address default = {{default}};
script {
    use default::STCUSD;
    fun main(signer: signer) {
        STCUSD::init(&signer)
    }
}

// check: EXECUTED

//! new-transaction
//! sender: ds1
address default = {{default}};
script {
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun main(signer: signer) {
        Oracle::init_price_data_source<STCUSD>(&signer, 100000);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun main(signer: signer) {
        Oracle::init_price_data_source<STCUSD>(&signer, 110000);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun test_read_price(_signer: signer) {
        assert(Oracle::read_price<STCUSD>(@ds1) == 100000, 1000);
        assert(Oracle::read_price<STCUSD>(@ds2) == 110000, 1001);
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
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun update_price(signer: signer) {
        Oracle::update_price<STCUSD>(&signer, 200000);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun test_read_price(_signer: signer) {
        assert(Oracle::read_price<STCUSD>(@ds1) == 200000, 1002);
        assert(Oracle::read_price<STCUSD>(@ds2) == 110000, 1003);
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
    use 0x1::OracleAggregator;
    use default::STCUSD::STCUSD;
    use 0x1::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        assert(OracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 2000) == ((200000+110000)/2), 1004);
        assert(OracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 1000) == 200000, 1005);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: alice
address default = {{default}};
address ds1 = {{ds1}};
address ds2 = {{ds2}};
script {
    use 0x1::OracleAggregator;
    use default::STCUSD::STCUSD;
    use 0x1::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        OracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 100);
    }
}

// abort OracleAggregator::ERR_NO_PRICE_DATA_AVIABLE
// check: "Keep(ABORTED { code: 25857"

//! new-transaction
//! sender: default
address default = {{default}};
module default::DelegateOracleDS{
    use 0x1::Oracle::{Self,UpdatePriceCapability};

    struct DelegateUpdatePriceCapability<DataT: copy+store+drop> has key{
        cap: UpdatePriceCapability<DataT>,
    }

    public fun delegate<DataT: copy+store+drop>(signer: &signer){
        let cap = Oracle::remove_update_price_capability<DataT>(signer);
        move_to(signer, DelegateUpdatePriceCapability{
            cap,
        });
    }
    //any one can update the ds in ds_addr
    public fun update_price_any<DataT: copy+store+drop>(ds_addr: address, price: u128) acquires DelegateUpdatePriceCapability{
        let cap = borrow_global_mut<DelegateUpdatePriceCapability<DataT>>(ds_addr);
        Oracle::update_price_by_cap(&mut cap.cap, price);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use default::DelegateOracleDS;
    use default::STCUSD::STCUSD;
    fun main(signer: signer) {
        DelegateOracleDS::delegate<STCUSD>(&signer);
    }
}

// check: EXECUTED


//! new-transaction
//! sender: ds2
address default = {{default}};
script {
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun update_price(signer: signer) {
        Oracle::update_price<STCUSD>(&signer, 0);
    }
}
// ds2 can not update ds2 datasource price
// check: ABORTED

//! new-transaction
//! sender: alice
address default = {{default}};
script {
    use default::STCUSD::STCUSD;
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
    use 0x1::Oracle;
    use default::STCUSD::STCUSD;
    fun test_read_price(_signer: signer) {
        assert(Oracle::read_price<STCUSD>(@ds2) == 0, 1004);
    }
}

// check: EXECUTED