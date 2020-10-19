
<a name="0x1_RewardConfig"></a>

# Module `0x1::RewardConfig`



-  [Struct `RewardConfig`](#0x1_RewardConfig_RewardConfig)
-  [Function `initialize`](#0x1_RewardConfig_initialize)
-  [Function `new_reward_config`](#0x1_RewardConfig_new_reward_config)
-  [Function `get_reward_config`](#0x1_RewardConfig_get_reward_config)
-  [Function `reward_delay`](#0x1_RewardConfig_reward_delay)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `new_reward_config`](#@Specification_0_new_reward_config)
    -  [Function `get_reward_config`](#@Specification_0_get_reward_config)
    -  [Function `reward_delay`](#@Specification_0_reward_delay)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



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
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS">Errors::ENOT_GENESIS</a>()));
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()));

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
    <b>assert</b>(reward_delay &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">Errors::EINVALID_ARGUMENT</a>()));
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

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_initialize">initialize</a>(account: &signer, reward_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">Config::PublishNewConfigAbortsIf</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;;
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">Config::PublishNewConfigEnsures</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;;
</code></pre>



<a name="@Specification_0_new_reward_config"></a>

### Function `new_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="RewardConfig.md#0x1_RewardConfig_RewardConfig">RewardConfig::RewardConfig</a>
</code></pre>




<a name="@Specification_0_get_reward_config"></a>

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



<a name="@Specification_0_reward_delay"></a>

### Function `reward_delay`


<pre><code><b>public</b> <b>fun</b> <a href="RewardConfig.md#0x1_RewardConfig_reward_delay">reward_delay</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="RewardConfig.md#0x1_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
</code></pre>
