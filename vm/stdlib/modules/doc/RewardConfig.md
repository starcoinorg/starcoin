
<a name="0x1_RewardConfig"></a>

# Module `0x1::RewardConfig`



-  [Struct <code><a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a></code>](#0x1_RewardConfig_RewardConfig)
-  [Function <code>initialize</code>](#0x1_RewardConfig_initialize)
-  [Function <code>new_reward_config</code>](#0x1_RewardConfig_new_reward_config)
-  [Function <code>get_reward_config</code>](#0x1_RewardConfig_get_reward_config)
-  [Function <code>reward_delay</code>](#0x1_RewardConfig_reward_delay)


<a name="0x1_RewardConfig_RewardConfig"></a>

## Struct `RewardConfig`



<pre><code><b>struct</b> <a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>reward_delay: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_RewardConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">Self::RewardConfig</a>&gt;(
        account,
        <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay)
    );
}
</code></pre>



</details>

<a name="0x1_RewardConfig_new_reward_config"></a>

## Function `new_reward_config`



<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay: u64) : <a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a> {
    <a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a> {reward_delay: reward_delay}
}
</code></pre>



</details>

<a name="0x1_RewardConfig_get_reward_config"></a>

## Function `get_reward_config`



<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_get_reward_config">get_reward_config</a>(): <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_get_reward_config">get_reward_config</a>(): <a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a> {
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_RewardConfig_reward_delay"></a>

## Function `reward_delay`



<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">reward_delay</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">reward_delay</a>() :u64 {
    <b>let</b> reward_config = <a href="RewardConfig.md#0x1_RewardConfig_get_reward_config">get_reward_config</a>();
    reward_config.reward_delay
}
</code></pre>



</details>
