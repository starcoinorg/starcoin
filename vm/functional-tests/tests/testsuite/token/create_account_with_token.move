script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    Account::create_account<STC>(0x3,dummy_auth_key_prefix);
    assert(Account::is_accepts_token<STC>(0x3), 1);
    assert(!Account::is_accepts_token<DummyToken>(0x3), 2);
}
}

// check: EXECUTED


//! new-transaction

script {
use 0x1::Account;
use 0x1::STC::STC;
use 0x1::DummyToken::DummyToken;

fun main() {
    let dummy_auth_key_prefix = x"00000000000000000000000000000000";
    Account::create_account<DummyToken>(0x4,dummy_auth_key_prefix);
    assert(Account::is_accepts_token<STC>(0x4), 1);
    assert(Account::is_accepts_token<DummyToken>(0x4), 2);
}
}

// check: EXECUTED


