
<a name="0x1_ChainId"></a>

# Module `0x1::ChainId`

The module provides chain id information.


-  [Resource `ChainId`](#0x1_ChainId_ChainId)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_ChainId_initialize)
-  [Function `get`](#0x1_ChainId_get)
-  [Function `is_dev`](#0x1_ChainId_is_dev)
-  [Function `is_test`](#0x1_ChainId_is_test)
-  [Function `is_halley`](#0x1_ChainId_is_halley)
-  [Function `is_proxima`](#0x1_ChainId_is_proxima)
-  [Function `is_barnard`](#0x1_ChainId_is_barnard)
-  [Function `is_main`](#0x1_ChainId_is_main)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `get`](#@Specification_1_get)
    -  [Function `is_dev`](#@Specification_1_is_dev)
    -  [Function `is_test`](#@Specification_1_is_test)
    -  [Function `is_halley`](#@Specification_1_is_halley)
    -  [Function `is_proxima`](#@Specification_1_is_proxima)
    -  [Function `is_barnard`](#@Specification_1_is_barnard)
    -  [Function `is_main`](#@Specification_1_is_main)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_ChainId_ChainId"></a>

## Resource `ChainId`

chain id data structure.


<pre><code><b>struct</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u8</code>
</dt>
<dd>
 real id.
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_ChainId_BARNARD_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_BARNARD_CHAIN_ID">BARNARD_CHAIN_ID</a>: u8 = 251;
</code></pre>



<a name="0x1_ChainId_DEV_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_DEV_CHAIN_ID">DEV_CHAIN_ID</a>: u8 = 254;
</code></pre>



<a name="0x1_ChainId_HALLEY_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_HALLEY_CHAIN_ID">HALLEY_CHAIN_ID</a>: u8 = 253;
</code></pre>



<a name="0x1_ChainId_MAIN_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_MAIN_CHAIN_ID">MAIN_CHAIN_ID</a>: u8 = 1;
</code></pre>



<a name="0x1_ChainId_PROXIMA_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_PROXIMA_CHAIN_ID">PROXIMA_CHAIN_ID</a>: u8 = 252;
</code></pre>



<a name="0x1_ChainId_TEST_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="ChainId.md#0x1_ChainId_TEST_CHAIN_ID">TEST_CHAIN_ID</a>: u8 = 255;
</code></pre>



<a name="0x1_ChainId_initialize"></a>

## Function `initialize`

Publish the chain ID under the genesis account


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>move_to</b>(account, <a href="ChainId.md#0x1_ChainId">ChainId</a> { id });
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
    <b>borrow_global</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).id
}
</code></pre>



</details>

<a name="0x1_ChainId_is_dev"></a>

## Function `is_dev`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_dev">is_dev</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_dev">is_dev</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_DEV_CHAIN_ID">DEV_CHAIN_ID</a>
}
</code></pre>



</details>

<a name="0x1_ChainId_is_test"></a>

## Function `is_test`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_test">is_test</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_test">is_test</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_TEST_CHAIN_ID">TEST_CHAIN_ID</a>
}
</code></pre>



</details>

<a name="0x1_ChainId_is_halley"></a>

## Function `is_halley`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_halley">is_halley</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_halley">is_halley</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_HALLEY_CHAIN_ID">HALLEY_CHAIN_ID</a>
}
</code></pre>



</details>

<a name="0x1_ChainId_is_proxima"></a>

## Function `is_proxima`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_proxima">is_proxima</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_proxima">is_proxima</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_PROXIMA_CHAIN_ID">PROXIMA_CHAIN_ID</a>
}
</code></pre>



</details>

<a name="0x1_ChainId_is_barnard"></a>

## Function `is_barnard`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_barnard">is_barnard</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_barnard">is_barnard</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_BARNARD_CHAIN_ID">BARNARD_CHAIN_ID</a>
}
</code></pre>



</details>

<a name="0x1_ChainId_is_main"></a>

## Function `is_main`



<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_main">is_main</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_main">is_main</a>(): bool <b>acquires</b> <a href="ChainId.md#0x1_ChainId">ChainId</a> {
    <a href="ChainId.md#0x1_ChainId_get">get</a>() == <a href="ChainId.md#0x1_ChainId_MAIN_CHAIN_ID">MAIN_CHAIN_ID</a>
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


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_initialize">initialize</a>(account: &signer, id: u8)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_get"></a>

### Function `get`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_get">get</a>(): u8
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_dev"></a>

### Function `is_dev`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_dev">is_dev</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_test"></a>

### Function `is_test`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_test">is_test</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_halley"></a>

### Function `is_halley`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_halley">is_halley</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_proxima"></a>

### Function `is_proxima`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_proxima">is_proxima</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_barnard"></a>

### Function `is_barnard`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_barnard">is_barnard</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_is_main"></a>

### Function `is_main`


<pre><code><b>public</b> <b>fun</b> <a href="ChainId.md#0x1_ChainId_is_main">is_main</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId">ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
