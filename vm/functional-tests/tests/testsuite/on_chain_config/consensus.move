script {
use 0x1::ConsensusConfig;

fun main() {
    let config = ConsensusConfig::get_config();
    assert(ConsensusConfig::uncle_rate_target(&config) == 80, 8100);
    //assert(ConsensusConfig::epoch_time_target() == 1209600, 8101);
    //assert(ConsensusConfig::reward_half_time_target() == 126144000, 8102);
}
}
