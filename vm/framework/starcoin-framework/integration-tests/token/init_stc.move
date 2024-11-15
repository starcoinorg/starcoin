//# init -n dev

//# faucet --addr alice

//# faucet --addr Genesis

//# run --signers alice
script {
use starcoin_framework::STC;

fun main(signer: signer) {
    STC::initialize(&signer, 500, 5000, 10, 600);
}
}

// check: ABORTED



//# run --signers Genesis

script {
use starcoin_framework::STC;

fun main(signer: signer) {
    STC::initialize(&signer, 500, 5000, 10, 600);
}
}

// check: RESOURCE_ALREADY_EXISTS
