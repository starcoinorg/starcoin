//# init -n dev

//# faucet --addr alice


//# run --signers alice
script {
use Std::Timestamp;

fun main() {
    assert!(!Timestamp::is_genesis(), 10);
}
}
