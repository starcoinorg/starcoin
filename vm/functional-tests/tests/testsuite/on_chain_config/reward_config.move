script {
use 0x1::RewardConfig;

fun main() {
    assert(RewardConfig::reward_delay() > 0, 8000);
    assert(RewardConfig::reward_halving_interval() > 0, 8001);
    assert(RewardConfig::reward_base() > 0, 8002);
}
}
