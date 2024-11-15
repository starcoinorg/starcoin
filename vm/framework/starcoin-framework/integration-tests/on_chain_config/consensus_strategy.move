//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use starcoin_framework::ConsensusStrategy;

fun main() {
    assert!(ConsensusStrategy::get() == 0, 1);
}
}
