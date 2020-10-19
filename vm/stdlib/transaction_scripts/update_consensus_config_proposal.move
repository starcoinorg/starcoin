script {
use 0x1::ConsensusConfig;
use 0x1::OnChainConfigDao;
use 0x1::STC;

fun update_consensus_config_proposal(account: &signer,
    uncle_rate_target: u64,
    base_block_time_target: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    epoch_block_count: u64,
    base_block_difficulty_window: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
    strategy: u8,
    exec_delay: u64) {
    let consensus_config = ConsensusConfig::new_consensus_config(uncle_rate_target,
                             base_block_time_target,
                             base_reward_per_block,
                             base_reward_per_uncle_percent,
                             epoch_block_count,
                             base_block_difficulty_window,
                             min_block_time_target,
                             max_block_time_target,
                             base_max_uncles_per_block,
                             base_block_gas_limit,
                             strategy);
    OnChainConfigDao::propose_update<STC::STC, ConsensusConfig::ConsensusConfig>(account, consensus_config, exec_delay);
}

spec fun update_consensus_config_proposal {
    pragma verify = false;
}
}
