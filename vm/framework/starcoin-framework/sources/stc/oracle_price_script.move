module starcoin_framework::oracle_price_script {
    use starcoin_framework::oracle_price;

    public entry fun register_oracle<OracleT: copy+store+drop>(sender: signer, precision: u8) {
        oracle_price::register_oracle_entry<OracleT>(sender, precision);
    }

    public entry fun init_data_source<OracleT: copy+store+drop>(sender: signer, init_value: u128) {
        oracle_price::init_data_source_entry<OracleT>(sender, init_value);
    }

    public entry fun update<OracleT: copy+store+drop>(sender: signer, value: u128) {
        oracle_price::update_entry<OracleT>(sender, value);
    }
}