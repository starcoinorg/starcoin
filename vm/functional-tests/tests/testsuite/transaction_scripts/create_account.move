//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! args: 0x75995fa86f8ebc0b0819ebf80abc0ee6, x"fb51f08c8e63ed9f4eac340b25d1b01d75995fa86f8ebc0b0819ebf80abc0ee6", 100u128
script {
    use 0x1::Account;
    use 0x1::STC::STC;

    fun main(account: &signer, fresh_address: address, auth_key: vector<u8>, initial_amount: u128) {
        Account::create_account_with_initial_amount<STC>(account, fresh_address, auth_key, initial_amount);
    }
}
// check: gas_used
// check: 1068941
// check: "Keep(EXECUTED)"