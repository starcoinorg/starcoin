//# init -n dev

//# faucet --addr alice

//# faucet --addr bob


//# publish
module alice::MyToken {
    use StarcoinFramework::Token;
    use StarcoinFramework::Signer;

    struct MyToken has copy, drop, store { }

    public fun init(account: &signer, precision: u8) {
        assert!(Signer::address_of(account) == @alice, 8000);

        Token::register_token<MyToken>(
                    account,
                    precision,
        );
    }
}

// check: EXECUTED

//# run --signers alice
script {
use alice::MyToken;

fun main(account: signer) {
    MyToken::init(&account, 39); // EPRECISION_TOO_LARGE
}
}

// check: "Keep(ABORTED { code: 26887"

//# run --signers alice
script {
use alice::MyToken;

fun main(account: signer) {
MyToken::init(&account, 3);
}
}

// check: EXECUTED

//# run --signers alice
script {
use alice::MyToken::MyToken;
use StarcoinFramework::Token;

fun main(_account: signer) {
    let sf = Token::scaling_factor<MyToken>();
    assert!(sf == 1000, 101);
}
}

// check: EXECUTED