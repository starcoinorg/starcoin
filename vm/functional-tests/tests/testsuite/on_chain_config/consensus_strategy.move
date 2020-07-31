script {
use 0x1::ConsensusStrategy;

fun main() {
    assert(ConsensusStrategy::get() == 0, 1);
}
}
