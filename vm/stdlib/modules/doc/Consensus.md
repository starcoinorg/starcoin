
<a name="0x1_Consensus"></a>

# Module `0x1::Consensus`

### Table of Contents

-  [Struct `Consensus`](#0x1_Consensus_Consensus)
-  [Resource `Epoch`](#0x1_Consensus_Epoch)
-  [Struct `NewEpochEvent`](#0x1_Consensus_NewEpochEvent)
-  [Resource `EpochData`](#0x1_Consensus_EpochData)
-  [Const `THOUSAND`](#0x1_Consensus_THOUSAND)
-  [Const `THOUSAND_U128`](#0x1_Consensus_THOUSAND_U128)
-  [Const `HUNDRED`](#0x1_Consensus_HUNDRED)
-  [Function `MAX_UNCLES_PER_BLOCK_IS_WRONG`](#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_WRONG)
-  [Function `UNCLES_IS_NOT_ZERO`](#0x1_Consensus_UNCLES_IS_NOT_ZERO)
-  [Function `initialize`](#0x1_Consensus_initialize)
-  [Function `set_uncle_rate_target`](#0x1_Consensus_set_uncle_rate_target)
-  [Function `set_epoch_block_count`](#0x1_Consensus_set_epoch_block_count)
-  [Function `set_min_block_time_target`](#0x1_Consensus_set_min_block_time_target)
-  [Function `get_config`](#0x1_Consensus_get_config)
-  [Function `uncle_rate_target`](#0x1_Consensus_uncle_rate_target)
-  [Function `epoch_block_count`](#0x1_Consensus_epoch_block_count)
-  [Function `init_block_time_target`](#0x1_Consensus_init_block_time_target)
-  [Function `min_block_time_target`](#0x1_Consensus_min_block_time_target)
-  [Function `max_block_time_target`](#0x1_Consensus_max_block_time_target)
-  [Function `reward_per_uncle_percent`](#0x1_Consensus_reward_per_uncle_percent)
-  [Function `max_uncles_per_block`](#0x1_Consensus_max_uncles_per_block)
-  [Function `block_difficulty_window`](#0x1_Consensus_block_difficulty_window)
-  [Function `compute_reward_per_block`](#0x1_Consensus_compute_reward_per_block)
-  [Function `adjust_epoch`](#0x1_Consensus_adjust_epoch)
-  [Function `update_epoch_data`](#0x1_Consensus_update_epoch_data)
-  [Function `emit_epoch_event`](#0x1_Consensus_emit_epoch_event)
-  [Function `epoch_start_time`](#0x1_Consensus_epoch_start_time)
-  [Function `uncles`](#0x1_Consensus_uncles)
-  [Function `epoch_start_block_number`](#0x1_Consensus_epoch_start_block_number)
-  [Function `epoch_end_block_number`](#0x1_Consensus_epoch_end_block_number)
-  [Function `epoch_number`](#0x1_Consensus_epoch_number)
-  [Function `block_time_target`](#0x1_Consensus_block_time_target)
-  [Specification](#0x1_Consensus_Specification)



<a name="0x1_Consensus_Consensus"></a>

## Struct `Consensus`



<pre><code><b>struct</b> <a href="#0x1_Consensus">Consensus</a>
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
<code>init_block_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>init_reward_per_block: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>reward_per_uncle_percent: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>epoch_block_count: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_difficulty_window: u64</code>
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
<code>max_uncles_per_block: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Consensus_Epoch"></a>

## Resource `Epoch`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Consensus_Epoch">Epoch</a>
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
<code>new_epoch_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Consensus_NewEpochEvent">Consensus::NewEpochEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Consensus_NewEpochEvent"></a>

## Struct `NewEpochEvent`



<pre><code><b>struct</b> <a href="#0x1_Consensus_NewEpochEvent">NewEpochEvent</a>
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

<a name="0x1_Consensus_EpochData"></a>

## Resource `EpochData`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Consensus_EpochData">EpochData</a>
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
</dl>


</details>

<a name="0x1_Consensus_THOUSAND"></a>

## Const `THOUSAND`



<pre><code><b>const</b> <a href="#0x1_Consensus_THOUSAND">THOUSAND</a>: u64 = 1000;
</code></pre>



<a name="0x1_Consensus_THOUSAND_U128"></a>

## Const `THOUSAND_U128`



<pre><code><b>const</b> <a href="#0x1_Consensus_THOUSAND_U128">THOUSAND_U128</a>: u128 = 1000;
</code></pre>



<a name="0x1_Consensus_HUNDRED"></a>

## Const `HUNDRED`



<pre><code><b>const</b> <a href="#0x1_Consensus_HUNDRED">HUNDRED</a>: u64 = 100;
</code></pre>



<a name="0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_WRONG"></a>

## Function `MAX_UNCLES_PER_BLOCK_IS_WRONG`



<pre><code><b>fun</b> <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1}
</code></pre>



</details>

<a name="0x1_Consensus_UNCLES_IS_NOT_ZERO"></a>

## Function `UNCLES_IS_NOT_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2}
</code></pre>



</details>

<a name="0x1_Consensus_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_initialize">initialize</a>(account: &signer, uncle_rate_target: u64, epoch_block_count: u64, init_block_time_target: u64, block_difficulty_window: u64, init_reward_per_block: u128, reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, max_uncles_per_block: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_initialize">initialize</a>(account: &signer, uncle_rate_target:u64,epoch_block_count: u64,
    init_block_time_target: u64, block_difficulty_window: u64,
    init_reward_per_block: u128, reward_per_uncle_percent: u64,
    min_block_time_target:u64, max_block_time_target: u64, max_uncles_per_block:u64) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    <b>assert</b>(uncle_rate_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(epoch_block_count &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(init_reward_per_block &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(init_block_time_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(block_difficulty_window &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(reward_per_uncle_percent &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(min_block_time_target &gt; 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());
    <b>assert</b>(max_uncles_per_block &gt;= 0, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());

    move_to&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(account, <a href="#0x1_Consensus_Epoch">Epoch</a> {
        epoch_number:0,
        epoch_start_time: <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>(),
        start_number: 0,
        end_number: epoch_block_count,
        block_time_target: init_block_time_target,
        reward_per_block: init_reward_per_block,
        new_epoch_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Consensus_NewEpochEvent">NewEpochEvent</a>&gt;(account),
    });

    move_to&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(account, <a href="#0x1_Consensus_EpochData">EpochData</a> {
        uncles: 0,
        total_reward: 0,
    });

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        <a href="#0x1_Consensus">Consensus</a> {
            uncle_rate_target: uncle_rate_target,
            init_block_time_target: init_block_time_target,
            init_reward_per_block: init_reward_per_block,
            epoch_block_count : epoch_block_count,
            block_difficulty_window: block_difficulty_window,
            reward_per_uncle_percent: reward_per_uncle_percent,
            min_block_time_target: min_block_time_target,
            max_block_time_target: max_block_time_target,
            max_uncles_per_block: max_uncles_per_block,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_set_uncle_rate_target"></a>

## Function `set_uncle_rate_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_uncle_rate_target">set_uncle_rate_target</a>(account: &signer, uncle_rate_target: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_uncle_rate_target">set_uncle_rate_target</a>(account: &signer, uncle_rate_target:u64) {
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(account);

    old_config.uncle_rate_target = uncle_rate_target;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        old_config,
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_set_epoch_block_count"></a>

## Function `set_epoch_block_count`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_epoch_block_count">set_epoch_block_count</a>(account: &signer, epoch_block_count: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_epoch_block_count">set_epoch_block_count</a>(account: &signer, epoch_block_count: u64) {
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(account);

    old_config.epoch_block_count = epoch_block_count;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        old_config,
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_set_min_block_time_target"></a>

## Function `set_min_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_min_block_time_target">set_min_block_time_target</a>(account: &signer, min_block_time_target: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_min_block_time_target">set_min_block_time_target</a>(account: &signer, min_block_time_target: u64) {
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(account);

    old_config.min_block_time_target = min_block_time_target;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        old_config,
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_get_config"></a>

## Function `get_config`



<pre><code><b>fun</b> <a href="#0x1_Consensus_get_config">get_config</a>(): <a href="#0x1_Consensus_Consensus">Consensus::Consensus</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_get_config">get_config</a>(): <a href="#0x1_Consensus">Consensus</a>{
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="#0x1_Consensus">Consensus</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_Consensus_uncle_rate_target"></a>

## Function `uncle_rate_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_uncle_rate_target">uncle_rate_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_uncle_rate_target">uncle_rate_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.uncle_rate_target
}
</code></pre>



</details>

<a name="0x1_Consensus_epoch_block_count"></a>

## Function `epoch_block_count`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_block_count">epoch_block_count</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_block_count">epoch_block_count</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.epoch_block_count
}
</code></pre>



</details>

<a name="0x1_Consensus_init_block_time_target"></a>

## Function `init_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_init_block_time_target">init_block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_init_block_time_target">init_block_time_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.init_block_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_min_block_time_target"></a>

## Function `min_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_min_block_time_target">min_block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_min_block_time_target">min_block_time_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.min_block_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_max_block_time_target"></a>

## Function `max_block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_max_block_time_target">max_block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_max_block_time_target">max_block_time_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.max_block_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_reward_per_uncle_percent"></a>

## Function `reward_per_uncle_percent`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_per_uncle_percent">reward_per_uncle_percent</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_per_uncle_percent">reward_per_uncle_percent</a>(): u64 {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.reward_per_uncle_percent
}
</code></pre>



</details>

<a name="0x1_Consensus_max_uncles_per_block"></a>

## Function `max_uncles_per_block`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_max_uncles_per_block">max_uncles_per_block</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_max_uncles_per_block">max_uncles_per_block</a>():u64 {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.max_uncles_per_block
}
</code></pre>



</details>

<a name="0x1_Consensus_block_difficulty_window"></a>

## Function `block_difficulty_window`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_block_difficulty_window">block_difficulty_window</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_block_difficulty_window">block_difficulty_window</a>(): u64 {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.block_difficulty_window
}
</code></pre>



</details>

<a name="0x1_Consensus_compute_reward_per_block"></a>

## Function `compute_reward_per_block`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128 {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    (current_config.init_reward_per_block * ((new_epoch_block_time_target <b>as</b> u128) * <a href="#0x1_Consensus_THOUSAND_U128">THOUSAND_U128</a>/(current_config.init_block_time_target <b>as</b> u128)))/<a href="#0x1_Consensus_THOUSAND_U128">THOUSAND_U128</a>
}
</code></pre>



</details>

<a name="0x1_Consensus_adjust_epoch"></a>

## Function `adjust_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, now: u64, uncles: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, now: u64, uncles: u64): u128 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a>, <a href="#0x1_Consensus_EpochData">EpochData</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>assert</b>(<a href="#0x1_Consensus_max_uncles_per_block">Self::max_uncles_per_block</a>() &gt;= uncles, <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_WRONG">MAX_UNCLES_PER_BLOCK_IS_WRONG</a>());

    <b>let</b> epoch_ref = borrow_global_mut&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> epoch_data = borrow_global_mut&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> (new_epoch,reward_per_block) = <b>if</b> (block_number &lt; epoch_ref.end_number) {
        (<b>false</b>, epoch_ref.reward_per_block)
    } <b>else</b> <b>if</b>(block_number == epoch_ref.end_number){
        //start a new epoch
        <b>assert</b>(uncles == 0, <a href="#0x1_Consensus_UNCLES_IS_NOT_ZERO">UNCLES_IS_NOT_ZERO</a>());
        <b>let</b> config = <a href="#0x1_Consensus_get_config">get_config</a>();

        <b>let</b> total_time = now - epoch_ref.epoch_start_time;
        <b>let</b> total_uncles = epoch_data.uncles;
        <b>let</b> blocks = epoch_ref.end_number - epoch_ref.start_number;
        <b>let</b> avg_block_time = total_time / blocks;
        <b>let</b> uncles_rate = total_uncles * <a href="#0x1_Consensus_THOUSAND">THOUSAND</a> / blocks;
        <b>let</b> new_epoch_block_time_target = (<a href="#0x1_Consensus_THOUSAND">THOUSAND</a> + uncles_rate) * avg_block_time / (config.uncle_rate_target + <a href="#0x1_Consensus_THOUSAND">THOUSAND</a>);

        <b>if</b> (new_epoch_block_time_target &lt; config.min_block_time_target) {
            new_epoch_block_time_target = config.min_block_time_target;
        };
        <b>if</b> (new_epoch_block_time_target &gt; config.max_block_time_target) {
            new_epoch_block_time_target = config.max_block_time_target;
        };
        <b>let</b> new_reward_per_block = <a href="#0x1_Consensus_compute_reward_per_block">Self::compute_reward_per_block</a>(new_epoch_block_time_target);
        epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
        epoch_ref.epoch_start_time = now;
        epoch_data.uncles = uncles;
        epoch_ref.start_number = block_number;
        epoch_ref.end_number = block_number + config.epoch_block_count;
        epoch_ref.block_time_target = new_epoch_block_time_target;
        epoch_ref.reward_per_block = new_reward_per_block;
        <a href="#0x1_Consensus_emit_epoch_event">emit_epoch_event</a>(epoch_ref, epoch_data.total_reward);
        (<b>true</b>, new_reward_per_block)
    }<b>else</b>{
        //This should never happend.
        <b>abort</b>(<a href="ErrorCode.md#0x1_ErrorCode_EUNREACHABLE">ErrorCode::EUNREACHABLE</a>())
    };

    <b>let</b> reward = reward_per_block + (reward_per_block * (<a href="#0x1_Consensus_reward_per_uncle_percent">Self::reward_per_uncle_percent</a>() <b>as</b> u128) * (uncles <b>as</b> u128) / 100);
    <a href="#0x1_Consensus_update_epoch_data">update_epoch_data</a>(epoch_data, new_epoch, reward, uncles);
    reward
}
</code></pre>



</details>

<a name="0x1_Consensus_update_epoch_data"></a>

## Function `update_epoch_data`



<pre><code><b>fun</b> <a href="#0x1_Consensus_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="#0x1_Consensus_EpochData">Consensus::EpochData</a>, new_epoch: bool, reward: u128, uncles: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="#0x1_Consensus_EpochData">EpochData</a>, new_epoch: bool, reward: u128, uncles: u64) {
    <b>if</b> (new_epoch){
        epoch_data.total_reward = reward;
        epoch_data.uncles = uncles;
    }<b>else</b>{
        epoch_data.total_reward = epoch_data.total_reward + reward;
        epoch_data.uncles = epoch_data.uncles + uncles;
    }
}
</code></pre>



</details>

<a name="0x1_Consensus_emit_epoch_event"></a>

## Function `emit_epoch_event`



<pre><code><b>fun</b> <a href="#0x1_Consensus_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="#0x1_Consensus_Epoch">Consensus::Epoch</a>, previous_epoch_total_reward: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="#0x1_Consensus_Epoch">Epoch</a>, previous_epoch_total_reward: u128){
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> epoch_ref.new_epoch_events,
        <a href="#0x1_Consensus_NewEpochEvent">NewEpochEvent</a> {
            epoch_number: epoch_ref.epoch_number,
            epoch_start_time: epoch_ref.epoch_start_time,
            start_number: epoch_ref.start_number,
            end_number: epoch_ref.end_number,
            block_time_target: epoch_ref.block_time_target,
            reward_per_block: epoch_ref.reward_per_block,
            previous_epoch_total_reward: previous_epoch_total_reward,
        }
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_epoch_start_time"></a>

## Function `epoch_start_time`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_start_time">epoch_start_time</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_start_time">epoch_start_time</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.epoch_start_time
}
</code></pre>



</details>

<a name="0x1_Consensus_uncles"></a>

## Function `uncles`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_uncles">uncles</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_uncles">uncles</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_EpochData">EpochData</a> {
    <b>let</b> epoch_data = borrow_global&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_data.uncles
}
</code></pre>



</details>

<a name="0x1_Consensus_epoch_start_block_number"></a>

## Function `epoch_start_block_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_start_block_number">epoch_start_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_start_block_number">epoch_start_block_number</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.start_number
}
</code></pre>



</details>

<a name="0x1_Consensus_epoch_end_block_number"></a>

## Function `epoch_end_block_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_end_block_number">epoch_end_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_end_block_number">epoch_end_block_number</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.end_number
}
</code></pre>



</details>

<a name="0x1_Consensus_epoch_number"></a>

## Function `epoch_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_number">epoch_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_number">epoch_number</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.epoch_number
}
</code></pre>



</details>

<a name="0x1_Consensus_block_time_target"></a>

## Function `block_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_block_time_target">block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_block_time_target">block_time_target</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.block_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_Specification"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
pragma aborts_if_is_strict;
</code></pre>
