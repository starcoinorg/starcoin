
<a name="update_reward_config_proposal"></a>

# Script `update_reward_config_proposal`



-  [Specification](#@Specification_0)
    -  [Function `update_reward_config_proposal`](#@Specification_0_update_reward_config_proposal)


<pre><code><b>use</b> <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao">0x1::OnChainConfigDao</a>;
<b>use</b> <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig">0x1::RewardConfig</a>;
<b>use</b> <a href="../../modules/doc/STC.md#0x1_STC">0x1::STC</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config_proposal">update_reward_config_proposal</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config_proposal">update_reward_config_proposal</a>(account: &signer,
    reward_delay: u64,
    exec_delay: u64) {
    <b>let</b> reward_config = <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_new_reward_config">RewardConfig::new_reward_config</a>(reward_delay);
    <a href="../../modules/doc/OnChainConfigDao.md#0x1_OnChainConfigDao_propose_update">OnChainConfigDao::propose_update</a>&lt;<a href="../../modules/doc/STC.md#0x1_STC_STC">STC::STC</a>, <a href="../../modules/doc/RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>&gt;(account, reward_config, exec_delay);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_update_reward_config_proposal"></a>

### Function `update_reward_config_proposal`


<pre><code><b>public</b> <b>fun</b> <a href="update_reward_config_proposal.md#update_reward_config_proposal">update_reward_config_proposal</a>(account: &signer, reward_delay: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
