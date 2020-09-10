script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_public_key = x"fb51f08c8e63ed9f4eac340b25d1b01d75995fa86f8ebc0b0819ebf80abc0ee6";
    Account::create_account<STC>(0xb2b986a327117426f56683557ff807ea, dummy_auth_public_key);
    assert(Account::is_accepts_token<STC>(0xb2b986a327117426f56683557ff807ea), 1);
    assert(!Account::is_accepts_token<DummyToken>(0xb2b986a327117426f56683557ff807ea), 2);
}
}

// check: EXECUTED


//! new-transaction

script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_public_key = x"9028e2757e3e57339af4b3df1d818cddac6e527363459f072d70484599956c8a";
    Account::create_account<DummyToken>(0x0ae49c10e21a33f671ed49b2d2ad55b2,dummy_auth_public_key);
    assert(Account::is_accepts_token<STC>(0x0ae49c10e21a33f671ed49b2d2ad55b2), 1);
    assert(Account::is_accepts_token<DummyToken>(0x0ae49c10e21a33f671ed49b2d2ad55b2), 2);
}
}

// check: EXECUTED


