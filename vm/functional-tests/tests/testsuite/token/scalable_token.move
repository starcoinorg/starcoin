//! account: alice
//! account: bob

//! sender: alice

module StableCoin {
    use 0x1::Token;

    struct StableCoin { }

    resource struct RebaseCapability {
        cap: Token::ScalingFactorModifyCapability<StableCoin>,
    }

    public fun init(signer: &signer) {
        Token::register_token<StableCoin>(signer, 1000000000, 1000000000);
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice

script {
use {{alice}}::StableCoin::{Self, StableCoin as T};
use 0x1::Account;
use 0x1::Token;
use 0x1::Signer;
fun main(account: &signer) {
    StableCoin::init(account);

    let market_cap = Token::market_cap<T>();
    assert(market_cap == 0, 8001);
    assert(Token::is_registered_in<T>({{alice}}), 8002);
    Account::accept_token<T>(account);

    let token = Token::mint<T>(account, 1000);
    Account::deposit(account, token);
    let balance = Account::balance<T>(Signer::address_of(account));
    assert(balance == 1000, 1000);
    Token::set_scaling_factor<T>(account, Token::base_scaling_factor<T>() * 2);
    let balance = Account::balance<T>(Signer::address_of(account));
    assert(balance == 2000, 2000);


    let token = Token::mint<T>(account, 1000);
    let value = Token::value(&token);
    assert(value == 1000, 10000);
    Account::deposit(account, token);
    let balance = Account::balance<T>(Signer::address_of(account));
    assert(balance == 3000, 3000);
    assert(Token::market_cap<T>() == 3000, 3000);
    assert(Token::total_share<T>() == 1500, 1500);
}
}

// check: EXECUTED