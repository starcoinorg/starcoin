//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
use StarcoinFramework::Timestamp;

fun main() {
    assert!(!Timestamp::is_genesis(), 10);
}
}
