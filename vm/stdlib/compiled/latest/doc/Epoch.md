
<a name="0x1_Epoch"></a>

# Module `0x1::Epoch`

The module provide epoch functionality for starcoin.


-  [Resource `Epoch`](#0x1_Epoch_Epoch)
-  [Struct `NewEpochEvent`](#0x1_Epoch_NewEpochEvent)
-  [Resource `EpochData`](#0x1_Epoch_EpochData)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_Epoch_initialize)
-  [Function `compute_next_block_time_target`](#0x1_Epoch_compute_next_block_time_target)
-  [Function `adjust_epoch`](#0x1_Epoch_adjust_epoch)
-  [Function `adjust_gas_limit`](#0x1_Epoch_adjust_gas_limit)
-  [Function `compute_gas_limit`](#0x1_Epoch_compute_gas_limit)
-  [Function `in_or_decrease_gas_limit`](#0x1_Epoch_in_or_decrease_gas_limit)
-  [Function `update_epoch_data`](#0x1_Epoch_update_epoch_data)
-  [Function `emit_epoch_event`](#0x1_Epoch_emit_epoch_event)
-  [Function `start_time`](#0x1_Epoch_start_time)
-  [Function `uncles`](#0x1_Epoch_uncles)
-  [Function `total_gas`](#0x1_Epoch_total_gas)
-  [Function `block_gas_limit`](#0x1_Epoch_block_gas_limit)
-  [Function `start_block_number`](#0x1_Epoch_start_block_number)
-  [Function `end_block_number`](#0x1_Epoch_end_block_number)
-  [Function `number`](#0x1_Epoch_number)
-  [Function `block_time_target`](#0x1_Epoch_block_time_target)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `compute_next_block_time_target`](#@Specification_1_compute_next_block_time_target)
    -  [Function `adjust_epoch`](#@Specification_1_adjust_epoch)
    -  [Function `adjust_gas_limit`](#@Specification_1_adjust_gas_limit)
    -  [Function `compute_gas_limit`](#@Specification_1_compute_gas_limit)
    -  [Function `in_or_decrease_gas_limit`](#@Specification_1_in_or_decrease_gas_limit)
    -  [Function `update_epoch_data`](#@Specification_1_update_epoch_data)
    -  [Function `emit_epoch_event`](#@Specification_1_emit_epoch_event)
    -  [Function `start_time`](#@Specification_1_start_time)
    -  [Function `uncles`](#@Specification_1_uncles)
    -  [Function `total_gas`](#@Specification_1_total_gas)
    -  [Function `block_gas_limit`](#@Specification_1_block_gas_limit)
    -  [Function `start_block_number`](#@Specification_1_start_block_number)
    -  [Function `end_block_number`](#@Specification_1_end_block_number)
    -  [Function `number`](#@Specification_1_number)
    -  [Function `block_time_target`](#@Specification_1_block_time_target)


<pre><code><b>use</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">0x1::ConsensusConfig</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Math.md#0x1_Math">0x1::Math</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_Epoch_Epoch"></a>

## Resource `Epoch`

Current epoch info.


<pre><code><b>struct</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>
 Number of current epoch
</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>
 Start time of current epoch
</dd>
<dt>
<code>start_block_number: u64</code>
</dt>
<dd>
 Start block's number of current epoch
</dd>
<dt>
<code>end_block_number: u64</code>
</dt>
<dd>
 End block's number of current epoch
</dd>
<dt>
<code>block_time_target: u64</code>
</dt>
<dd>
 Average target time to calculate a block's difficulty in current epoch
</dd>
<dt>
<code>reward_per_block: u128</code>
</dt>
<dd>
 Rewards per block in current epoch
</dd>
<dt>
<code>reward_per_uncle_percent: u64</code>
</dt>
<dd>
 Percentage of <code>reward_per_block</code> to reward a uncle block in current epoch
</dd>
<dt>
<code>block_difficulty_window: u64</code>
</dt>
<dd>
 How many ancestor blocks which use to calculate next block's difficulty in current epoch
</dd>
<dt>
<code>max_uncles_per_block: u64</code>
</dt>
<dd>
 Maximum number of uncle block per block in current epoch
</dd>
<dt>
<code>block_gas_limit: u64</code>
</dt>
<dd>
 Maximum gases per block in current epoch
</dd>
<dt>
<code>strategy: u8</code>
</dt>
<dd>
 Strategy to calculate difficulty in current epoch
</dd>
<dt>
<code>new_epoch_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Epoch.md#0x1_Epoch_NewEpochEvent">Epoch::NewEpochEvent</a>&gt;</code>
</dt>
<dd>
 Switch Epoch Event
</dd>
</dl>


</details>

<a name="0x1_Epoch_NewEpochEvent"></a>

## Struct `NewEpochEvent`

New epoch event.


<pre><code><b>struct</b> <a href="Epoch.md#0x1_Epoch_NewEpochEvent">NewEpochEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>
 Epoch::number
</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>
 Epoch::start_time
</dd>
<dt>
<code>start_block_number: u64</code>
</dt>
<dd>
 Epoch::start_block_number
</dd>
<dt>
<code>end_block_number: u64</code>
</dt>
<dd>
 Epoch::end_block_number
</dd>
<dt>
<code>block_time_target: u64</code>
</dt>
<dd>
 Epoch::block_time_target
</dd>
<dt>
<code>reward_per_block: u128</code>
</dt>
<dd>
 Epoch::reward_per_block
</dd>
<dt>
<code>previous_epoch_total_reward: u128</code>
</dt>
<dd>
 Total rewards during previous epoch
</dd>
</dl>


</details>

<a name="0x1_Epoch_EpochData"></a>

## Resource `EpochData`

Epoch data.


<pre><code><b>struct</b> <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uncles: u64</code>
</dt>
<dd>
 Up to now, Number of uncle block during current epoch
</dd>
<dt>
<code>total_reward: u128</code>
</dt>
<dd>
 Up to now, Total rewards during current epoch
</dd>
<dt>
<code>total_gas: u128</code>
</dt>
<dd>
 Up to now, Total gases during current epoch
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Epoch_EINVALID_UNCLES_COUNT"></a>



<pre><code><b>const</b> <a href="Epoch.md#0x1_Epoch_EINVALID_UNCLES_COUNT">EINVALID_UNCLES_COUNT</a>: u64 = 101;
</code></pre>



<a name="0x1_Epoch_EUNREACHABLE"></a>



<pre><code><b>const</b> <a href="Epoch.md#0x1_Epoch_EUNREACHABLE">EUNREACHABLE</a>: u64 = 19;
</code></pre>



<a name="0x1_Epoch_HUNDRED"></a>



<pre><code><b>const</b> <a href="Epoch.md#0x1_Epoch_HUNDRED">HUNDRED</a>: u64 = 100;
</code></pre>



<a name="0x1_Epoch_THOUSAND"></a>



<pre><code><b>const</b> <a href="Epoch.md#0x1_Epoch_THOUSAND">THOUSAND</a>: u64 = 1000;
</code></pre>



<a name="0x1_Epoch_THOUSAND_U128"></a>



<pre><code><b>const</b> <a href="Epoch.md#0x1_Epoch_THOUSAND_U128">THOUSAND_U128</a>: u128 = 1000;
</code></pre>



<a name="0x1_Epoch_initialize"></a>

## Function `initialize`

Initialization of the module.


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_initialize">initialize</a>(
    account: &signer,
) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>let</b> config = <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">ConsensusConfig::get_config</a>();
    <b>move_to</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(
        account,
        <a href="Epoch.md#0x1_Epoch">Epoch</a> {
            number: 0,
            start_time: <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>(),
            start_block_number: 0,
            end_block_number: <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_count">ConsensusConfig::epoch_block_count</a>(&config),
            block_time_target: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_time_target">ConsensusConfig::base_block_time_target</a>(&config),
            reward_per_block: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reward_per_block">ConsensusConfig::base_reward_per_block</a>(&config),
            reward_per_uncle_percent: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reward_per_uncle_percent">ConsensusConfig::base_reward_per_uncle_percent</a>(&config),
            block_difficulty_window: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_difficulty_window">ConsensusConfig::base_block_difficulty_window</a>(&config),
            max_uncles_per_block: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_max_uncles_per_block">ConsensusConfig::base_max_uncles_per_block</a>(&config),
            block_gas_limit: <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_gas_limit">ConsensusConfig::base_block_gas_limit</a>(&config),
            strategy: <a href="ConsensusConfig.md#0x1_ConsensusConfig_strategy">ConsensusConfig::strategy</a>(&config),
            new_epoch_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Epoch.md#0x1_Epoch_NewEpochEvent">NewEpochEvent</a>&gt;(account),
        },
    );
    <b>move_to</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(account, <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a> { uncles: 0, total_reward: 0, total_gas: 0 });
}
</code></pre>



</details>

<a name="0x1_Epoch_compute_next_block_time_target"></a>

## Function `compute_next_block_time_target`

compute next block time_target.


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_next_block_time_target">compute_next_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, last_epoch_time_target: u64, epoch_start_time: u64, now_milli_second: u64, start_block_number: u64, end_block_number: u64, total_uncles: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_next_block_time_target">compute_next_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, last_epoch_time_target: u64, epoch_start_time: u64, now_milli_second: u64, start_block_number: u64, end_block_number: u64, total_uncles: u64): u64 {
    <b>let</b> total_time = now_milli_second - epoch_start_time;
    <b>let</b> blocks = end_block_number - start_block_number;
    <b>let</b> avg_block_time = total_time / blocks;
    <b>let</b> uncles_rate = total_uncles * <a href="Epoch.md#0x1_Epoch_THOUSAND">THOUSAND</a> / blocks;
    <b>let</b> new_epoch_block_time_target = (<a href="Epoch.md#0x1_Epoch_THOUSAND">THOUSAND</a> + uncles_rate) * avg_block_time /
            (<a href="ConsensusConfig.md#0x1_ConsensusConfig_uncle_rate_target">ConsensusConfig::uncle_rate_target</a>(config) + <a href="Epoch.md#0x1_Epoch_THOUSAND">THOUSAND</a>);
    <b>if</b> (new_epoch_block_time_target &gt; last_epoch_time_target * 2) {
        new_epoch_block_time_target = last_epoch_time_target * 2;
    };
    <b>if</b> (new_epoch_block_time_target &lt; last_epoch_time_target / 2) {
        new_epoch_block_time_target = last_epoch_time_target / 2;
    };
    <b>let</b> min_block_time_target = <a href="ConsensusConfig.md#0x1_ConsensusConfig_min_block_time_target">ConsensusConfig::min_block_time_target</a>(config);
    <b>let</b> max_block_time_target = <a href="ConsensusConfig.md#0x1_ConsensusConfig_max_block_time_target">ConsensusConfig::max_block_time_target</a>(config);
    <b>if</b> (new_epoch_block_time_target &lt; min_block_time_target) {
        new_epoch_block_time_target = min_block_time_target;
    };
    <b>if</b> (new_epoch_block_time_target &gt; max_block_time_target) {
        new_epoch_block_time_target = max_block_time_target;
    };
    new_epoch_block_time_target
}
</code></pre>



</details>

<a name="0x1_Epoch_adjust_epoch"></a>

## Function `adjust_epoch`

adjust_epoch try to advance to next epoch if current epoch ends.


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, timestamp: u64, uncles: u64, parent_gas_used: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, timestamp: u64, uncles: u64, parent_gas_used:u64): u128
<b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a>, <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>let</b> epoch_ref = <b>borrow_global_mut</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>assert</b>!(epoch_ref.max_uncles_per_block &gt;= uncles, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Epoch.md#0x1_Epoch_EINVALID_UNCLES_COUNT">EINVALID_UNCLES_COUNT</a>));

    <b>let</b> epoch_data = <b>borrow_global_mut</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>let</b> (new_epoch, reward_per_block) = <b>if</b> (block_number &lt; epoch_ref.end_block_number) {
        (<b>false</b>, epoch_ref.reward_per_block)
    } <b>else</b> <b>if</b> (block_number == epoch_ref.end_block_number) {
        //start a new epoch
        <b>assert</b>!(uncles == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Epoch.md#0x1_Epoch_EINVALID_UNCLES_COUNT">EINVALID_UNCLES_COUNT</a>));
        // block time target unit is milli_seconds.
        <b>let</b> now_milli_seconds = timestamp;

        <b>let</b> config = <a href="ConsensusConfig.md#0x1_ConsensusConfig_get_config">ConsensusConfig::get_config</a>();
        <b>let</b> last_epoch_time_target = epoch_ref.block_time_target;
        <b>let</b> new_epoch_block_time_target = <a href="Epoch.md#0x1_Epoch_compute_next_block_time_target">compute_next_block_time_target</a>(&config, last_epoch_time_target, epoch_ref.start_time, now_milli_seconds, epoch_ref.start_block_number, epoch_ref.end_block_number, epoch_data.uncles);
        <b>let</b> new_reward_per_block = <a href="ConsensusConfig.md#0x1_ConsensusConfig_do_compute_reward_per_block">ConsensusConfig::do_compute_reward_per_block</a>(&config, new_epoch_block_time_target);

        //<b>update</b> epoch by adjust result or config, because <a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a> may be updated.
        epoch_ref.number = epoch_ref.number + 1;
        epoch_ref.start_time = now_milli_seconds;
        epoch_ref.start_block_number = block_number;
        epoch_ref.end_block_number = block_number + <a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_count">ConsensusConfig::epoch_block_count</a>(&config);
        epoch_ref.block_time_target = new_epoch_block_time_target;
        epoch_ref.reward_per_block = new_reward_per_block;
        epoch_ref.reward_per_uncle_percent = <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_reward_per_uncle_percent">ConsensusConfig::base_reward_per_uncle_percent</a>(&config);
        epoch_ref.block_difficulty_window = <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_difficulty_window">ConsensusConfig::base_block_difficulty_window</a>(&config);
        epoch_ref.max_uncles_per_block = <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_max_uncles_per_block">ConsensusConfig::base_max_uncles_per_block</a>(&config);
        epoch_ref.strategy = <a href="ConsensusConfig.md#0x1_ConsensusConfig_strategy">ConsensusConfig::strategy</a>(&config);

        epoch_data.uncles = 0;
        <b>let</b> last_epoch_total_gas = epoch_data.total_gas + (parent_gas_used <b>as</b> u128);
        <a href="Epoch.md#0x1_Epoch_adjust_gas_limit">adjust_gas_limit</a>(&config, epoch_ref, last_epoch_time_target, new_epoch_block_time_target, last_epoch_total_gas);
        <a href="Epoch.md#0x1_Epoch_emit_epoch_event">emit_epoch_event</a>(epoch_ref, epoch_data.total_reward);
        (<b>true</b>, new_reward_per_block)
    } <b>else</b> {
        //This should never happened.
        <b>abort</b> <a href="Epoch.md#0x1_Epoch_EUNREACHABLE">EUNREACHABLE</a>
    };
    <b>let</b> reward = reward_per_block +
            reward_per_block * (epoch_ref.reward_per_uncle_percent <b>as</b> u128) * (uncles <b>as</b> u128) / (<a href="Epoch.md#0x1_Epoch_HUNDRED">HUNDRED</a> <b>as</b> u128);
    <a href="Epoch.md#0x1_Epoch_update_epoch_data">update_epoch_data</a>(epoch_data, new_epoch, reward, uncles, parent_gas_used);
    reward
}
</code></pre>



</details>

<a name="0x1_Epoch_adjust_gas_limit"></a>

## Function `adjust_gas_limit`



<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_gas_limit">adjust_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_Epoch">Epoch::Epoch</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_gas_limit">adjust_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch">Epoch</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas:u128) {
    <b>let</b> new_gas_limit = <a href="Epoch.md#0x1_Epoch_compute_gas_limit">compute_gas_limit</a>(config, last_epoch_time_target, new_epoch_time_target, epoch_ref.block_gas_limit, last_epoch_total_gas);
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&new_gas_limit)) {
        epoch_ref.block_gas_limit = <a href="Option.md#0x1_Option_destroy_some">Option::destroy_some</a>(new_gas_limit);
    }
}
</code></pre>



</details>

<a name="0x1_Epoch_compute_gas_limit"></a>

## Function `compute_gas_limit`

Compute block's gas limit of next epoch.


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_gas_limit">compute_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_gas_limit">compute_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128) : <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt; {
    <b>let</b> epoch_block_count = (<a href="ConsensusConfig.md#0x1_ConsensusConfig_epoch_block_count">ConsensusConfig::epoch_block_count</a>(config) <b>as</b> u128);
    <b>let</b> gas_limit_threshold = (last_epoch_total_gas &gt;= <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>((last_epoch_block_gas_limit <b>as</b> u128) * epoch_block_count, (80 <b>as</b> u128), (<a href="Epoch.md#0x1_Epoch_HUNDRED">HUNDRED</a> <b>as</b> u128)));
    <b>let</b> new_gas_limit = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;u64&gt;();

    <b>let</b> min_block_time_target = <a href="ConsensusConfig.md#0x1_ConsensusConfig_min_block_time_target">ConsensusConfig::min_block_time_target</a>(config);
    <b>let</b> max_block_time_target = <a href="ConsensusConfig.md#0x1_ConsensusConfig_max_block_time_target">ConsensusConfig::max_block_time_target</a>(config);
    <b>let</b> base_block_gas_limit =  <a href="ConsensusConfig.md#0x1_ConsensusConfig_base_block_gas_limit">ConsensusConfig::base_block_gas_limit</a>(config);
    <b>if</b> (last_epoch_time_target == new_epoch_time_target) {
        <b>if</b> (new_epoch_time_target == min_block_time_target && gas_limit_threshold) {
            <b>let</b> increase_gas_limit = <a href="Epoch.md#0x1_Epoch_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit, 110, base_block_gas_limit);
            new_gas_limit = <a href="Option.md#0x1_Option_some">Option::some</a>(increase_gas_limit);
        } <b>else</b> <b>if</b> (new_epoch_time_target == max_block_time_target && !gas_limit_threshold) {
            <b>let</b> decrease_gas_limit = <a href="Epoch.md#0x1_Epoch_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit, 90, base_block_gas_limit);
            new_gas_limit = <a href="Option.md#0x1_Option_some">Option::some</a>(decrease_gas_limit);
        }
    };

    new_gas_limit
}
</code></pre>



</details>

<a name="0x1_Epoch_in_or_decrease_gas_limit"></a>

## Function `in_or_decrease_gas_limit`



<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64 {
    <b>let</b> tmp_gas_limit = <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>((last_epoch_block_gas_limit <b>as</b> u128), (percent <b>as</b> u128), (<a href="Epoch.md#0x1_Epoch_HUNDRED">HUNDRED</a> <b>as</b> u128));
    <b>let</b> new_gas_limit = <b>if</b> (tmp_gas_limit &gt; (min_block_gas_limit  <b>as</b> u128)) {
        (tmp_gas_limit <b>as</b> u64)
    } <b>else</b> {
        min_block_gas_limit
    };

    new_gas_limit
}
</code></pre>



</details>

<a name="0x1_Epoch_update_epoch_data"></a>

## Function `update_epoch_data`



<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_EpochData">Epoch::EpochData</a>, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used:u64) {
    <b>if</b> (new_epoch) {
        epoch_data.total_reward = reward;
        epoch_data.uncles = uncles;
        epoch_data.total_gas = 0;
    } <b>else</b> {
        epoch_data.total_reward = epoch_data.total_reward + reward;
        epoch_data.uncles = epoch_data.uncles + uncles;
        epoch_data.total_gas = epoch_data.total_gas + (parent_gas_used <b>as</b> u128);
    }
}
</code></pre>



</details>

<a name="0x1_Epoch_emit_epoch_event"></a>

## Function `emit_epoch_event`



<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_Epoch">Epoch::Epoch</a>, previous_epoch_total_reward: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch">Epoch</a>, previous_epoch_total_reward: u128) {
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> epoch_ref.new_epoch_events,
        <a href="Epoch.md#0x1_Epoch_NewEpochEvent">NewEpochEvent</a> {
            number: epoch_ref.number,
            start_time: epoch_ref.start_time,
            start_block_number: epoch_ref.start_block_number,
            end_block_number: epoch_ref.end_block_number,
            block_time_target: epoch_ref.block_time_target,
            reward_per_block: epoch_ref.reward_per_block,
            previous_epoch_total_reward,
        },
    );
}
</code></pre>



</details>

<a name="0x1_Epoch_start_time"></a>

## Function `start_time`

Get start time of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_time">start_time</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_time">start_time</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.start_time
}
</code></pre>



</details>

<a name="0x1_Epoch_uncles"></a>

## Function `uncles`

Get uncles number of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_uncles">uncles</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_uncles">uncles</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a> {
    <b>let</b> epoch_data = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_data.uncles
}
</code></pre>



</details>

<a name="0x1_Epoch_total_gas"></a>

## Function `total_gas`

Get total gas of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_total_gas">total_gas</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_total_gas">total_gas</a>(): u128 <b>acquires</b> <a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a> {
    <b>let</b> epoch_data = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_data.total_gas
}
</code></pre>



</details>

<a name="0x1_Epoch_block_gas_limit"></a>

## Function `block_gas_limit`

Get block's gas_limit of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_gas_limit">block_gas_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_gas_limit">block_gas_limit</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.block_gas_limit
}
</code></pre>



</details>

<a name="0x1_Epoch_start_block_number"></a>

## Function `start_block_number`

Get start block's number of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_block_number">start_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_block_number">start_block_number</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.start_block_number
}
</code></pre>



</details>

<a name="0x1_Epoch_end_block_number"></a>

## Function `end_block_number`

Get end block's number of current epoch


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_end_block_number">end_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_end_block_number">end_block_number</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.end_block_number
}
</code></pre>



</details>

<a name="0x1_Epoch_number"></a>

## Function `number`

Get current epoch number


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_number">number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_number">number</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.number
}
</code></pre>



</details>

<a name="0x1_Epoch_block_time_target"></a>

## Function `block_time_target`

Get current block time target


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_time_target">block_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_time_target">block_time_target</a>(): u64 <b>acquires</b> <a href="Epoch.md#0x1_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = <b>borrow_global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    epoch_ref.block_time_target
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="ConsensusConfig.md#0x1_ConsensusConfig">ConsensusConfig</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_compute_next_block_time_target"></a>

### Function `compute_next_block_time_target`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_next_block_time_target">compute_next_block_time_target</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, last_epoch_time_target: u64, epoch_start_time: u64, now_milli_second: u64, start_block_number: u64, end_block_number: u64, total_uncles: u64): u64
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_adjust_epoch"></a>

### Function `adjust_epoch`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_epoch">adjust_epoch</a>(account: &signer, block_number: u64, timestamp: u64, uncles: u64, parent_gas_used: u64): u128
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).max_uncles_per_block &lt; uncles;
<b>aborts_if</b> <b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> block_number == <b>global</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).end_block_number && uncles != 0;
</code></pre>



<a name="@Specification_1_adjust_gas_limit"></a>

### Function `adjust_gas_limit`


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_adjust_gas_limit">adjust_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_Epoch">Epoch::Epoch</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_total_gas: u128)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_compute_gas_limit"></a>

### Function `compute_gas_limit`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_compute_gas_limit">compute_gas_limit</a>(config: &<a href="ConsensusConfig.md#0x1_ConsensusConfig_ConsensusConfig">ConsensusConfig::ConsensusConfig</a>, last_epoch_time_target: u64, new_epoch_time_target: u64, last_epoch_block_gas_limit: u64, last_epoch_total_gas: u128): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_in_or_decrease_gas_limit"></a>

### Function `in_or_decrease_gas_limit`


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_in_or_decrease_gas_limit">in_or_decrease_gas_limit</a>(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64
</code></pre>




<pre><code><b>include</b> <a href="Math.md#0x1_Math_MulDivAbortsIf">Math::MulDivAbortsIf</a>{x: last_epoch_block_gas_limit, y: percent, z: <a href="Epoch.md#0x1_Epoch_HUNDRED">HUNDRED</a>};
<b>aborts_if</b> <a href="Math.md#0x1_Math_spec_mul_div">Math::spec_mul_div</a>() &gt; MAX_U64;
</code></pre>



<a name="@Specification_1_update_epoch_data"></a>

### Function `update_epoch_data`


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_update_epoch_data">update_epoch_data</a>(epoch_data: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_EpochData">Epoch::EpochData</a>, new_epoch: bool, reward: u128, uncles: u64, parent_gas_used: u64)
</code></pre>




<pre><code><b>aborts_if</b> !new_epoch && epoch_data.total_reward + reward &gt; MAX_U128;
<b>aborts_if</b> !new_epoch && epoch_data.uncles + uncles &gt; MAX_U64;
<b>aborts_if</b> !new_epoch && epoch_data.total_gas + parent_gas_used &gt; MAX_U128;
</code></pre>



<a name="@Specification_1_emit_epoch_event"></a>

### Function `emit_epoch_event`


<pre><code><b>fun</b> <a href="Epoch.md#0x1_Epoch_emit_epoch_event">emit_epoch_event</a>(epoch_ref: &<b>mut</b> <a href="Epoch.md#0x1_Epoch_Epoch">Epoch::Epoch</a>, previous_epoch_total_reward: u128)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_start_time"></a>

### Function `start_time`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_time">start_time</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_uncles"></a>

### Function `uncles`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_uncles">uncles</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_total_gas"></a>

### Function `total_gas`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_total_gas">total_gas</a>(): u128
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_block_gas_limit"></a>

### Function `block_gas_limit`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_gas_limit">block_gas_limit</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_start_block_number"></a>

### Function `start_block_number`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_start_block_number">start_block_number</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_end_block_number"></a>

### Function `end_block_number`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_end_block_number">end_block_number</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_number"></a>

### Function `number`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_number">number</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_block_time_target"></a>

### Function `block_time_target`


<pre><code><b>public</b> <b>fun</b> <a href="Epoch.md#0x1_Epoch_block_time_target">block_time_target</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Epoch.md#0x1_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
