//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# publish

module alice::SillyColdWallet {
    use starcoin_framework::account;

    struct T has key, store {
        cap: account::WithdrawCapability,
        owner: address,
    }

    public fun publish(account: &signer, cap: account::WithdrawCapability, owner: address) {
        move_to(account, T { cap, owner });
    }
}


//# run --signers alice


script {
    use alice::SillyColdWallet;
    use starcoin_framework::account;

    // create a cold wallet for Bob that withdraws from Alice's account
    fun main(sender: signer) {
        let cap = account::extract_withdraw_capability(&sender);
        SillyColdWallet::publish(&sender, cap, @bob);
    }
}
// check: "Keep(EXECUTED)"


//# run --signers alice

script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;

    // check that Alice can no longer withdraw from her account
    fun main(account: signer) {
        let with_cap = account::extract_withdraw_capability(&account);
        // should fail with withdrawal capability already extracted
        account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
        account::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 25857,"


//# run --signers alice
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;
    use starcoin_framework::signer;

    // check that Alice can no longer withdraw from her account
    fun main(account: signer) {
        let with_cap = account::extract_withdraw_capability(&account);
        // should fail with withdrawal capability already extracted
        let coin = account::withdraw_with_metadata<STC>(&account, 1000, x"");
        account::deposit_with_metadata<STC>(signer::address_of(&account), coin, x"");
        account::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 25857,"


//# run --signers bob

script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;

    // check that Bob can still pay from his normal account
    fun main(account: signer) {
        let with_cap = account::extract_withdraw_capability(&account);
        account::pay_from_capability<STC>(&with_cap, @bob, 1000, x"");
        account::restore_withdraw_capability(with_cap);
    }
}


//# run --signers bob
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;

    // check that Bob can still withdraw from his normal account
    fun main(account: signer) {
        let with_cap = account::extract_withdraw_capability(&account);
        let coin = account::withdraw_with_capability<STC>(&with_cap, 1000);
        coin::deposit<STC>(&account, coin);
        account::restore_withdraw_capability(with_cap);
    }
}


//# run --signers bob
script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;
    use starcoin_framework::signer;

    // check that Bob can still withdraw from his normal account
    fun main(account: signer) {
        let coin = account::withdraw_with_metadata<STC>(&account, 1000, x"");
        account::deposit_with_metadata<STC>(signer::address_of(&account), coin, x"");
    }
}


//# run --signers carol

script {
    use starcoin_framework::starcoin_coin::STC;
    use starcoin_framework::account;

    // check that other users can still pay into Alice's account in the normal way
    fun main(account: signer) {
        let with_cap = account::extract_withdraw_capability(&account);
        account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
        account::restore_withdraw_capability(with_cap);
    }
}
