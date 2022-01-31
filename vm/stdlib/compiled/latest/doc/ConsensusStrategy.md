
<a name="0x1_ConsensusStrategy"></a>

# Module `0x1::ConsensusStrategy`

The module provides the information of current consensus strategy.


-  [Struct `ConsensusStrategy`](#0x1_ConsensusStrategy_ConsensusStrategy)
-  [Function `initialize`](#0x1_ConsensusStrategy_initialize)
-  [Function `get`](#0x1_ConsensusStrategy_get)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `get`](#@Specification_0_get)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_ConsensusStrategy_ConsensusStrategy"></a>

## Struct `ConsensusStrategy`

ConsensusStrategy data.


<pre><code><b>struct</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="0x1_ConsensusStrategy_initialize"></a>

## Function `initialize`

Publish the chain ID under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">initialize</a>(account: &signer, consensus_strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">initialize</a>(account: &signer, consensus_strategy: u8) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>let</b> cap = <a href="Config.md#0x1_Config_publish_new_config_with_capability">Config::publish_new_config_with_capability</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;(
        account,
        <a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a> { value:consensus_strategy }
    );
    //destroy the cap, so <a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a> can not been change.
    <a href="Config.md#0x1_Config_destroy_modify_config_capability">Config::destroy_modify_config_capability</a>(cap);
}
</code></pre>



</details>

<a name="0x1_ConsensusStrategy_get"></a>

## Function `get`

Return the consensus strategy type of this chain


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_get">get</a>(): u8 {
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).value
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_initialize">initialize</a>(account: &signer, consensus_strategy: u8)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_0_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="ConsensusStrategy.md#0x1_ConsensusStrategy_get">get</a>(): u8
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="ConsensusStrategy.md#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
