//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use starcoin_framework::starcoin_coin::{Self, STC};
use starcoin_framework::DummyToken::{Self, DummyToken};
fun main() {
    assert!(STC::is_stc<STC>(), 1);
    //TODO support check any type.
    //assert!(!STC::is_stc<bool>(), 3);
    assert!(!STC::is_stc<DummyToken>(), 4);
    assert!(DummyToken::is_dummy_token<DummyToken>(), 5);
    assert!(!DummyToken::is_dummy_token<STC>(), 6);
}
}

// check: EXECUTED
