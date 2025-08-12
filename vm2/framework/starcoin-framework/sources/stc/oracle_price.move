module starcoin_framework::oracle_price {

    use starcoin_std::math128;
    use starcoin_framework::oracle::{Self, UpdateCapability, DataRecord};

    struct PriceOracleInfo has copy, store, drop {
        scaling_factor: u128,
    }

    public entry fun register_oracle_entry<OracleT: copy+store+drop>(sender: signer, precision: u8) {
        register_oracle<OracleT>(&sender, precision);
    }

    public fun register_oracle<OracleT: copy + store + drop>(sender: &signer, precision: u8) {
        let scaling_factor = math128::pow(10, (precision as u128));
        oracle::register_oracle<OracleT, PriceOracleInfo>(sender, PriceOracleInfo {
            scaling_factor,
        });
    }


    public entry fun init_data_source_entry<OracleT: copy+store+drop>(sender: signer, init_value: u128) {
        init_data_source<OracleT>(&sender, init_value);
    }

    public fun init_data_source<OracleT: copy+store+drop>(sender: &signer, init_value: u128) {
        oracle::init_data_source<OracleT, PriceOracleInfo, u128>(sender, init_value);
    }

    public fun is_data_source_initialized<OracleT: copy+store+drop>(ds_addr: address): bool {
        oracle::is_data_source_initialized<OracleT, u128>(ds_addr)
    }

    public fun get_scaling_factor<OracleT: copy + store + drop>(): u128 {
        let info = oracle::get_oracle_info<OracleT, PriceOracleInfo>();
        info.scaling_factor
    }

    public entry fun update_entry<OracleT: copy+store+drop>(sender: signer, value: u128) {
        update<OracleT>(&sender, value);
    }

    public fun update<OracleT: copy+store+drop>(sender: &signer, value: u128) {
        oracle::update<OracleT, u128>(sender, value);
    }

    public fun update_with_cap<OracleT: copy+store+drop>(cap: &mut UpdateCapability<OracleT>, value: u128) {
        oracle::update_with_cap<OracleT, u128>(cap, value);
    }

    public fun read<OracleT: copy+store+drop>(addr: address): u128 {
        oracle::read<OracleT, u128>(addr)
    }

    public fun read_record<OracleT: copy+store+drop>(addr: address): DataRecord<u128> {
        oracle::read_record<OracleT, u128>(addr)
    }

    public fun read_records<OracleT: copy+store+drop>(addrs: &vector<address>): vector<DataRecord<u128>> {
        oracle::read_records<OracleT, u128>(addrs)
    }
}

