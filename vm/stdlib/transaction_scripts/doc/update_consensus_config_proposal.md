
<a name="update_consensus_config"></a>

# Script `update_consensus_config`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_consensus_config_proposal.md#update_consensus_config">update_consensus_config</a></code>](#@Specification_0_update_consensus_config)



<pre><code><b>public</b> <b>fun</b> <a href="update_consensus_config_proposal.md#update_consensus_config">update_consensus_config</a>(account: &signer, uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_consensus_config_proposal.md#update_consensus_config">update_consensus_config</a>(account: &signer,
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
    <b>let</b> consensus_config = <a href="../../modules/doc/ConsensusConfig.md#0x1_ConsensusConfig_new_consensus_config">ConsensusConfig::new_consensus_config</a>(uncle_rate_target,
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
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>&gt;(account, consensus_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_consensus_config"></a>

### Function `update_consensus_config`


<pre><code><b>public</b> <b>fun</b> <a href="update_consensus_config_proposal.md#update_consensus_config">update_consensus_config</a>(account: &signer, uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
