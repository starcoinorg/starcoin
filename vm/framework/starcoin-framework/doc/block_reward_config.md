
<a id="0x1_block_reward_config"></a>

# Module `0x1::block_reward_config`

The module provide configuration for block reward.


-  [Struct `RewardConfig`](#0x1_block_reward_config_RewardConfig)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_block_reward_config_initialize)
-  [Function `new_reward_config`](#0x1_block_reward_config_new_reward_config)
-  [Function `get_reward_config`](#0x1_block_reward_config_get_reward_config)
-  [Function `reward_delay`](#0x1_block_reward_config_reward_delay)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `new_reward_config`](#@Specification_1_new_reward_config)
    -  [Function `get_reward_config`](#@Specification_1_get_reward_config)
    -  [Function `reward_delay`](#@Specification_1_reward_delay)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_block_reward_config_RewardConfig"></a>

## Struct `RewardConfig`

Reward configuration


<pre><code><b>struct</b> <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a> <b>has</b> <b>copy</b>, drop, store
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_block_reward_config_EINVALID_ARGUMENT"></a>



<pre><code><b>const</b> <a href="block_reward_config.md#0x1_block_reward_config_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>: u64 = 18;
</code></pre>



<a id="0x1_block_reward_config_initialize"></a>

## Function `initialize`

Module initialization.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">Self::RewardConfig</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="block_reward_config.md#0x1_block_reward_config_new_reward_config">new_reward_config</a>(reward_delay)
    );
}
</code></pre>



</details>

<a id="0x1_block_reward_config_new_reward_config"></a>

## Function `new_reward_config`

Create a new reward config mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a> {
    <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a> { reward_delay: reward_delay }
}
</code></pre>



</details>

<a id="0x1_block_reward_config_get_reward_config"></a>

## Function `get_reward_config`

Get reward configuration.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_get_reward_config">get_reward_config</a>(): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_get_reward_config">get_reward_config</a>(): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a> {
    <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>())
}
</code></pre>



</details>

<a id="0x1_block_reward_config_reward_delay"></a>

## Function `reward_delay`

Get reward delay.


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_reward_delay">reward_delay</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_reward_delay">reward_delay</a>(): u64 {
    <b>let</b> reward_config = <a href="block_reward_config.md#0x1_block_reward_config_get_reward_config">get_reward_config</a>();
    reward_config.reward_delay
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, reward_delay: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">on_chain_config::PublishNewConfigAbortsIf</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">on_chain_config::PublishNewConfigEnsures</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;;
</code></pre>



<a id="@Specification_1_new_reward_config"></a>

### Function `new_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_new_reward_config">new_reward_config</a>(reward_delay: u64): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>
</code></pre>




<a id="@Specification_1_get_reward_config"></a>

### Function `get_reward_config`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_get_reward_config">get_reward_config</a>(): <a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">block_reward_config::RewardConfig</a>
</code></pre>




<pre><code><b>include</b> <a href="block_reward_config.md#0x1_block_reward_config_GetRewardConfigAbortsIf">GetRewardConfigAbortsIf</a>;
</code></pre>




<a id="0x1_block_reward_config_GetRewardConfigAbortsIf"></a>


<pre><code><b>schema</b> <a href="block_reward_config.md#0x1_block_reward_config_GetRewardConfigAbortsIf">GetRewardConfigAbortsIf</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
}
</code></pre>



<a id="@Specification_1_reward_delay"></a>

### Function `reward_delay`


<pre><code><b>public</b> <b>fun</b> <a href="block_reward_config.md#0x1_block_reward_config_reward_delay">reward_delay</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="block_reward_config.md#0x1_block_reward_config_RewardConfig">RewardConfig</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
