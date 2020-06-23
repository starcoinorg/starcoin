//! account: alice

//! sender: alice
module Coin1 {
    struct C {}
    struct D {}
    struct Coin1<T> {}
}
//! check: EXECUTED

//! new-transaction
//! sender: alice
script {
use 0x1::Coin;
use 0x1::FixedPoint32;
use 0x1::Account;
use 0x1::Signer;
use {{alice}}::Coin1::{Coin1, C, D};
fun main(signer: &signer) {
    Coin::register_currency<Coin1<C>>(
        signer,
        FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
    );
    Account::add_currency<Coin1<C>>(signer);
    Coin::register_currency<Coin1<D>>(
        signer,
        FixedPoint32::create_from_rational(1, 1), // exchange rate to STC
        1000000, // scaling_factor = 10^6
        1000,    // fractional_part = 10^3
    );
    Account::add_currency<Coin1<D>>(signer);
    let b = Account::balance<Coin1<C>>(Signer::address_of(signer));
    assert(b == 0, 1);
}
}

//! check: EXECUTED