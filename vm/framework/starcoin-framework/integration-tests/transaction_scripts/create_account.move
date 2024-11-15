//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# run --signers alice --args 0x75995fa86f8ebc0b0819ebf80abc0ee6 --args 100u128
script {
    use starcoin_framework::account;
    use starcoin_framework::starcoin_coin::STC;

    fun main(account: signer, fresh_address: address, initial_amount: u128) {
        account::create_account_with_initial_amount_v2<STC>(account, fresh_address, initial_amount);
    }
}
