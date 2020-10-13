
<a name="0x1_ConsensusConfig"></a>

# Module `0x1::ConsensusConfig`



-  [Struct <code><a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a></code>](#0x1_ConsensusConfig_ConsensusConfig)
-  [Resource <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a></code>](#0x1_ConsensusConfig_Epoch)
-  [Struct <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_NewEpochEvent">NewEpochEvent</a></code>](#0x1_ConsensusConfig_NewEpochEvent)
-  [Resource <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a></code>](#0x1_ConsensusConfig_EpochData)
-  [Const <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND">THOUSAND</a></code>](#0x1_ConsensusConfig_THOUSAND)
-  [Const <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND_U128">THOUSAND_U128</a></code>](#0x1_ConsensusConfig_THOUSAND_U128)
-  [Const <code><a href="ConsensusConfig.md#0x1_ConsensusConfig_HUNDRED">HUNDRED</a></code>](#0x1_ConsensusConfig_HUNDRED)
-  [Function <code>MAX_UNCLES_PER_BLOCK_IS_WRONG</code>](#0x1_ConsensusConfig_MAX_UNCLES_PER_BLOCK_IS_WRONG)
-  [Function <code>UNCLES_IS_NOT_ZERO</code>](#0x1_ConsensusConfig_UNCLES_IS_NOT_ZERO)
-  [Function <code>initialize</code>](#0x1_ConsensusConfig_initialize)
-  [Function <code>new_consensus_config</code>](#0x1_ConsensusConfig_new_consensus_config)
-  [Function <code>get_config</code>](#0x1_ConsensusConfig_get_config)
-  [Function <code>uncle_rate_target</code>](#0x1_ConsensusConfig_uncle_rate_target)
-  [Function <code>base_block_time_target</code>](#0x1_ConsensusConfig_base_block_time_target)
-  [Function <code>base_reword_per_block</code>](#0x1_ConsensusConfig_base_reword_per_block)
-  [Function <code>epoch_block_count</code>](#0x1_ConsensusConfig_epoch_block_count)
-  [Function <code>base_block_difficulty_window</code>](#0x1_ConsensusConfig_base_block_difficulty_window)
-  [Function <code>base_reward_per_uncle_percent</code>](#0x1_ConsensusConfig_base_reward_per_uncle_percent)
-  [Function <code>min_block_time_target</code>](#0x1_ConsensusConfig_min_block_time_target)
-  [Function <code>max_block_time_target</code>](#0x1_ConsensusConfig_max_block_time_target)
-  [Function <code>base_max_uncles_per_block</code>](#0x1_ConsensusConfig_base_max_uncles_per_block)
-  [Function <code>base_block_gas_limit</code>](#0x1_ConsensusConfig_base_block_gas_limit)
-  [Function <code>compute_reward_per_block</code>](#0x1_ConsensusConfig_compute_reward_per_block)
-  [Function <code>do_compute_reward_per_block</code>](#0x1_ConsensusConfig_do_compute_reward_per_block)
-  [Function <code>adjust_epoch</code>](#0x1_ConsensusConfig_adjust_epoch)
-  [Function <code>adjust_gas_limit</code>](#0x1_ConsensusConfig_adjust_gas_limit)
-  [Function <code>compute_gas_limit</code>](#0x1_ConsensusConfig_compute_gas_limit)
-  [Function <code>in_or_decrease_gas_limit</code>](#0x1_ConsensusConfig_in_or_decrease_gas_limit)
-  [Function <code>update_epoch_data</code>](#0x1_ConsensusConfig_update_epoch_data)
-  [Function <code>emit_epoch_event</code>](#0x1_ConsensusConfig_emit_epoch_event)
-  [Function <code>epoch_start_time</code>](#0x1_ConsensusConfig_epoch_start_time)
-  [Function <code>uncles</code>](#0x1_ConsensusConfig_uncles)
-  [Function <code>epoch_total_gas</code>](#0x1_ConsensusConfig_epoch_total_gas)
-  [Function <code>epoch_block_gas_limit</code>](#0x1_ConsensusConfig_epoch_block_gas_limit)
-  [Function <code>epoch_start_block_number</code>](#0x1_ConsensusConfig_epoch_start_block_number)
-  [Function <code>epoch_end_block_number</code>](#0x1_ConsensusConfig_epoch_end_block_number)
-  [Function <code>epoch_number</code>](#0x1_ConsensusConfig_epoch_number)
-  [Function <code>block_time_target</code>](#0x1_ConsensusConfig_block_time_target)
-  [Specification](#@Specification_0)


<a name="0x1_ConsensusConfig_ConsensusConfig"></a>

## Struct `ConsensusConfig`



<pre><code><b>struct</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uncle_rate_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>base_block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>base_reward_per_block: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>base_reward_per_uncle_percent: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_block_count: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>base_block_difficulty_window: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>min_block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>max_block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>base_max_uncles_per_block: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>base_block_gas_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ConsensusConfig_Epoch"></a>

## Resource `Epoch`



<pre><code><b>resource</b> <b>struct</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>epoch_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>start_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>end_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_block: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_uncle_percent: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_difficulty_window: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>max_uncles_per_block: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_gas_limit: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>new_epoch_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_NewEpochEvent">ConsensusConfig::NewEpochEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ConsensusConfig_NewEpochEvent"></a>

## Struct `NewEpochEvent`



<pre><code><b>struct</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_NewEpochEvent">NewEpochEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>epoch_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>start_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>end_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_block: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>previous_epoch_total_reward: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ConsensusConfig_EpochData"></a>

## Resource `EpochData`



<pre><code><b>resource</b> <b>struct</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uncles: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>total_reward: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>total_gas: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ConsensusConfig_THOUSAND"></a>

## Const `THOUSAND`



<pre><code><b>const</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND">THOUSAND</a>: u64 = 1000;
</code></pre>



<a name="0x1_ConsensusConfig_THOUSAND_U128"></a>

## Const `THOUSAND_U128`



<pre><code><b>const</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND_U128">THOUSAND_U128</a>: u128 = 1000;
</code></pre>



<a name="0x1_ConsensusConfig_HUNDRED"></a>

## Const `HUNDRED`



<pre><code><b>const</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_HUNDRED">HUNDRED</a>: u64 = 100;
</code></pre>



<a name="0x1_ConsensusConfig_MAX_UNCLES_PER_BLOCK_IS_WRONG"></a>

## Function `MAX_UNCLES_PER_BLOCK_IS_WRONG`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_UNCLES_IS_NOT_ZERO"></a>

## Function `UNCLES_IS_NOT_ZERO`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_initialize">initialize</a>(account: &signer, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_initialize">initialize</a>(
    account: &signer,
    uncle_rate_target: u64,
    epoch_block_count: u64,
    base_block_time_target: u64,
    base_block_difficulty_window: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>(),
    );
    <b>assert</b>(uncle_rate_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(epoch_block_count &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_reward_per_block &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_block_time_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_block_difficulty_window &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_reward_per_uncle_percent &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(min_block_time_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_max_uncles_per_block &gt;= 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(base_block_gas_limit &gt;= 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());

    move_to&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(
        account,
        <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
            epoch_number: 0,
            epoch_start_time: <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>(),
            start_number: 0,
            end_number: epoch_block_count,
            block_time_target: base_block_time_target,
            reward_per_block: base_reward_per_block,
            reward_per_uncle_percent: base_reward_per_uncle_percent,
            block_difficulty_window: base_block_difficulty_window,
            max_uncles_per_block: base_max_uncles_per_block,
            block_gas_limit: base_block_gas_limit,
            new_epoch_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_NewEpochEvent">NewEpochEvent</a>&gt;(account),
        },
    );
    move_to&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>&gt;(account, <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a> { uncles: 0, total_reward: 0, total_gas: 0 });
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">Self::ConsensusConfig</a>&gt;(
        account,
        <a href="ConsensusConfig.md#0x1_ConsensusConfig_new_consensus_config">new_consensus_config</a>(
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            epoch_block_count,
            base_block_difficulty_window,
            base_reward_per_uncle_percent,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
        ),
    );
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_new_consensus_config"></a>

## Function `new_consensus_config`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_new_consensus_config">new_consensus_config</a>(uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64): <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_new_consensus_config">new_consensus_config</a>(uncle_rate_target: u64,
                                base_block_time_target: u64,
                                base_reward_per_block: u128,
                                base_reward_per_uncle_percent: u64,
                                epoch_block_count: u64,
                                base_block_difficulty_window: u64,
                                min_block_time_target: u64,
                                max_block_time_target: u64,
                                base_max_uncles_per_block: u64,
                                base_block_gas_limit: u64,): <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a> {
    <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a> {
        uncle_rate_target,
        base_block_time_target,
        base_reward_per_block,
        epoch_block_count,
        base_block_difficulty_window,
        base_reward_per_uncle_percent,
        min_block_time_target,
        max_block_time_target,
        base_max_uncles_per_block,
        base_block_gas_limit,
    }
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_get_config"></a>

## Function `get_config`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">get_config</a>(): <a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">get_config</a>(): <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a> {
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_uncle_rate_target"></a>

## Function `uncle_rate_target`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_uncle_rate_target">uncle_rate_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_uncle_rate_target">uncle_rate_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.uncle_rate_target
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_block_time_target"></a>

## Function `base_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_time_target">base_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_time_target">base_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_time_target
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_reword_per_block"></a>

## Function `base_reword_per_block`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reword_per_block">base_reword_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reword_per_block">base_reword_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u128 {
    config.base_reward_per_block
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_block_count"></a>

## Function `epoch_block_count`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_count">epoch_block_count</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_count">epoch_block_count</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.epoch_block_count
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_block_difficulty_window"></a>

## Function `base_block_difficulty_window`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_difficulty_window">base_block_difficulty_window</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_difficulty_window">base_block_difficulty_window</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_difficulty_window
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_reward_per_uncle_percent"></a>

## Function `base_reward_per_uncle_percent`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reward_per_uncle_percent">base_reward_per_uncle_percent</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reward_per_uncle_percent">base_reward_per_uncle_percent</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_reward_per_uncle_percent
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_min_block_time_target"></a>

## Function `min_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_min_block_time_target">min_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_min_block_time_target">min_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.min_block_time_target
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_max_block_time_target"></a>

## Function `max_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_max_block_time_target">max_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_max_block_time_target">max_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.max_block_time_target
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_max_uncles_per_block"></a>

## Function `base_max_uncles_per_block`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_max_uncles_per_block">base_max_uncles_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_max_uncles_per_block">base_max_uncles_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_max_uncles_per_block
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_base_block_gas_limit"></a>

## Function `base_block_gas_limit`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_gas_limit">base_block_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_gas_limit">base_block_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_gas_limit
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_compute_reward_per_block"></a>

## Function `compute_reward_per_block`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128 {
    <b>let</b> config = <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">get_config</a>();
    <a href="ConsensusConfig.md#0x1_ConsensusConfig_do_compute_reward_per_block">do_compute_reward_per_block</a>(&config, new_epoch_block_time_target)
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_do_compute_reward_per_block"></a>

## Function `do_compute_reward_per_block`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_do_compute_reward_per_block">do_compute_reward_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, new_epoch_block_time_target: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_do_compute_reward_per_block">do_compute_reward_per_block</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, new_epoch_block_time_target: u64): u128 {
    config.base_reward_per_block *
            (new_epoch_block_time_target <b>as</b> u128) * <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND_U128">THOUSAND_U128</a> /
            (config.base_block_time_target <b>as</b> u128) / <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND_U128">THOUSAND_U128</a>
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_adjust_epoch"></a>

## Function `adjust_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, now: u64, uncles: u64, parent_gas_used: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, now: u64, uncles: u64, parent_gas_used:u64): u128
<b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>, <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a> {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>(),
    );

    <b>let</b> epoch_ref = borrow_global_mut&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>assert</b>(epoch_ref.max_uncles_per_block &gt;= uncles, <a href="ConsensusConfig.md#0x1_ConsensusConfig_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>());

    <b>let</b> epoch_data = borrow_global_mut&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> (new_epoch, reward_per_block) = <b>if</b> (block_number &lt; epoch_ref.end_number) {
        (<b>false</b>, epoch_ref.reward_per_block)
    } <b>else</b> <b>if</b> (block_number == epoch_ref.end_number) {
        //start a new epoch
        <b>assert</b>(uncles == 0, <a href="ConsensusConfig.md#0x1_ConsensusConfig_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>());
        <b>let</b> config = <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">get_config</a>();
        <b>let</b> last_epoch_time_target = epoch_ref.block_time_target;
        <b>let</b> total_time = now - epoch_ref.epoch_start_time;
        <b>let</b> total_uncles = epoch_data.uncles;
        <b>let</b> blocks = epoch_ref.end_number - epoch_ref.start_number;
        <b>let</b> avg_block_time = total_time / blocks;
        <b>let</b> uncles_rate = total_uncles * <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND">THOUSAND</a> / blocks;
        <b>let</b> new_epoch_block_time_target = (<a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND">THOUSAND</a> + uncles_rate) * avg_block_time /
            (config.uncle_rate_target + <a href="ConsensusConfig.md#0x1_ConsensusConfig_THOUSAND">THOUSAND</a>);

        <b>if</b> (new_epoch_block_time_target &lt; config.min_block_time_target) {
            new_epoch_block_time_target = config.min_block_time_target;
        };
        <b>if</b> (new_epoch_block_time_target &gt; config.max_block_time_target) {
            new_epoch_block_time_target = config.max_block_time_target;
        };
        <b>let</b> new_reward_per_block = <a href="ConsensusConfig.md#0x1_ConsensusConfig_do_compute_reward_per_block">do_compute_reward_per_block</a>(&config, new_epoch_block_time_target);

        //<b>update</b> epoch by adjust result or config, because <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a> may be updated.
        epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
        epoch_ref.epoch_start_time = now;
        epoch_ref.start_number = block_number;
        epoch_ref.end_number = block_number + config.epoch_block_count;
        epoch_ref.block_time_target = new_epoch_block_time_target;
        epoch_ref.reward_per_block = new_reward_per_block;
        epoch_ref.reward_per_uncle_percent = config.base_reward_per_uncle_percent;
        epoch_ref.block_difficulty_window = config.base_block_difficulty_window;
        epoch_ref.max_uncles_per_block = config.base_max_uncles_per_block;

        epoch_data.uncles = 0;
        <b>let</b> last_epoch_total_gas = epoch_data.total_gas + (parent_gas_used <b>as</b> u128);
        <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_gas_limit">adjust_gas_limit</a>(&config, epoch_ref, last_epoch_time_target, new_epoch_block_time_target, last_epoch_total_gas);
        <a href="ConsensusConfig.md#0x1_ConsensusConfig_emit_epoch_event">emit_epoch_event</a>(epoch_ref, epoch_data.total_reward);
        (<b>true</b>, new_reward_per_block)
    } <b>else</b> {
        //This should never happened.
        <b>abort</b> <a href="ErrorCode.md#0x1_ErrorCode_EUNREACHABLE">ErrorCode::EUNREACHABLE</a>()
    };
    <b>let</b> reward = reward_per_block +
        reward_per_block * (epoch_ref.reward_per_uncle_percent <b>as</b> u128) * (uncles <b>as</b> u128) / (<a href="ConsensusConfig.md#0x1_ConsensusConfig_HUNDRED">HUNDRED</a> <b>as</b> u128);
    <a href="ConsensusConfig.md#0x1_ConsensusConfig_update_epoch_data">update_epoch_data</a>(epoch_data, new_epoch, reward, uncles, parent_gas_used);
    reward
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_adjust_gas_limit"></a>

## Function `adjust_gas_limit`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_gas_limit">adjust_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, epoch_ref: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">ConsensusConfig::Epoch</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_gas_limit">adjust_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, epoch_ref: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas:u128) {
    <b>let</b> new_gas_limit = <a href="ConsensusConfig.md#0x1_ConsensusConfig_compute_gas_limit">compute_gas_limit</a>(config, last_epoch_time_target, new_epoch_time_target, epoch_ref.block_gas_limit, last_epoch_total_gas);
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&new_gas_limit)) {
        epoch_ref.block_gas_limit = <a href="Option.md#0x1_Option_destroy_some">Option::destroy_some</a>(new_gas_limit);
    }
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_compute_gas_limit"></a>

## Function `compute_gas_limit`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_compute_gas_limit">compute_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_compute_gas_limit">compute_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128) : <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt; {
    <b>let</b> maybe_adjust_gas_limit = (last_epoch_total_gas &gt;= <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>((last_epoch_block_gas_limit <b>as</b> u128) * (config.epoch_block_count <b>as</b> u128), (80 <b>as</b> u128), (<a href="ConsensusConfig.md#0x1_ConsensusConfig_HUNDRED">HUNDRED</a> <b>as</b> u128)));
    <b>let</b> new_gas_limit = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;u64&gt;();
    <b>if</b> (last_epoch_time_target == new_epoch_time_target) {
        <b>if</b> (new_epoch_time_target == config.min_block_time_target && !maybe_adjust_gas_limit) {
            <b>let</b> increase_gas_limit = <a href="ConsensusConfig.md#0x1_ConsensusConfig_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit, 110, config.base_block_gas_limit);
            new_gas_limit = <a href="Option.md#0x1_Option_some">Option::some</a>(increase_gas_limit);
        } <b>else</b> <b>if</b> (new_epoch_time_target == config.max_block_time_target && maybe_adjust_gas_limit) {
            <b>let</b> decrease_gas_limit = <a href="ConsensusConfig.md#0x1_ConsensusConfig_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit, 90, config.base_block_gas_limit);
            new_gas_limit = <a href="Option.md#0x1_Option_some">Option::some</a>(decrease_gas_limit);
        }
    };

    new_gas_limit
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_in_or_decrease_gas_limit"></a>

## Function `in_or_decrease_gas_limit`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64 {
    <b>let</b> tmp_gas_limit = <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>((last_epoch_block_gas_limit <b>as</b> u128), (percent <b>as</b> u128), (<a href="ConsensusConfig.md#0x1_ConsensusConfig_HUNDRED">HUNDRED</a> <b>as</b> u128));
    <b>let</b> new_gas_limit = <b>if</b> (tmp_gas_limit &gt; (min_block_gas_limit  <b>as</b> u128)) {
        (tmp_gas_limit <b>as</b> u64)
    } <b>else</b> {
        min_block_gas_limit
    };

    new_gas_limit
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_update_epoch_data"></a>

## Function `update_epoch_data`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">ConsensusConfig::EpochData</a>, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used:u64) {
    <b>if</b> (new_epoch) {
        epoch_data.total_reward = reward;
        epoch_data.uncles = uncles;
        epoch_data.total_reward = 0;
    } <b>else</b> {
        epoch_data.total_reward = epoch_data.total_reward + reward;
        epoch_data.uncles = epoch_data.uncles + uncles;
        epoch_data.total_gas = epoch_data.total_gas + (parent_gas_used <b>as</b> u128);
    }
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_emit_epoch_event"></a>

## Function `emit_epoch_event`



<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">ConsensusConfig::Epoch</a>, previous_epoch_total_reward: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>, previous_epoch_total_reward: u128) {
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> epoch_ref.new_epoch_events,
        <a href="ConsensusConfig.md#0x1_ConsensusConfig_NewEpochEvent">NewEpochEvent</a> {
            epoch_number: epoch_ref.epoch_number,
            epoch_start_time: epoch_ref.epoch_start_time,
            start_number: epoch_ref.start_number,
            end_number: epoch_ref.end_number,
            block_time_target: epoch_ref.block_time_target,
            reward_per_block: epoch_ref.reward_per_block,
            previous_epoch_total_reward,
        },
    );
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_start_time"></a>

## Function `epoch_start_time`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_start_time">epoch_start_time</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_start_time">epoch_start_time</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.epoch_start_time
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_uncles"></a>

## Function `uncles`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_uncles">uncles</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_uncles">uncles</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a> {
    <b>let</b> epoch_data = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_data.uncles
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_total_gas"></a>

## Function `epoch_total_gas`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_total_gas">epoch_total_gas</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_total_gas">epoch_total_gas</a>(): u128 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a> {
    <b>let</b> epoch_data = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_data.total_gas
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_block_gas_limit"></a>

## Function `epoch_block_gas_limit`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_gas_limit">epoch_block_gas_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_gas_limit">epoch_block_gas_limit</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.block_gas_limit
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_start_block_number"></a>

## Function `epoch_start_block_number`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_start_block_number">epoch_start_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_start_block_number">epoch_start_block_number</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.start_number
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_end_block_number"></a>

## Function `epoch_end_block_number`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_end_block_number">epoch_end_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_end_block_number">epoch_end_block_number</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.end_number
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_epoch_number"></a>

## Function `epoch_number`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_number">epoch_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_number">epoch_number</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.epoch_number
}
</code></pre>



</details>

<a name="0x1_ConsensusConfig_block_time_target"></a>

## Function `block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_block_time_target">block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_block_time_target">block_time_target</a>(): u64 <b>acquires</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.block_time_target
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
pragma aborts_if_is_strict;
</code></pre>
