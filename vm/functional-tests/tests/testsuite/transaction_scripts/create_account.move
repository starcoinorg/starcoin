//! account: alice, 100000 0x1::STC::STC
//! account: bob

//! sender: alice
//! args: 0x75995fa86f8ebc0b0819ebf80abc0ee6, 100u128
script {
    use 0x1::Account;
    use 0x1::STC::STC;

    fun main(account: signer, fresh_address: address, initial_amount: u128) {
        Account::create_account_with_initial_amount_v2<STC>(account, fresh_address, initial_amount);
    }
}
// check: gas_used
// check: 930513
// check: "Keep(EXECUTED)"