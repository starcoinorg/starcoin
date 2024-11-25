//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr ds1

//# faucet --addr ds2



//# run --signers ds1
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun main(account: signer) {
        oracle_price::init_data_source<STCUSD>(&account, 100000);
    }
}
// check: EXECUTED



//# run --signers ds2
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun main(account: signer) {
        oracle_price::init_data_source<STCUSD>(&account, 110000);
    }
}

// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::{STCUSD};

    fun test_is_data_source_initiailzed(_signer: signer) {
        assert!(!oracle_price::is_data_source_initialized<STCUSD>(@alice), 997);
        assert!(oracle_price::is_data_source_initialized<STCUSD>(@ds1), 998);
        assert!(oracle_price::is_data_source_initialized<STCUSD>(@ds2), 999);
    }
}

// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::{Self, STCUSD};

    fun test_read_price(_signer: signer) {
        assert!(oracle_price::read<STCUSD>(@ds1) == 100000, 1000);
        assert!(oracle_stc_usd::read(@ds1) == 100000, 2000);
        assert!(oracle_price::read<STCUSD>(@ds2) == 110000, 1001);
        assert!(oracle_stc_usd::read(@ds2) == 110000, 2001);
    }
}

// check: EXECUTED


//# block --author 0x1

//# run --signers ds1
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun update_price(account: signer) {
        oracle_price::update<STCUSD>(&account, 200000);
    }
}


//# run --signers alice
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun test_read_price(_signer: signer) {
        assert!(oracle_price::read<STCUSD>(@ds1) == 200000, 1002);
        assert!(oracle_price::read<STCUSD>(@ds2) == 110000, 1003);
    }
}


//# block --author 0x1

//# run --signers alice
script {
    use starcoin_framework::oracle_aggregator;
    use starcoin_framework::oracle_stc_usd::STCUSD;
    use std::vector;

    fun test_aggregator_price(_signer: signer) {
        let ds = vector::empty();
        vector::push_back(&mut ds, @ds1);
        vector::push_back(&mut ds, @ds2);
        assert!(
            oracle_aggregator::latest_price_average_aggregator<STCUSD>(&ds, 2000) == ((200000 + 110000) / 2),
            1004
        );
        assert!(oracle_aggregator::latest_price_average_aggregator<STCUSD>(&ds, 1000) == 200000, 1005);
    }
}

// check: EXECUTED


//# run --signers alice
script {
    use starcoin_framework::oracle_aggregator;
    use starcoin_framework::oracle_stc_usd::STCUSD;
    use std::vector;

    fun test_aggregator_price(_signer: signer) {
        let ds = vector::empty();
        vector::push_back(&mut ds, @ds1);
        vector::push_back(&mut ds, @ds2);
        oracle_aggregator::latest_price_average_aggregator<STCUSD>(&ds, 100);
    }
}

// abort OracleAggregator::ERR_NO_PRICE_DATA_AVIABLE
// check: "Keep(ABORTED { code: 25857"

//# publish
module bob::DelegateOracleDS {
    use starcoin_framework::oracle::{Self, UpdateCapability};
    use starcoin_framework::oracle_price;

    struct DelegateUpdateOracleCapability<phantom OracleT: copy+store+drop> has key {
        cap: UpdateCapability<OracleT>,
    }

    public fun delegate<OracleT: copy+store+drop>(account: &signer) {
        let cap = oracle::remove_update_capability<OracleT>(account);
        move_to(account, DelegateUpdateOracleCapability {
            cap,
        });
    }

    //any one can update the ds in ds_addr
    public fun update_price_any<OracleT: copy+store+drop>(
        ds_addr: address,
        price: u128
    ) acquires DelegateUpdateOracleCapability {
        let cap = borrow_global_mut<DelegateUpdateOracleCapability<OracleT>>(ds_addr);
        oracle_price::update_with_cap(&mut cap.cap, price);
    }
}

// check: EXECUTED

//# run --signers ds2
script {
    use bob::DelegateOracleDS;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun main(account: signer) {
        DelegateOracleDS::delegate<STCUSD>(&account);
    }
}

// check: EXECUTED


//# run --signers ds2
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun update_price(account: signer) {
        oracle_price::update<STCUSD>(&account, 0);
    }
}
// ds2 can not update ds2 datasource price
// check: ABORTED

//# run --signers alice
script {
    use starcoin_framework::oracle_stc_usd::STCUSD;
    use bob::DelegateOracleDS;

    fun update_price(_signer: signer) {
        DelegateOracleDS::update_price_any<STCUSD>(@ds2, 0);
    }
}
// alice can update ds2 datasource price by DelegateOracleDS
// check: EXECUTED

//# run --signers alice
script {
    use starcoin_framework::oracle_price;
    use starcoin_framework::oracle_stc_usd::STCUSD;

    fun test_read_price(_signer: signer) {
        assert!(oracle_price::read<STCUSD>(@ds2) == 0, 1004);
    }
}

// check: EXECUTED