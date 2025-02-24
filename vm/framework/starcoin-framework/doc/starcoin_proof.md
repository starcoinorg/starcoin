
<a id="0x1_starcoin_proof_verifier"></a>

# Module `0x1::starcoin_proof_verifier`



-  [Resource `StarcoinMerkle`](#0x1_starcoin_proof_verifier_StarcoinMerkle)
-  [Struct `Node`](#0x1_starcoin_proof_verifier_Node)
-  [Constants](#@Constants_0)
-  [Function `create`](#0x1_starcoin_proof_verifier_create)
-  [Function `verify_on`](#0x1_starcoin_proof_verifier_verify_on)
-  [Function `verify`](#0x1_starcoin_proof_verifier_verify)
-  [Function `computer_root_hash`](#0x1_starcoin_proof_verifier_computer_root_hash)
-  [Function `splite_symbol`](#0x1_starcoin_proof_verifier_splite_symbol)
-  [Function `split`](#0x1_starcoin_proof_verifier_split)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="starcoin_proof_bit.md#0x1_starcoin_proof_bit">0x1::starcoin_proof_bit</a>;
<b>use</b> <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash">0x1::starcoin_proof_structured_hash</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_starcoin_proof_verifier_StarcoinMerkle"></a>

## Resource `StarcoinMerkle`



<pre><code><b>struct</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_StarcoinMerkle">StarcoinMerkle</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>merkle_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_starcoin_proof_verifier_Node"></a>

## Struct `Node`



<pre><code><b>struct</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_Node">Node</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>hash1: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>hash2: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_starcoin_proof_verifier_DELMITER"></a>



<pre><code><b>const</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_DELMITER">DELMITER</a>: u8 = 124;
</code></pre>



<a id="0x1_starcoin_proof_verifier_HASH_LEN_IN_BIT"></a>



<pre><code><b>const</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_HASH_LEN_IN_BIT">HASH_LEN_IN_BIT</a>: u64 = 256;
</code></pre>



<a id="0x1_starcoin_proof_verifier_SPARSE_MERKLE_INTERNAL_NODE"></a>



<pre><code><b>const</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_SPARSE_MERKLE_INTERNAL_NODE">SPARSE_MERKLE_INTERNAL_NODE</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [83, 112, 97, 114, 115, 101, 77, 101, 114, 107, 108, 101, 73, 110, 116, 101, 114, 110, 97, 108, 78, 111, 100, 101];
</code></pre>



<a id="0x1_starcoin_proof_verifier_SPARSE_MERKLE_LEAF_NODE"></a>



<pre><code><b>const</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_SPARSE_MERKLE_LEAF_NODE">SPARSE_MERKLE_LEAF_NODE</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [83, 112, 97, 114, 115, 101, 77, 101, 114, 107, 108, 101, 76, 101, 97, 102, 78, 111, 100, 101];
</code></pre>



<a id="0x1_starcoin_proof_verifier_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_create">create</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, merkle_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_create">create</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, merkle_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> s = <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_StarcoinMerkle">StarcoinMerkle</a> {
        merkle_root
    };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, s);
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_verifier_verify_on"></a>

## Function `verify_on`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_verify_on">verify_on</a>(merkle_address: <b>address</b>, account_address: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, account_state_root_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_verify_on">verify_on</a>(
    merkle_address: <b>address</b>,
    account_address: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    account_state_root_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): bool
<b>acquires</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_StarcoinMerkle">StarcoinMerkle</a> {
    <b>let</b> merkle = <b>borrow_global</b>&lt;<a href="starcoin_proof.md#0x1_starcoin_proof_verifier_StarcoinMerkle">StarcoinMerkle</a>&gt;(merkle_address);
    <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_verify">verify</a>(*&merkle.merkle_root, account_address, account_state_root_hash, proofs)
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_verifier_verify"></a>

## Function `verify`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_verify">verify</a>(expected_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, account_address: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, account_state_root_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_verify">verify</a>(
    expected_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    account_address: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    account_state_root_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): bool {
    <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_computer_root_hash">Self::computer_root_hash</a>(<a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(account_address), account_state_root_hash, proofs) == expected_root
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_verifier_computer_root_hash"></a>

## Function `computer_root_hash`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_computer_root_hash">computer_root_hash</a>(element_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, element_blob_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_computer_root_hash">computer_root_hash</a>(
    element_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    element_blob_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proofs: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> leaf_node = <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_Node">Node</a> { hash1: element_key, hash2: element_blob_hash };
    <b>let</b> current_hash = <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_hash">starcoin_proof_structured_hash::hash</a>(<a href="starcoin_proof.md#0x1_starcoin_proof_verifier_SPARSE_MERKLE_LEAF_NODE">SPARSE_MERKLE_LEAF_NODE</a>, &leaf_node);
    <b>let</b> i = 0;
    <b>let</b> proof_length = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&proofs);
    <b>while</b> (i &lt; proof_length) {
        <b>let</b> sibling = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&proofs, i);
        <b>let</b> bit = <a href="starcoin_proof_bit.md#0x1_starcoin_proof_bit_get_bit">starcoin_proof_bit::get_bit</a>(&element_key, proof_length - i - 1);
        <b>let</b> internal_node = <b>if</b> (bit) {
            <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_Node">Node</a> { hash1: sibling, hash2: current_hash }
        } <b>else</b> {
            <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_Node">Node</a> { hash1: current_hash, hash2: sibling }
        };
        current_hash = <a href="starcoin_proof_structured_hash.md#0x1_starcoin_proof_structured_hash_hash">starcoin_proof_structured_hash::hash</a>(<a href="starcoin_proof.md#0x1_starcoin_proof_verifier_SPARSE_MERKLE_INTERNAL_NODE">SPARSE_MERKLE_INTERNAL_NODE</a>, &internal_node);
        i = i + 1;
    };
    current_hash
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_verifier_splite_symbol"></a>

## Function `splite_symbol`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_splite_symbol">splite_symbol</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_splite_symbol">splite_symbol</a>(): u8 {
    <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_DELMITER">DELMITER</a>
}
</code></pre>



</details>

<a id="0x1_starcoin_proof_verifier_split"></a>

## Function `split`



<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_split">split</a>(input: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_split">split</a>(input: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt; {
    <b>let</b> result: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt; = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
    <b>let</b> current_segment = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> i = 0;
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&input);

    <b>while</b> (i &lt; len) {
        <b>let</b> current_byte = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&input, i);
        <b>if</b> (current_byte == <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_DELMITER">DELMITER</a>) {
            <b>if</b> (!<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&current_segment)) {
                <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, current_segment);
                current_segment = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
            };
        } <b>else</b> {
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> current_segment, current_byte);
        };
        i = i + 1;
    };

    <b>if</b> (!<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&current_segment)) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, current_segment);
    };
    result
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
