
<a name="propose_update_reward_config"></a>

# Script `propose_update_reward_config`



-  [Specification](#@Specification_0)
    -  [Function `propose_update_reward_config`](#@Specification_0_propose_update_reward_config)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="propose_update_reward_config.md#propose_update_reward_config">propose_update_reward_config</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="propose_update_reward_config.md#propose_update_reward_config">propose_update_reward_config</a>(account: &signer,
    reward_delay: u64,
    exec_delay: u64) {
    <b>let</b> reward_config = <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_new_reward_config">RewardConfig::new_reward_config</a>(reward_delay);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(account, reward_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_propose_update_reward_config"></a>

### Function `propose_update_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="propose_update_reward_config.md#propose_update_reward_config">propose_update_reward_config</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
