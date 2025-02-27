
<a id="0x1_starcoin_proof_structured_hash"></a>

# Module `0x1::starcoin_proof_structured_hash`



-  [Constants](#@Constants_0)
-  [Function `hash`](#0x1_starcoin_proof_structured_hash_hash)
-  [Function `concat`](#0x1_starcoin_proof_structured_hash_concat)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/bcs.md#0x1_bcs">0x1::bcs</a>;
<b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_starcoin_proof_structured_hash_STARCOIN_HASH_PREFIX"></a>



<pre><code><b>const</b> <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_STARCOIN_HASH_PREFIX">STARCOIN_HASH_PREFIX</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [83, 84, 65, 82, 67, 79, 73, 78, 58, 58];
</code></pre>



<a id="0x1_starcoin_proof_structured_hash_hash"></a>

## Function `hash`



<pre><code><b>public</b> <b>fun</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">hash</a>&lt;MoveValue: store&gt;(structure: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data: &MoveValue): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">hash</a>&lt;MoveValue: store&gt;(structure: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data: &MoveValue): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> prefix_hash = <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(<a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_concat">concat</a>(&<a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_STARCOIN_HASH_PREFIX">STARCOIN_HASH_PREFIX</a>, structure));
    <b>let</b> bcs_bytes = <a href="../../move-stdlib/doc/bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(data);
    <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(<a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_concat">concat</a>(&prefix_hash, bcs_bytes))
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_structured_hash_concat"></a>

## Function `concat`



<pre><code><b>fun</b> <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_concat">concat</a>(v1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, v2: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_concat">concat</a>(v1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, v2: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> data = *v1;
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> data, v2);
    data
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
