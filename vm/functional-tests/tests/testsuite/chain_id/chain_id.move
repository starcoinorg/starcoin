script {
use 0x1::ChainId;

fun main() {
    assert(ChainId::get() == 255, 1000);
}
}
