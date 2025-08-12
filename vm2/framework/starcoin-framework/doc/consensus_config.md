
<a id="0x1_consensus_config"></a>

# Module `0x1::consensus_config`

The module provide configuration of consensus parameters.


-  [Struct `ConsensusConfig`](#0x1_consensus_config_ConsensusConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_consensus_config_initialize)
-  [Function `new_consensus_config`](#0x1_consensus_config_new_consensus_config)
-  [Function `get_config`](#0x1_consensus_config_get_config)
-  [Function `uncle_rate_target`](#0x1_consensus_config_uncle_rate_target)
-  [Function `base_block_time_target`](#0x1_consensus_config_base_block_time_target)
-  [Function `base_reward_per_block`](#0x1_consensus_config_base_reward_per_block)
-  [Function `epoch_block_count`](#0x1_consensus_config_epoch_block_count)
-  [Function `base_block_difficulty_window`](#0x1_consensus_config_base_block_difficulty_window)
-  [Function `base_reward_per_uncle_percent`](#0x1_consensus_config_base_reward_per_uncle_percent)
-  [Function `min_block_time_target`](#0x1_consensus_config_min_block_time_target)
-  [Function `max_block_time_target`](#0x1_consensus_config_max_block_time_target)
-  [Function `base_max_uncles_per_block`](#0x1_consensus_config_base_max_uncles_per_block)
-  [Function `base_block_gas_limit`](#0x1_consensus_config_base_block_gas_limit)
-  [Function `strategy`](#0x1_consensus_config_strategy)
-  [Function `compute_reward_per_block`](#0x1_consensus_config_compute_reward_per_block)
-  [Function `do_compute_reward_per_block`](#0x1_consensus_config_do_compute_reward_per_block)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `new_consensus_config`](#@Specification_1_new_consensus_config)
    -  [Function `get_config`](#@Specification_1_get_config)
    -  [Function `compute_reward_per_block`](#@Specification_1_compute_reward_per_block)
    -  [Function `do_compute_reward_per_block`](#@Specification_1_do_compute_reward_per_block)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_consensus_config_ConsensusConfig"></a>

## Struct `ConsensusConfig`

consensus configurations.


<pre><code><b>struct</b> <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uncle_rate_target: u64</code>
</dt>
<dd>
 Uncle block rate per epoch
</dd>
<dt>
<code>base_block_time_target: u64</code>
</dt>
<dd>
 Average target time to calculate a block's difficulty
</dd>
<dt>
<code>base_reward_per_block: u128</code>
</dt>
<dd>
 Rewards per block
</dd>
<dt>
<code>base_reward_per_uncle_percent: u64</code>
</dt>
<dd>
 Percentage of <code>base_reward_per_block</code> to reward a uncle block
</dd>
<dt>
<code>epoch_block_count: u64</code>
</dt>
<dd>
 Blocks each epoch
</dd>
<dt>
<code>base_block_difficulty_window: u64</code>
</dt>
<dd>
 How many ancestor blocks which use to calculate next block's difficulty
</dd>
<dt>
<code>min_block_time_target: u64</code>
</dt>
<dd>
 Minimum target time to calculate a block's difficulty
</dd>
<dt>
<code>max_block_time_target: u64</code>
</dt>
<dd>
 Maximum target time to calculate a block's difficulty
</dd>
<dt>
<code>base_max_uncles_per_block: u64</code>
</dt>
<dd>
 Maximum number of uncle block per block
</dd>
<dt>
<code>base_block_gas_limit: u64</code>
</dt>
<dd>
 Maximum gases per block
</dd>
<dt>
<code>strategy: u8</code>
</dt>
<dd>
 Strategy to calculate difficulty
</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_consensus_config_EINVALID_ARGUMENT"></a>



<pre><code><b>const</b> <a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>: u64 = 18;
</code></pre>



<a id="0x1_consensus_config_initialize"></a>

## Function `initialize`

Initialization of the module.


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_initialize">initialize</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
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
    strategy: u8,
) {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">Self::ConsensusConfig</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="consensus_config.md#0x1_consensus_config_new_consensus_config">new_consensus_config</a>(
            uncle_rate_target,
            base_block_time_target,
            base_reward_per_block,
            base_reward_per_uncle_percent,
            epoch_block_count,
            base_block_difficulty_window,
            min_block_time_target,
            max_block_time_target,
            base_max_uncles_per_block,
            base_block_gas_limit,
            strategy,
        ),
    );
}
</code></pre>



</details>

<a id="0x1_consensus_config_new_consensus_config"></a>

## Function `new_consensus_config`

Create a new consensus config mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_new_consensus_config">new_consensus_config</a>(uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_new_consensus_config">new_consensus_config</a>(
    uncle_rate_target: u64,
    base_block_time_target: u64,
    base_reward_per_block: u128,
    base_reward_per_uncle_percent: u64,
    epoch_block_count: u64,
    base_block_difficulty_window: u64,
    min_block_time_target: u64,
    max_block_time_target: u64,
    base_max_uncles_per_block: u64,
    base_block_gas_limit: u64,
    strategy: u8,
): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a> {
    <b>assert</b>!(uncle_rate_target &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    <b>assert</b>!(base_block_time_target &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    <b>assert</b>!(base_reward_per_block &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    <b>assert</b>!(epoch_block_count &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    <b>assert</b>!(base_block_difficulty_window &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    // base_reward_per_uncle_percent can been zero.
    <b>assert</b>!(min_block_time_target &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));
    <b>assert</b>!(max_block_time_target &gt;= min_block_time_target, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="consensus_config.md#0x1_consensus_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>));

    <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a> {
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
        strategy,
    }
}
</code></pre>



</details>

<a id="0x1_consensus_config_get_config"></a>

## Function `get_config`

Get config data.


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_get_config">get_config</a>(): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_get_config">get_config</a>(): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a> {
    <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>())
}
</code></pre>



</details>

<a id="0x1_consensus_config_uncle_rate_target"></a>

## Function `uncle_rate_target`

Get uncle_rate_target


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_uncle_rate_target">uncle_rate_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_uncle_rate_target">uncle_rate_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.uncle_rate_target
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_block_time_target"></a>

## Function `base_block_time_target`

Get base_block_time_target


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_time_target">base_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_time_target">base_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_time_target
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_reward_per_block"></a>

## Function `base_reward_per_block`

Get base_reward_per_block


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_reward_per_block">base_reward_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_reward_per_block">base_reward_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u128 {
    config.base_reward_per_block
}
</code></pre>



</details>

<a id="0x1_consensus_config_epoch_block_count"></a>

## Function `epoch_block_count`

Get epoch_block_count


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_epoch_block_count">epoch_block_count</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_epoch_block_count">epoch_block_count</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.epoch_block_count
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_block_difficulty_window"></a>

## Function `base_block_difficulty_window`

Get base_block_difficulty_window


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_difficulty_window">base_block_difficulty_window</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_difficulty_window">base_block_difficulty_window</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_difficulty_window
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_reward_per_uncle_percent"></a>

## Function `base_reward_per_uncle_percent`

Get base_reward_per_uncle_percent


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_reward_per_uncle_percent">base_reward_per_uncle_percent</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_reward_per_uncle_percent">base_reward_per_uncle_percent</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_reward_per_uncle_percent
}
</code></pre>



</details>

<a id="0x1_consensus_config_min_block_time_target"></a>

## Function `min_block_time_target`

Get min_block_time_target


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_min_block_time_target">min_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_min_block_time_target">min_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.min_block_time_target
}
</code></pre>



</details>

<a id="0x1_consensus_config_max_block_time_target"></a>

## Function `max_block_time_target`

Get max_block_time_target


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_max_block_time_target">max_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_max_block_time_target">max_block_time_target</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.max_block_time_target
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_max_uncles_per_block"></a>

## Function `base_max_uncles_per_block`

Get base_max_uncles_per_block


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_max_uncles_per_block">base_max_uncles_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_max_uncles_per_block">base_max_uncles_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_max_uncles_per_block
}
</code></pre>



</details>

<a id="0x1_consensus_config_base_block_gas_limit"></a>

## Function `base_block_gas_limit`

Get base_block_gas_limit


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_gas_limit">base_block_gas_limit</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_base_block_gas_limit">base_block_gas_limit</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u64 {
    config.base_block_gas_limit
}
</code></pre>



</details>

<a id="0x1_consensus_config_strategy"></a>

## Function `strategy`

Get strategy


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_strategy">strategy</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_strategy">strategy</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>): u8 {
    config.strategy
}
</code></pre>



</details>

<a id="0x1_consensus_config_compute_reward_per_block"></a>

## Function `compute_reward_per_block`

Compute block reward given the <code>new_epoch_block_time_target</code>.


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128 {
    <b>let</b> config = <a href="consensus_config.md#0x1_consensus_config_get_config">get_config</a>();
    <a href="consensus_config.md#0x1_consensus_config_do_compute_reward_per_block">do_compute_reward_per_block</a>(&config, new_epoch_block_time_target)
}
</code></pre>



</details>

<a id="0x1_consensus_config_do_compute_reward_per_block"></a>

## Function `do_compute_reward_per_block`

Compute block reward given the <code>new_epoch_block_time_target</code>, and the consensus config.


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_do_compute_reward_per_block">do_compute_reward_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>, new_epoch_block_time_target: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_do_compute_reward_per_block">do_compute_reward_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>, new_epoch_block_time_target: u64): u128 {
    <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(
        config.base_reward_per_block,
        (new_epoch_block_time_target <b>as</b> u128),
        (config.base_block_time_target <b>as</b> u128)
    )
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b>(<b>friend</b>) <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, uncle_rate_target: u64, epoch_block_count: u64, base_block_time_target: u64, base_block_difficulty_window: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> uncle_rate_target == 0;
<b>aborts_if</b> epoch_block_count == 0;
<b>aborts_if</b> base_reward_per_block == 0;
<b>aborts_if</b> base_block_time_target == 0;
<b>aborts_if</b> base_block_difficulty_window == 0;
<b>aborts_if</b> min_block_time_target == 0;
<b>aborts_if</b> <a href="consensus_config.md#0x1_consensus_config_max_block_time_target">max_block_time_target</a> &lt; min_block_time_target;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">on_chain_config::PublishNewConfigAbortsIf</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">on_chain_config::PublishNewConfigEnsures</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;;
</code></pre>



<a id="@Specification_1_new_consensus_config"></a>

### Function `new_consensus_config`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_new_consensus_config">new_consensus_config</a>(uncle_rate_target: u64, base_block_time_target: u64, base_reward_per_block: u128, base_reward_per_uncle_percent: u64, epoch_block_count: u64, base_block_difficulty_window: u64, min_block_time_target: u64, max_block_time_target: u64, base_max_uncles_per_block: u64, base_block_gas_limit: u64, strategy: u8): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>
</code></pre>




<pre><code><b>aborts_if</b> uncle_rate_target == 0;
<b>aborts_if</b> epoch_block_count == 0;
<b>aborts_if</b> base_reward_per_block == 0;
<b>aborts_if</b> base_block_time_target == 0;
<b>aborts_if</b> base_block_difficulty_window == 0;
<b>aborts_if</b> min_block_time_target == 0;
<b>aborts_if</b> <a href="consensus_config.md#0x1_consensus_config_max_block_time_target">max_block_time_target</a> &lt; min_block_time_target;
</code></pre>



<a id="@Specification_1_get_config"></a>

### Function `get_config`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_get_config">get_config</a>(): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>




<a id="0x1_consensus_config_spec_get_config"></a>


<pre><code><b>fun</b> <a href="consensus_config.md#0x1_consensus_config_spec_get_config">spec_get_config</a>(): <a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a> {
   <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).payload
}
</code></pre>



<a id="@Specification_1_compute_reward_per_block"></a>

### Function `compute_reward_per_block`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_compute_reward_per_block">compute_reward_per_block</a>(new_epoch_block_time_target: u64): u128
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">ConsensusConfig</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>include</b> <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_MulDivAbortsIf">math128::MulDivAbortsIf</a> {
    a: <a href="consensus_config.md#0x1_consensus_config_spec_get_config">spec_get_config</a>().base_reward_per_block,
    b: new_epoch_block_time_target,
    c: <a href="consensus_config.md#0x1_consensus_config_spec_get_config">spec_get_config</a>().base_block_time_target
};
</code></pre>



<a id="@Specification_1_do_compute_reward_per_block"></a>

### Function `do_compute_reward_per_block`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_config.md#0x1_consensus_config_do_compute_reward_per_block">do_compute_reward_per_block</a>(config: &<a href="consensus_config.md#0x1_consensus_config_ConsensusConfig">consensus_config::ConsensusConfig</a>, new_epoch_block_time_target: u64): u128
</code></pre>




<pre><code><b>include</b> <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_MulDivAbortsIf">math128::MulDivAbortsIf</a> {
    a: config.base_reward_per_block,
    b: new_epoch_block_time_target,
    c: config.base_block_time_target
};
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
