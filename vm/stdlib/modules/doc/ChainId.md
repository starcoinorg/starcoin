
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`



-  [Resource <code><a href="ChainId.md#0x1_ChainId">ChainId</a></code>](#0x1_ChainId_ChainId)
-  [Function <code>initialize</code>](#0x1_ChainId_initialize)
-  [Function <code>get</code>](#0x1_ChainId_get)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>get</code>](#@Specification_0_get)


<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`



<pre><code><b>resource</b> <b>struct</b> <a href="ChainId.md#0x1_ChainId">ChainId</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_ChainId_initialize"></a>

## Function `initialize`

Publish the chain ID under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>()
    );
    move_to(account, <a href="ChainId.md#0x1_ChainId">ChainId</a> { id });
}
</code></pre>



</details>

<a name="0x1_ChainId_get"></a>

## Function `get`

Return the chain ID of this chain


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8 <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    borrow_global&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).id
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
