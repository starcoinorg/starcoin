
<a id="0x1_bcs"></a>

# Module `0x1::bcs`

Utility for converting a Move value to its binary representation in BCS (Binary Canonical
Serialization). BCS is the binary encoding for Move resources and other non-module values
published on-chain. See https://github.com/starcoin-labs/bcs#binary-canonical-serialization-bcs for more
details on BCS.


-  [Function `to_bytes`](#0x1_bcs_to_bytes)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



<a id="0x1_bcs_to_bytes"></a>

## Function `to_bytes`

Returns the binary representation of <code>v</code> in BCS (Binary Canonical Serialization) format.
Aborts with <code>0x1c5</code> error code if serialization fails.


<pre><code><b>public</b> <b>fun</b> <a href="bcs.md#0x1_bcs_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="bcs.md#0x1_bcs_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



Native function which is defined in the prover's prelude.


<a id="0x1_bcs_serialize"></a>


<pre><code><b>native</b> <b>fun</b> <a href="bcs.md#0x1_bcs_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): <a href="vector.md#0x1_vector">vector</a>&lt;u8&gt;;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
