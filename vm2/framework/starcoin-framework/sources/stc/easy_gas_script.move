module starcoin_framework::easy_gas_script {

    use starcoin_framework::easy_gas;

    public entry fun register<TokenType: store>(sender: signer, precision: u8) {
        easy_gas::register_oracle<TokenType>(&sender, precision)
    }

    public entry fun init_data_source<TokenType: store>(sender: signer, init_value: u128) {
        easy_gas::init_oracle_source<TokenType>(&sender, init_value);
    }

    public entry fun update<TokenType: store>(sender: signer, value: u128) {
        easy_gas::update_oracle<TokenType>(&sender, value)
    }

    public entry fun withdraw_gas_fee_entry<TokenType: store>(sender: signer, amount: u128) {
        easy_gas::withdraw_gas_fee<TokenType>(&sender, amount);
    }

    // public entry fun deposit<TokenType: store>(sender: signer, amount:u128)  {
    //     let address = EasyGas::get_gas_fee_address();
    //     peer_to_peer_v2<TokenType>(sender, address, amount)
    // }
}