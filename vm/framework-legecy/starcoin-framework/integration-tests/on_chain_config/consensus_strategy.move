//# init -n dev

//# faucet --addr alice

//# run --signers alice
script {
use StarcoinFramework::ConsensusStrategy;

fun main() {
    assert!(ConsensusStrategy::get() == 0, 1);
}
}
