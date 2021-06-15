//! account: alice
//! account: bob
//! account: carol

//! sender: alice
address alice = {{alice}};
module alice::SillyColdWallet {
    use 0x1::Account;

    struct T has key, store {
        cap: Account::WithdrawCapability,
        owner: address,
    }

    public fun publish(account: &signer, cap: Account::WithdrawCapability, owner: address) {
        move_to(account, T { cap, owner });
    }
}

//! new-transaction
//! sender: alice
address alice = {{alice}};
address bob = {{bob}};
script {
use alice::SillyColdWallet;
use 0x1::Account;

// create a cold wallet for Bob that withdraws from Alice's account
fun main(sender: signer) {
    let cap = Account::extract_withdraw_capability(&sender);
    SillyColdWallet::publish(&sender, cap, @bob);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
use 0x1::STC::STC;
use 0x1::Account;

// check that Alice can no longer withdraw from her account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    // should fail with withdrawal capability already extracted
    Account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 25857,"

//! new-transaction
//! sender: alice
script {
use 0x1::STC::STC;
use 0x1::Account;
use 0x1::Signer;

// check that Alice can no longer withdraw from her account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    // should fail with withdrawal capability already extracted
    let coin = Account::withdraw_with_metadata<STC>(&account, 1000, x"");
    Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
    Account::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 25857,"

//! new-transaction
//! sender: bob
address bob = {{bob}};
script {
use 0x1::STC::STC;
use 0x1::Account;

// check that Bob can still pay from his normal account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    Account::pay_from_capability<STC>(&with_cap, @bob, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}

//! new-transaction
//! sender: bob
script {
use 0x1::STC::STC;
use 0x1::Account;

// check that Bob can still withdraw from his normal account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    let coin = Account::withdraw_with_capability<STC>(&with_cap, 1000);
    Account::deposit_to_self<STC>(&account, coin);
    Account::restore_withdraw_capability(with_cap);
}
}

//! new-transaction
//! sender: bob
script {
use 0x1::STC::STC;
use 0x1::Account;
use 0x1::Signer;

    // check that Bob can still withdraw from his normal account
    fun main(account: signer) {
        let coin = Account::withdraw_with_metadata<STC>(&account, 1000, x"");
        Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
    }
}

//! new-transaction
//! sender: carol
address alice = {{alice}};
script {
use 0x1::STC::STC;
use 0x1::Account;

// check that other users can still pay into Alice's account in the normal way
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    Account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}
