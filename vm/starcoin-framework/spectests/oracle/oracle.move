//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr ds1

//# faucet --addr ds2



//# run --signers ds1
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        PriceOracle::init_data_source<STCUSD>(&signer, 100000);
    }
}

// check: EXECUTED

//# run --signers ds2
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        PriceOracle::init_data_source<STCUSD>(&signer, 110000);
    }
}

// check: EXECUTED


//# run --signers alice
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::{STCUSD};
    fun test_is_data_source_initiailzed(_signer: signer) {
        assert!(!PriceOracle::is_data_source_initialized<STCUSD>(@alice), 997);
        assert!(PriceOracle::is_data_source_initialized<STCUSD>(@ds1), 998);
        assert!(PriceOracle::is_data_source_initialized<STCUSD>(@ds2), 999);
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::{Self,STCUSD};
    fun test_read_price(_signer: signer) {
        assert!(PriceOracle::read<STCUSD>(@ds1) == 100000, 1000);
        assert!(STCUSDOracle::read(@ds1) == 100000, 2000);
        assert!(PriceOracle::read<STCUSD>(@ds2) == 110000, 1001);
        assert!(STCUSDOracle::read(@ds2) == 110000, 2001);
}
}

// check: EXECUTED


//# block --author 0x1

//# run --signers ds1
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun update_price(signer: signer) {
        PriceOracle::update<STCUSD>(&signer, 200000);
    }
}


//# run --signers alice
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun test_read_price(_signer: signer) {
        assert!(PriceOracle::read<STCUSD>(@ds1) == 200000, 1002);
        assert!(PriceOracle::read<STCUSD>(@ds2) == 110000, 1003);
    }
}


//# block --author 0x1

//# run --signers alice
script {
    use StarcoinFramework::PriceOracleAggregator;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    use StarcoinFramework::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        assert!(PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 2000) == ((200000+110000)/2), 1004);
        assert!(PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 1000) == 200000, 1005);
    }
}

// check: EXECUTED


//# run --signers alice
script {
    use StarcoinFramework::PriceOracleAggregator;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    use StarcoinFramework::Vector;
    fun test_aggregator_price(_signer: signer) {
        let ds = Vector::empty();
        Vector::push_back(&mut ds, @ds1);
        Vector::push_back(&mut ds, @ds2);
        PriceOracleAggregator::latest_price_average_aggregator<STCUSD>(&ds, 100);
    }
}

// abort OracleAggregator::ERR_NO_PRICE_DATA_AVIABLE
// check: "Keep(ABORTED { code: 25857"

//# publish
module bob::DelegateOracleDS{
    use StarcoinFramework::Oracle::{Self,UpdateCapability};
    use StarcoinFramework::PriceOracle;

    struct DelegateUpdateOracleCapability<phantom OracleT: copy+store+drop> has key{
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

//# run --signers ds2
script {
    use bob::DelegateOracleDS;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun main(signer: signer) {
        DelegateOracleDS::delegate<STCUSD>(&signer);
    }
}

// check: EXECUTED


//# run --signers ds2
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun update_price(signer: signer) {
        PriceOracle::update<STCUSD>(&signer, 0);
    }
}
// ds2 can not update ds2 datasource price
// check: ABORTED

//# run --signers alice
script {
    use StarcoinFramework::STCUSDOracle::STCUSD;
    use bob::DelegateOracleDS;
    fun update_price(_signer: signer) {
        DelegateOracleDS::update_price_any<STCUSD>(@ds2, 0);
    }
}
// alice can update ds2 datasource price by DelegateOracleDS
// check: EXECUTED

//# run --signers alice
script {
    use StarcoinFramework::PriceOracle;
    use StarcoinFramework::STCUSDOracle::STCUSD;
    fun test_read_price(_signer: signer) {
        assert!(PriceOracle::read<STCUSD>(@ds2) == 0, 1004);
    }
}

// check: EXECUTED