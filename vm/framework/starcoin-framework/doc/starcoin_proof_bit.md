
<a id="0x1_starcoin_proof_bit"></a>

# Module `0x1::starcoin_proof_bit`



-  [Function `get_bit`](#0x1_starcoin_proof_bit_get_bit)


<pre><code></code></pre>



<a id="0x1_starcoin_proof_bit_get_bit"></a>

## Function `get_bit`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof_bit.md#0x1_starcoin_proof_bit_get_bit">get_bit</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof_bit.md#0x1_starcoin_proof_bit_get_bit">get_bit</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, index: u64): bool {
    <b>let</b> pos = index / 8;
    <b>let</b> bit = (7 - index % 8);
    (*<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(data, pos) &gt;&gt; (bit <b>as</b> u8)) & 1u8 != 0
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
