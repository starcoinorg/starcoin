//! account: alice

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::initialize(signer);
}
}

// check: ABORTED


//! new-transaction
//! sender: genesis

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::initialize(signer);
}
}

// check: CANNOT_WRITE_EXISTING_RESOURCE


//! new-transaction
//! sender: alice

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::update_global_time(signer, 20);
}
}

// check: ABORTED

//! new-transaction
//! sender: genesis

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::update_global_time(signer, 20);
}
}

// check: EXECUTED


//! new-transaction
//! sender: genesis

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    let now = Timestamp::now_seconds();
    Timestamp::update_global_time(signer, now-1);
}
}

// check: ABORTED
// check: 5001



