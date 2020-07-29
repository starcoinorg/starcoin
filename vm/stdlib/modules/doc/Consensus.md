
<a name="0x1_Consensus"></a>

# Module `0x1::Consensus`

### Table of Contents

-  [Struct `Consensus`](#0x1_Consensus_Consensus)
-  [Resource `Epoch`](#0x1_Consensus_Epoch)
-  [Struct `NewEpochEvent`](#0x1_Consensus_NewEpochEvent)
-  [Resource `EpochData`](#0x1_Consensus_EpochData)
-  [Function `UNCLE_RATE_TARGET_IS_ZERO`](#0x1_Consensus_UNCLE_RATE_TARGET_IS_ZERO)
-  [Function `EPOCH_TIME_TARGET_IS_ZERO`](#0x1_Consensus_EPOCH_TIME_TARGET_IS_ZERO)
-  [Function `REWARD_HALF_EPOCH_IS_ZERO`](#0x1_Consensus_REWARD_HALF_EPOCH_IS_ZERO)
-  [Function `INIT_BLOCK_TIME_TARGET_IS_ZERO`](#0x1_Consensus_INIT_BLOCK_TIME_TARGET_IS_ZERO)
-  [Function `BLOCK_DIFFICULTY_WINDOW_IS_ZERO`](#0x1_Consensus_BLOCK_DIFFICULTY_WINDOW_IS_ZERO)
-  [Function `INIT_REWARD_PER_EPOCH_IS_ZERO`](#0x1_Consensus_INIT_REWARD_PER_EPOCH_IS_ZERO)
-  [Function `REWARD_PER_UNCLE_PERCENT_IS_ZERO`](#0x1_Consensus_REWARD_PER_UNCLE_PERCENT_IS_ZERO)
-  [Function `MIN_TIME_TARGET_IS_ZERO`](#0x1_Consensus_MIN_TIME_TARGET_IS_ZERO)
-  [Function `MAX_UNCLES_PER_BLOCK_IS_ZERO`](#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_ZERO)
-  [Function `initialize`](#0x1_Consensus_initialize)
-  [Function `set_uncle_rate_target`](#0x1_Consensus_set_uncle_rate_target)
-  [Function `set_epoch_time_target`](#0x1_Consensus_set_epoch_time_target)
-  [Function `set_reward_half_epoch`](#0x1_Consensus_set_reward_half_epoch)
-  [Function `get_config`](#0x1_Consensus_get_config)
-  [Function `uncle_rate_target`](#0x1_Consensus_uncle_rate_target)
-  [Function `epoch_time_target`](#0x1_Consensus_epoch_time_target)
-  [Function `min_time_target`](#0x1_Consensus_min_time_target)
-  [Function `reward_half_epoch`](#0x1_Consensus_reward_half_epoch)
-  [Function `reward_per_uncle_percent`](#0x1_Consensus_reward_per_uncle_percent)
-  [Function `max_uncles_per_block`](#0x1_Consensus_max_uncles_per_block)
-  [Function `block_difficulty_window`](#0x1_Consensus_block_difficulty_window)
-  [Function `reward_per_block`](#0x1_Consensus_reward_per_block)
-  [Function `first_epoch`](#0x1_Consensus_first_epoch)
-  [Function `adjust_epoch`](#0x1_Consensus_adjust_epoch)
-  [Function `epoch_start_time`](#0x1_Consensus_epoch_start_time)
-  [Function `uncles`](#0x1_Consensus_uncles)
-  [Function `start_number`](#0x1_Consensus_start_number)
-  [Function `end_number`](#0x1_Consensus_end_number)
-  [Function `epoch_number`](#0x1_Consensus_epoch_number)
-  [Function `block_time_target`](#0x1_Consensus_block_time_target)
-  [Function `reward_per_epoch`](#0x1_Consensus_reward_per_epoch)



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

<code>epoch_time_target: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>reward_half_epoch: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>block_difficulty_window: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>reward_per_uncle_percent: u64</code>
</dt>
<dd>

</dd>
<dt>

<code>min_time_target: u64</code>
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

<code>reward_per_epoch: u128</code>
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

<code>reward_per_epoch: u128</code>
</dt>
<dd>

</dd>
<dt>

<code>reward_per_block: u128</code>
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

<a name="0x1_Consensus_UNCLE_RATE_TARGET_IS_ZERO"></a>

## Function `UNCLE_RATE_TARGET_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_UNCLE_RATE_TARGET_IS_ZERO">UNCLE_RATE_TARGET_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_UNCLE_RATE_TARGET_IS_ZERO">UNCLE_RATE_TARGET_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 0}
</code></pre>



</details>

<a name="0x1_Consensus_EPOCH_TIME_TARGET_IS_ZERO"></a>

## Function `EPOCH_TIME_TARGET_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_EPOCH_TIME_TARGET_IS_ZERO">EPOCH_TIME_TARGET_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_EPOCH_TIME_TARGET_IS_ZERO">EPOCH_TIME_TARGET_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1}
</code></pre>



</details>

<a name="0x1_Consensus_REWARD_HALF_EPOCH_IS_ZERO"></a>

## Function `REWARD_HALF_EPOCH_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_REWARD_HALF_EPOCH_IS_ZERO">REWARD_HALF_EPOCH_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_REWARD_HALF_EPOCH_IS_ZERO">REWARD_HALF_EPOCH_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2}
</code></pre>



</details>

<a name="0x1_Consensus_INIT_BLOCK_TIME_TARGET_IS_ZERO"></a>

## Function `INIT_BLOCK_TIME_TARGET_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_INIT_BLOCK_TIME_TARGET_IS_ZERO">INIT_BLOCK_TIME_TARGET_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_INIT_BLOCK_TIME_TARGET_IS_ZERO">INIT_BLOCK_TIME_TARGET_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 3}
</code></pre>



</details>

<a name="0x1_Consensus_BLOCK_DIFFICULTY_WINDOW_IS_ZERO"></a>

## Function `BLOCK_DIFFICULTY_WINDOW_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_BLOCK_DIFFICULTY_WINDOW_IS_ZERO">BLOCK_DIFFICULTY_WINDOW_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_BLOCK_DIFFICULTY_WINDOW_IS_ZERO">BLOCK_DIFFICULTY_WINDOW_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 4}
</code></pre>



</details>

<a name="0x1_Consensus_INIT_REWARD_PER_EPOCH_IS_ZERO"></a>

## Function `INIT_REWARD_PER_EPOCH_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_INIT_REWARD_PER_EPOCH_IS_ZERO">INIT_REWARD_PER_EPOCH_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_INIT_REWARD_PER_EPOCH_IS_ZERO">INIT_REWARD_PER_EPOCH_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 5}
</code></pre>



</details>

<a name="0x1_Consensus_REWARD_PER_UNCLE_PERCENT_IS_ZERO"></a>

## Function `REWARD_PER_UNCLE_PERCENT_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_REWARD_PER_UNCLE_PERCENT_IS_ZERO">REWARD_PER_UNCLE_PERCENT_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_REWARD_PER_UNCLE_PERCENT_IS_ZERO">REWARD_PER_UNCLE_PERCENT_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 6}
</code></pre>



</details>

<a name="0x1_Consensus_MIN_TIME_TARGET_IS_ZERO"></a>

## Function `MIN_TIME_TARGET_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_MIN_TIME_TARGET_IS_ZERO">MIN_TIME_TARGET_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_MIN_TIME_TARGET_IS_ZERO">MIN_TIME_TARGET_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 7}
</code></pre>



</details>

<a name="0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_ZERO"></a>

## Function `MAX_UNCLES_PER_BLOCK_IS_ZERO`



<pre><code><b>fun</b> <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_ZERO">MAX_UNCLES_PER_BLOCK_IS_ZERO</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_ZERO">MAX_UNCLES_PER_BLOCK_IS_ZERO</a>(): u64 { <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 8}
</code></pre>



</details>

<a name="0x1_Consensus_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_initialize">initialize</a>(account: &signer, uncle_rate_target: u64, epoch_time_target: u64, reward_half_epoch: u64, init_block_time_target: u64, block_difficulty_window: u64, init_reward_per_epoch: u128, reward_per_uncle_percent: u64, min_time_target: u64, max_uncles_per_block: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_initialize">initialize</a>(account: &signer,uncle_rate_target:u64,epoch_time_target: u64,
    reward_half_epoch: u64,init_block_time_target: u64, block_difficulty_window: u64,
    init_reward_per_epoch: u128, reward_per_uncle_percent: u64,
    min_time_target:u64, max_uncles_per_block:u64) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>assert</b>(uncle_rate_target &gt; 0, <a href="#0x1_Consensus_UNCLE_RATE_TARGET_IS_ZERO">UNCLE_RATE_TARGET_IS_ZERO</a>());
    <b>assert</b>(epoch_time_target &gt; 0, <a href="#0x1_Consensus_EPOCH_TIME_TARGET_IS_ZERO">EPOCH_TIME_TARGET_IS_ZERO</a>());
    <b>assert</b>(reward_half_epoch &gt; 0, <a href="#0x1_Consensus_REWARD_HALF_EPOCH_IS_ZERO">REWARD_HALF_EPOCH_IS_ZERO</a>());
    <b>assert</b>(init_block_time_target &gt; 0, <a href="#0x1_Consensus_INIT_BLOCK_TIME_TARGET_IS_ZERO">INIT_BLOCK_TIME_TARGET_IS_ZERO</a>());
    <b>assert</b>(block_difficulty_window &gt; 0, <a href="#0x1_Consensus_BLOCK_DIFFICULTY_WINDOW_IS_ZERO">BLOCK_DIFFICULTY_WINDOW_IS_ZERO</a>());
    <b>assert</b>(init_reward_per_epoch &gt; 0, <a href="#0x1_Consensus_INIT_REWARD_PER_EPOCH_IS_ZERO">INIT_REWARD_PER_EPOCH_IS_ZERO</a>());
    <b>assert</b>(reward_per_uncle_percent &gt; 0, <a href="#0x1_Consensus_REWARD_PER_UNCLE_PERCENT_IS_ZERO">REWARD_PER_UNCLE_PERCENT_IS_ZERO</a>());
    <b>assert</b>(min_time_target &gt; 0, <a href="#0x1_Consensus_MIN_TIME_TARGET_IS_ZERO">MIN_TIME_TARGET_IS_ZERO</a>());
    <b>assert</b>(max_uncles_per_block &gt;= 0, <a href="#0x1_Consensus_MAX_UNCLES_PER_BLOCK_IS_ZERO">MAX_UNCLES_PER_BLOCK_IS_ZERO</a>());

    move_to&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(account, <a href="#0x1_Consensus_Epoch">Epoch</a> {
        epoch_number:0,
        epoch_start_time: 0,
        start_number: 0,
        end_number: 0,
        block_time_target: init_block_time_target,
        reward_per_epoch: init_reward_per_epoch,
        reward_per_block: 0,
        new_epoch_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Consensus_NewEpochEvent">NewEpochEvent</a>&gt;(account),
    });

    move_to&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(account, <a href="#0x1_Consensus_EpochData">EpochData</a> {
        uncles: 0,
        total_reward: 0,
    });

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        <a href="#0x1_Consensus">Consensus</a> {
            uncle_rate_target: uncle_rate_target,//80
            epoch_time_target : epoch_time_target, // two weeks in seconds 1209600
            reward_half_epoch: reward_half_epoch,
            block_difficulty_window: block_difficulty_window,
            reward_per_uncle_percent: reward_per_uncle_percent,
            min_time_target: min_time_target,
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

<a name="0x1_Consensus_set_epoch_time_target"></a>

## Function `set_epoch_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_epoch_time_target">set_epoch_time_target</a>(account: &signer, epoch_time_target: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_epoch_time_target">set_epoch_time_target</a>(account: &signer, epoch_time_target: u64) {
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(account);

    old_config.epoch_time_target = epoch_time_target;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        old_config,
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_set_reward_half_epoch"></a>

## Function `set_reward_half_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_reward_half_epoch">set_reward_half_epoch</a>(account: &signer, reward_half_epoch: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_set_reward_half_epoch">set_reward_half_epoch</a>(account: &signer, reward_half_epoch: u64) {
    <b>let</b> old_config = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(account);

    old_config.reward_half_epoch = reward_half_epoch;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_Consensus_Consensus">Self::Consensus</a>&gt;(
        account,
        old_config,
    );
}
</code></pre>



</details>

<a name="0x1_Consensus_get_config"></a>

## Function `get_config`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_get_config">get_config</a>(): <a href="#0x1_Consensus_Consensus">Consensus::Consensus</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_get_config">get_config</a>(): <a href="#0x1_Consensus">Consensus</a>{
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="#0x1_Consensus">Consensus</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>())
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

<a name="0x1_Consensus_epoch_time_target"></a>

## Function `epoch_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_time_target">epoch_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_epoch_time_target">epoch_time_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.epoch_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_min_time_target"></a>

## Function `min_time_target`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_min_time_target">min_time_target</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_min_time_target">min_time_target</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.min_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_reward_half_epoch"></a>

## Function `reward_half_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_half_epoch">reward_half_epoch</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_half_epoch">reward_half_epoch</a>(): u64  {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.reward_half_epoch
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



<pre><code><b>fun</b> <a href="#0x1_Consensus_block_difficulty_window">block_difficulty_window</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_block_difficulty_window">block_difficulty_window</a>(): u64 {
    <b>let</b> current_config = <a href="#0x1_Consensus_get_config">get_config</a>();
    current_config.block_difficulty_window
}
</code></pre>



</details>

<a name="0x1_Consensus_reward_per_block"></a>

## Function `reward_per_block`



<pre><code><b>fun</b> <a href="#0x1_Consensus_reward_per_block">reward_per_block</a>(blocks: u64, reward_per_epoch: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_reward_per_block">reward_per_block</a>(blocks:u64, reward_per_epoch: u128): u128 {
    <b>let</b> max_uncles = (blocks * <a href="#0x1_Consensus_uncle_rate_target">Self::uncle_rate_target</a>() * <a href="#0x1_Consensus_reward_per_uncle_percent">Self::reward_per_uncle_percent</a>()) / (THOUSAND * HUNDRED);
    <b>let</b> reward = reward_per_epoch / ((max_uncles <b>as</b> u128) + (blocks <b>as</b> u128));
    reward
}
</code></pre>



</details>

<a name="0x1_Consensus_first_epoch"></a>

## Function `first_epoch`



<pre><code><b>fun</b> <a href="#0x1_Consensus_first_epoch">first_epoch</a>(block_height: u64, block_time: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Consensus_first_epoch">first_epoch</a>(block_height: u64, block_time: u64) <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>assert</b>(block_height == 1, 333);
    <b>let</b> epoch_ref = borrow_global_mut&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    <b>let</b> count = <a href="#0x1_Consensus_epoch_time_target">Self::epoch_time_target</a>() / epoch_ref.block_time_target;
    <b>assert</b>(count &gt; 1, 336);
    epoch_ref.epoch_start_time = block_time;
    epoch_ref.start_number = 1;
    epoch_ref.end_number = epoch_ref.start_number + count;
    epoch_ref.epoch_number = epoch_ref.epoch_number + 1;
    epoch_ref.reward_per_block = <a href="#0x1_Consensus_reward_per_block">Self::reward_per_block</a>(count, epoch_ref.reward_per_epoch);
}
</code></pre>



</details>

<a name="0x1_Consensus_adjust_epoch"></a>

## Function `adjust_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_adjust_epoch">adjust_epoch</a>(account: &signer, block_height: u64, block_time: u64, uncles: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_adjust_epoch">adjust_epoch</a>(account: &signer, block_height: u64, block_time: u64, uncles: u64): u128 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a>, <a href="#0x1_Consensus_EpochData">EpochData</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>assert</b>(<a href="#0x1_Consensus_max_uncles_per_block">Self::max_uncles_per_block</a>() &gt;= uncles, 339);
    <b>if</b> (block_height == 1) {
        <b>assert</b>(uncles == 0, 334);
        <a href="#0x1_Consensus_first_epoch">Self::first_epoch</a>(block_height, block_time);
    } <b>else</b> {
        <b>let</b> epoch_ref = borrow_global_mut&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
        <b>let</b> epoch_data = borrow_global_mut&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
        <b>if</b> (block_height &lt; epoch_ref.end_number) {
            epoch_data.uncles = epoch_data.uncles + uncles;
        } <b>else</b> {
            <b>assert</b>(uncles == 0, 334);
            <b>assert</b>(block_time &gt; epoch_ref.epoch_start_time, 335);
            <b>let</b> total_time = block_time - epoch_ref.epoch_start_time;
            <b>let</b> total_uncles = epoch_data.uncles;
            <b>let</b> blocks = epoch_ref.end_number - epoch_ref.start_number;
            <b>let</b> avg_block_time = total_time / blocks;
            <b>let</b> uncles_rate = total_uncles * THOUSAND / blocks;
            <b>let</b> new_epoch_block_time_target = (THOUSAND + uncles_rate) * avg_block_time / (<a href="#0x1_Consensus_uncle_rate_target">Self::uncle_rate_target</a>() + THOUSAND);
            <b>let</b> total_reward = epoch_data.total_reward;

            <b>if</b> (new_epoch_block_time_target &lt; <a href="#0x1_Consensus_min_time_target">Self::min_time_target</a>()) {
                new_epoch_block_time_target = <a href="#0x1_Consensus_min_time_target">Self::min_time_target</a>();
            };
            <b>let</b> new_epoch_blocks = <b>if</b> ((total_time + new_epoch_block_time_target) &lt;= (<a href="#0x1_Consensus_epoch_time_target">Self::epoch_time_target</a>() * 2)) {
                <b>let</b> new_epoch_time_target = <a href="#0x1_Consensus_epoch_time_target">Self::epoch_time_target</a>() * 2 - total_time;
                new_epoch_time_target / new_epoch_block_time_target
            } <b>else</b> {
                1
            };
            <b>assert</b>(new_epoch_blocks &gt;= 1, 337);

            epoch_ref.epoch_start_time = block_time;
            epoch_data.uncles = uncles;
            epoch_ref.start_number = block_height;
            epoch_ref.end_number = block_height + new_epoch_blocks;
            epoch_ref.block_time_target = new_epoch_block_time_target;
            epoch_ref.epoch_number = epoch_ref.epoch_number + 1;

            <b>let</b> old_reward_per_epoch = epoch_ref.reward_per_epoch;
            <b>let</b> current_reward_per_epoch = <b>if</b> (epoch_ref.epoch_number % <a href="#0x1_Consensus_reward_half_epoch">Self::reward_half_epoch</a>() == 1) {
                (epoch_ref.reward_per_epoch / 2)
            } <b>else</b> {
                epoch_ref.reward_per_epoch
            };

            <b>if</b> ((old_reward_per_epoch + current_reward_per_epoch) &gt; total_reward) {
                epoch_ref.reward_per_epoch = (old_reward_per_epoch + current_reward_per_epoch) - total_reward;
            } <b>else</b> {
                epoch_ref.reward_per_epoch = 0;
            };

            epoch_ref.reward_per_block = <a href="#0x1_Consensus_reward_per_block">Self::reward_per_block</a>(new_epoch_blocks, epoch_ref.reward_per_epoch);
        }
    };

    <b>let</b> epoch_ref = borrow_global_mut&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    <b>let</b> reward = epoch_ref.reward_per_block + (epoch_ref.reward_per_block * (<a href="#0x1_Consensus_reward_per_uncle_percent">Self::reward_per_uncle_percent</a>() <b>as</b> u128) * (uncles <b>as</b> u128) / 100);

    <b>let</b> epoch_data = borrow_global_mut&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    <b>if</b> (block_height == epoch_ref.start_number) {
        epoch_data.total_reward = reward;
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> epoch_ref.new_epoch_events,
            <a href="#0x1_Consensus_NewEpochEvent">NewEpochEvent</a> {
                epoch_number: epoch_ref.epoch_number,
                epoch_start_time: epoch_ref.epoch_start_time,
                start_number: epoch_ref.start_number,
                end_number: epoch_ref.end_number,
                block_time_target: epoch_ref.block_time_target,
                reward_per_epoch: epoch_ref.reward_per_epoch,
                reward_per_block: epoch_ref.reward_per_block,
            }
        );
    } <b>else</b> {
        epoch_data.total_reward = epoch_data.total_reward + reward;
    };

    reward
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
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
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
    <b>let</b> epoch_data = borrow_global&lt;<a href="#0x1_Consensus_EpochData">EpochData</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    epoch_data.uncles
}
</code></pre>



</details>

<a name="0x1_Consensus_start_number"></a>

## Function `start_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_start_number">start_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_start_number">start_number</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    epoch_ref.start_number
}
</code></pre>



</details>

<a name="0x1_Consensus_end_number"></a>

## Function `end_number`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_end_number">end_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_end_number">end_number</a>(): u64 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
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
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
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
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    epoch_ref.block_time_target
}
</code></pre>



</details>

<a name="0x1_Consensus_reward_per_epoch"></a>

## Function `reward_per_epoch`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_per_epoch">reward_per_epoch</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Consensus_reward_per_epoch">reward_per_epoch</a>(): u128 <b>acquires</b> <a href="#0x1_Consensus_Epoch">Epoch</a> {
    <b>let</b> epoch_ref = borrow_global&lt;<a href="#0x1_Consensus_Epoch">Epoch</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    epoch_ref.reward_per_epoch
}
</code></pre>



</details>
