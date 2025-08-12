module starcoin_framework::oracle_stc_usd {
    use starcoin_framework::oracle::{DataRecord};
    use starcoin_framework::oracle_price::{Self};

    /// The STC to USD price oracle
    struct STCUSD has copy, store, drop {}

    public fun register(sender: &signer) {
        oracle_price::register_oracle<STCUSD>(sender, 6);
    }

    public fun read(ds_addr: address): u128 {
        oracle_price::read<STCUSD>(ds_addr)
    }

    public fun read_record(ds_addr: address): DataRecord<u128> {
        oracle_price::read_record<STCUSD>(ds_addr)
    }

    public fun read_records(ds_addrs: &vector<address>): vector<DataRecord<u128>> {
        oracle_price::read_records<STCUSD>(ds_addrs)
    }
}
