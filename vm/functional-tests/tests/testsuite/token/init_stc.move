//! account: alice

script {
use 0x1::STC;

fun main(signer: &signer) {
    STC::initialize(signer, 500, 5000, 10, 600);
}
}

// check: ABORTED


//! new-transaction
//! sender: genesis

script {
use 0x1::STC;

fun main(signer: &signer) {
    STC::initialize(signer, 500, 5000, 10, 600);
}
}

// check: RESOURCE_ALREADY_EXISTS
