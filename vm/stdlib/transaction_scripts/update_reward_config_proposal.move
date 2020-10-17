script {
use 0x1::RewardConfig;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun update_reward_config_proposal(account: &signer,
    reward_delay: u64,
    exec_delay: u64) {
    let reward_config = RewardConfig::new_reward_config(reward_delay);
    OnChainConfigDao::propose_update<STC::STC, RewardConfig::RewardConfig>(account, reward_config, exec_delay);
}

spec fun update_reward_config_proposal {
    pragma verify = false;
}
}
