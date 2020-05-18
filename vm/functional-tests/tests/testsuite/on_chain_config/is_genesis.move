script {
use 0x0::Timestamp;
use 0x0::Transaction;

fun main() {
    Transaction::assert(!Timestamp::is_genesis(), 10)
}
}
