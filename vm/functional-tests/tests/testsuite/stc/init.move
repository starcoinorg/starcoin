//! account: alice

script {
use 0x1::STC;

fun main(signer: &signer) {
    STC::initialize(signer);
}
}

// check: ABORTED


//! new-transaction
//! sender: genesis

script {
use 0x1::STC;

fun main(signer: &signer) {
    STC::initialize(signer);
}
}

// check: CANNOT_WRITE_EXISTING_RESOURCE
