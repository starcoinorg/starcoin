
<a id="0x1_consensus_strategy"></a>

# Module `0x1::consensus_strategy`

The module provides the information of current consensus strategy.


-  [Struct `ConsensusStrategy`](#0x1_consensus_strategy_ConsensusStrategy)
-  [Function `initialize`](#0x1_consensus_strategy_initialize)
-  [Function `get`](#0x1_consensus_strategy_get)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `get`](#@Specification_0_get)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_consensus_strategy_ConsensusStrategy"></a>

## Struct `ConsensusStrategy`

ConsensusStrategy data.


<pre><code><b>struct</b> <a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u8</code>
</dt>
<dd>
 Value of strategy
</dd>
</dl>


</details>

<a id="0x1_consensus_strategy_initialize"></a>

## Function `initialize`

Publish the chain ID under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="consensus_strategy.md#0x1_consensus_strategy">consensus_strategy</a>: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="consensus_strategy.md#0x1_consensus_strategy">consensus_strategy</a>: u8) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> cap = <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config_with_capability">on_chain_config::publish_new_config_with_capability</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a> { value: <a href="consensus_strategy.md#0x1_consensus_strategy">consensus_strategy</a> }
    );
    //destroy the cap, so <a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a> can not been change.
    <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">on_chain_config::destroy_modify_config_capability</a>(cap);
}
</code></pre>



</details>

<a id="0x1_consensus_strategy_get"></a>

## Function `get`

Return the consensus strategy type of this chain


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_get">get</a>(): u8 {
    <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).value
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="consensus_strategy.md#0x1_consensus_strategy">consensus_strategy</a>: u8)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_0_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="consensus_strategy.md#0x1_consensus_strategy_get">get</a>(): u8
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="consensus_strategy.md#0x1_consensus_strategy_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
