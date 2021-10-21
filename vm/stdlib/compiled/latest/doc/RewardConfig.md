
<a name="0x1_RewardConfig"></a>

# Module `0x1::RewardConfig`

The module provide configuration for block reward.


-  [Struct `RewardConfig`](#0x1_RewardConfig_RewardConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_RewardConfig_initialize)
-  [Function `new_reward_config`](#0x1_RewardConfig_new_reward_config)
-  [Function `get_reward_config`](#0x1_RewardConfig_get_reward_config)
-  [Function `reward_delay`](#0x1_RewardConfig_reward_delay)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `new_reward_config`](#@Specification_1_new_reward_config)
    -  [Function `get_reward_config`](#@Specification_1_get_reward_config)
    -  [Function `reward_delay`](#@Specification_1_reward_delay)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_RewardConfig_RewardConfig"></a>

## Struct `RewardConfig`

Reward configuration


<pre><code><b>struct</b> <a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>reward_delay: u64</code>
</dt>
<dd>
 how many blocks delay reward distribution.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_RewardConfig_EINVALID_ARGUMENT"></a>



<pre><code><b>const</b> <a href="RewardConfig.md#0x1_RewardConfig_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>: u64 = 18;
</code></pre>



<a name="0x1_RewardConfig_initialize"></a>

## Function `initialize`

Module initialization.


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">Self::RewardConfig</a>&gt;(
        account,
        <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay)
    );
}
</code></pre>



</details>

<a name="0x1_RewardConfig_new_reward_config"></a>

## Function `new_reward_config`

Create a new reward config mainly used in DAO.


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

Get reward configuration.


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

Get reward delay.


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

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">Config::PublishNewConfigAbortsIf</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;;
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">Config::PublishNewConfigEnsures</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;;
</code></pre>



<a name="@Specification_1_new_reward_config"></a>

### Function `new_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>
</code></pre>




<a name="@Specification_1_get_reward_config"></a>

### Function `get_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_get_reward_config">get_reward_config</a>(): <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>
</code></pre>




<pre><code><b>include</b> <a href="RewardConfig.md#0x1_RewardConfig_GetRewardConfigAbortsIf">GetRewardConfigAbortsIf</a>;
</code></pre>




<a name="0x1_RewardConfig_GetRewardConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="RewardConfig.md#0x1_RewardConfig_GetRewardConfigAbortsIf">GetRewardConfigAbortsIf</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
}
</code></pre>



<a name="@Specification_1_reward_delay"></a>

### Function `reward_delay`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">reward_delay</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
</code></pre>
