
<a name="update_reward_config"></a>

# Script `update_reward_config`



-  [Specification](#@Specification_0)
    -  [Function <code><a href="update_reward_config_proposal.md#update_reward_config">update_reward_config</a></code>](#@Specification_0_update_reward_config)



<pre><code><b>public</b> <b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config">update_reward_config</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config">update_reward_config</a>(account: &signer,
    reward_delay: u64,
    exec_delay: u64) {
    <b>let</b> reward_config = <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_new_reward_config">RewardConfig::new_reward_config</a>(reward_delay);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(account, reward_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_reward_config"></a>

### Function `update_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config">update_reward_config</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
