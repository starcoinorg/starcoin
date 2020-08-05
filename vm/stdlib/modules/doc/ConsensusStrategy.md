
<a name="0x1_ConsensusStrategy"></a>

# Module `0x1::ConsensusStrategy`

### Table of Contents

-  [Struct `ConsensusStrategy`](#0x1_ConsensusStrategy_ConsensusStrategy)
-  [Function `initialize`](#0x1_ConsensusStrategy_initialize)
-  [Function `get`](#0x1_ConsensusStrategy_get)



<a name="0x1_ConsensusStrategy_ConsensusStrategy"></a>

## Struct `ConsensusStrategy`



<pre><code><b>struct</b> <a href="#0x1_ConsensusStrategy">ConsensusStrategy</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>value: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ConsensusStrategy_initialize"></a>

## Function `initialize`

Publish the chain ID under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ConsensusStrategy_initialize">initialize</a>(account: &signer, consensus_strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ConsensusStrategy_initialize">initialize</a>(account: &signer, consensus_strategy: u8) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>()
    );
    <b>let</b> cap = <a href="Config.md#0x1_Config_publish_new_config_with_capability">Config::publish_new_config_with_capability</a>&lt;<a href="#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;(
        account,
        <a href="#0x1_ConsensusStrategy">ConsensusStrategy</a> { value:consensus_strategy }
    );
    //destory the cap, so <a href="#0x1_ConsensusStrategy">ConsensusStrategy</a> can not been change.
    <a href="Config.md#0x1_Config_destory_modify_config_capability">Config::destory_modify_config_capability</a>(cap);
}
</code></pre>



</details>

<a name="0x1_ConsensusStrategy_get"></a>

## Function `get`

Return the consensus strategy type of this chain


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ConsensusStrategy_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ConsensusStrategy_get">get</a>(): u8 {
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="#0x1_ConsensusStrategy">ConsensusStrategy</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).value
}
</code></pre>



</details>
