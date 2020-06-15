script {
use 0x1::Timestamp;

fun main() {
    assert(!Timestamp::is_genesis(), 10)
}
}
