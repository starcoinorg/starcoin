script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_key = x"fb51f08c8e63ed9f4eac340b25d1b01d75995fa86f8ebc0b0819ebf80abc0ee6";
    let created_address = Account::create_account<STC>(dummy_auth_key);
    assert(Account::is_accepts_token<STC>(created_address), 1);
    assert(!Account::is_accepts_token<DummyToken>(created_address), 2);
}
}

// check: EXECUTED


//! new-transaction

script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_key = x"9028e2757e3e57339af4b3df1d818cddac6e527363459f072d70484599956c8a";
    let created_address = Account::create_account<DummyToken>(dummy_auth_key);
    assert(Account::is_accepts_token<STC>(created_address), 1);
    assert(Account::is_accepts_token<DummyToken>(created_address), 2);
}
}

// check: EXECUTED


