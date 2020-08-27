//! account: alice

script {
use 0x1::Timestamp;

fun main(_signer: &signer) {
    assert(!Timestamp::is_genesis(), 1000);
}
}

//! new-transaction
//! sender: alice

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::initialize(signer, 0);
}
}

// check: ABORTED


//! new-transaction
//! sender: genesis

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::initialize(signer, 0);
}
}

// check: RESOURCE_ALREADY_EXISTS 


//! new-transaction
//! sender: alice

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    Timestamp::update_global_time(signer, 10);
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
    Timestamp::update_global_time(signer, now);
}
}

// check: ABORTED
// check: 100


//! new-transaction
//! sender: genesis

script {
use 0x1::Timestamp;

fun main(signer: &signer) {
    let now = Timestamp::now_seconds();
    Timestamp::update_global_time(signer, now-1);
}
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
// check: 100



