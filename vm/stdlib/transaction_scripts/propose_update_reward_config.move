script {
use 0x1::RewardConfig;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun propose_update_reward_config(account: &signer,
    reward_delay: u64,
    exec_delay: u64) {
    let reward_config = RewardConfig::new_reward_config(reward_delay);
    OnChainConfigDao::propose_update<STC::STC, RewardConfig::RewardConfig>(account, reward_config, exec_delay);
}

spec fun propose_update_reward_config {
    pragma verify = false;
}
}
