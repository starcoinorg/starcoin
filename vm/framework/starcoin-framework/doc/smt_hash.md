
<a id="0x1_smt_hash"></a>

# Module `0x1::smt_hash`



-  [Constants](#@Constants_0)
-  [Function `size`](#0x1_smt_hash_size)
-  [Function `hash`](#0x1_smt_hash_hash)
-  [Function `size_zero_bytes`](#0x1_smt_hash_size_zero_bytes)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_smt_hash_SIZE_ZERO_BYTES"></a>



<pre><code><b>const</b> <a href="smt_hash.md#0x1_smt_hash_SIZE_ZERO_BYTES">SIZE_ZERO_BYTES</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
</code></pre>



<a id="0x1_smt_hash_size"></a>

## Function `size`



<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_size">size</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_size">size</a>(): u64 {
    32
}
</code></pre>



</details>

<a id="0x1_smt_hash_hash"></a>

## Function `hash`



<pre><code><b>public</b> <b>fun</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">hash</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">hash</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(*data)
}
</code></pre>



</details>

<a id="0x1_smt_hash_size_zero_bytes"></a>

## Function `size_zero_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_size_zero_bytes">size_zero_bytes</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_size_zero_bytes">size_zero_bytes</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="smt_hash.md#0x1_smt_hash_SIZE_ZERO_BYTES">SIZE_ZERO_BYTES</a>
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
