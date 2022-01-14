
<a name="0x1_BCS"></a>

# Module `0x1::BCS`

Utility for converting a Move value to its binary representation in BCS (Diem Canonical
Serialization). BCS is the binary encoding for Move resources and other non-module values
published on-chain.


-  [Function `to_bytes`](#0x1_BCS_to_bytes)
-  [Function `to_address`](#0x1_BCS_to_address)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



<a name="0x1_BCS_to_bytes"></a>

## Function `to_bytes`

Return the binary representation of <code>v</code> in BCS (Starcoin Canonical Serialization) format


<pre><code><b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_bytes">to_bytes</a>&lt;MoveValue: store&gt;(v: &MoveValue): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_bytes">to_bytes</a>&lt;MoveValue: store&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_BCS_to_address"></a>

## Function `to_address`

Return the address of key bytes


<pre><code><b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_address">to_address</a>(key_bytes: vector&lt;u8&gt;): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="BCS.md#0x1_BCS_to_address">to_address</a>(key_bytes: vector&lt;u8&gt;): <b>address</b>;
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a name="0x1_BCS_serialize"></a>


<pre><code><b>native</b> <b>fun</b> <a href="BCS.md#0x1_BCS_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>
