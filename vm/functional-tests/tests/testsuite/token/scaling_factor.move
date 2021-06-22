// Test user-defined token
//! account: alice
//! account: bob

//! sender: alice
address alice = {{alice}};
module alice::MyToken {
    use 0x1::Token;
    use 0x1::Signer;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer, precision: u8) {
        assert(Signer::address_of(account) == @alice, 8000);

        Token::register_token<MyToken>(
                    account,
                    precision,
        );
    }
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
use alice::MyToken;

fun main(account: signer) {
    MyToken::init(&account, 39); // EPRECISION_TOO_LARGE
}
}

// check: "Keep(ABORTED { code: 26887"

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
use alice::MyToken;

fun main(account: signer) {
MyToken::init(&account, 3);
}
}

// check: EXECUTED

//! new-transaction
//! sender: alice
address alice = {{alice}};
script {
use alice::MyToken::MyToken;
use 0x1::Token;

fun main(_account: signer) {
    let sf = Token::scaling_factor<MyToken>();
    assert(sf == 1000, 101);
}
}

// check: EXECUTED