
<a name="0x1_Hash"></a>

# Module `0x1::Hash`

The module provide sha-hash functionality for Move.


-  [Function `sha2_256`](#0x1_Hash_sha2_256)
-  [Function `sha3_256`](#0x1_Hash_sha3_256)
-  [Function `keccak_256`](#0x1_Hash_keccak_256)
-  [Function `ripemd160`](#0x1_Hash_ripemd160)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



<a name="0x1_Hash_sha2_256"></a>

## Function `sha2_256`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha2_256">sha2_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha2_256">sha2_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_Hash_sha3_256"></a>

## Function `sha3_256`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha3_256">sha3_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha3_256">sha3_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_Hash_keccak_256"></a>

## Function `keccak_256`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_keccak_256">keccak_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_keccak_256">keccak_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_Hash_ripemd160"></a>

## Function `ripemd160`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_ripemd160">ripemd160</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_ripemd160">ripemd160</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>
