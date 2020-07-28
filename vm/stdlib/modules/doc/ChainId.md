
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`

### Table of Contents

-  [Resource `ChainId`](#0x1_ChainId_ChainId)
-  [Function `initialize`](#0x1_ChainId_initialize)
-  [Function `get`](#0x1_ChainId_get)



<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_ChainId">ChainId</a>
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), ENOT_GENESIS);
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(),
        ENOT_GENESIS_ACCOUNT
    );

    move_to(account, <a href="#0x1_ChainId">ChainId</a> { id })
}
</code></pre>



</details>

<a name="0x1_ChainId_get"></a>

## Function `get`

Return the chain ID of this Libra instance


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_get">get</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_ChainId_get">get</a>(): u8 <b>acquires</b> <a href="#0x1_ChainId">ChainId</a> {
    borrow_global&lt;<a href="#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>()).id
}
</code></pre>



</details>
