script {
use 0x1::Consensus;

fun main() {
    assert(Consensus::uncle_rate_target() == 80, 8100);
    assert(Consensus::epoch_time_target() == 1209600, 8101);
    assert(Consensus::reward_half_time_target() == 126144000, 8102);
}
}
