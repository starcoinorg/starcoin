//# init -n dev

//# faucet --addr alice

//# faucet --addr bob

//# faucet --addr carol

//# publish

module alice::SillyColdWallet {
    use StarcoinFramework::Account;

    struct T has key, store {
        cap: Account::WithdrawCapability,
        owner: address,
    }

    public fun publish(account: &signer, cap: Account::WithdrawCapability, owner: address) {
        move_to(account, T { cap, owner });
    }
}


//# run --signers alice


script {
use alice::SillyColdWallet;
use StarcoinFramework::Account;

// create a cold wallet for Bob that withdraws from Alice's account
fun main(sender: signer) {
    let cap = Account::extract_withdraw_capability(&sender);
    SillyColdWallet::publish(&sender, cap, @bob);
}
}
// check: "Keep(EXECUTED)"


//# run --signers alice

script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;

// check that Alice can no longer withdraw from her account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    // should fail with withdrawal capability already extracted
    Account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}
// check: "Keep(ABORTED { code: 25857,"


//# run --signers alice
script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;
use StarcoinFramework::Signer;

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


//# run --signers bob

script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;

// check that Bob can still pay from his normal account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    Account::pay_from_capability<STC>(&with_cap, @bob, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}


//# run --signers bob
script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;

// check that Bob can still withdraw from his normal account
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    let coin = Account::withdraw_with_capability<STC>(&with_cap, 1000);
    Account::deposit_to_self<STC>(&account, coin);
    Account::restore_withdraw_capability(with_cap);
}
}


//# run --signers bob
script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;
use StarcoinFramework::Signer;

    // check that Bob can still withdraw from his normal account
    fun main(account: signer) {
        let coin = Account::withdraw_with_metadata<STC>(&account, 1000, x"");
        Account::deposit_with_metadata<STC>(Signer::address_of(&account), coin, x"");
    }
}


//# run --signers carol

script {
use StarcoinFramework::STC::STC;
use StarcoinFramework::Account;

// check that other users can still pay into Alice's account in the normal way
fun main(account: signer) {
    let with_cap = Account::extract_withdraw_capability(&account);
    Account::pay_from_capability<STC>(&with_cap, @alice, 1000, x"");
    Account::restore_withdraw_capability(with_cap);
}
}
