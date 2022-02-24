//# init -n dev

//# faucet --addr alice

//# faucet --addr Genesis

//# run --signers alice
script {
use StarcoinFramework::Timestamp;

fun main(_signer: signer) {
    assert!(!Timestamp::is_genesis(), 1000);
}
}

//# run --signers alice
script {
    use StarcoinFramework::Timestamp;

    fun main(_signer: signer) {
        Timestamp::assert_genesis();
    }
}
// check: "Keep(ABORTED { code: 3073"

//# run --signers alice
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    Timestamp::initialize(&signer, 0);
}
}

// check: ABORTED



//# run --signers Genesis
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    Timestamp::initialize(&signer, 0);
}
}

// check: RESOURCE_ALREADY_EXISTS 



//# run --signers alice
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    Timestamp::update_global_time(&signer, 10);
}
}

// check: ABORTED


//# run --signers Genesis
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    Timestamp::update_global_time(&signer, 200000);
}
}

// check: EXECUTED




//# run --signers Genesis
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    let now = Timestamp::now_seconds();
    Timestamp::update_global_time(&signer, now);
}
}
// check: ABORTED


//# run --signers Genesis
script {
use StarcoinFramework::Timestamp;

fun main(signer: signer) {
    let now = Timestamp::now_seconds();
    Timestamp::update_global_time(&signer, now-1);
}
}

// check: ABORTED
// check: 100



