
<a id="0x1_smt_proofs"></a>

# Module `0x1::smt_proofs`



-  [Constants](#@Constants_0)
-  [Function `verify_non_membership_proof_by_key`](#0x1_smt_proofs_verify_non_membership_proof_by_key)
-  [Function `verify_non_membership_proof_by_leaf_path`](#0x1_smt_proofs_verify_non_membership_proof_by_leaf_path)
-  [Function `verify_membership_proof_by_key_value`](#0x1_smt_proofs_verify_membership_proof_by_key_value)
-  [Function `verify_membership_proof`](#0x1_smt_proofs_verify_membership_proof)
-  [Function `compute_root_hash_by_leaf`](#0x1_smt_proofs_compute_root_hash_by_leaf)
-  [Function `compute_root_hash_new_leaf_included`](#0x1_smt_proofs_compute_root_hash_new_leaf_included)
-  [Function `create_membership_proof`](#0x1_smt_proofs_create_membership_proof)
-  [Function `create_membership_side_nodes`](#0x1_smt_proofs_create_membership_side_nodes)
-  [Function `compute_root_hash`](#0x1_smt_proofs_compute_root_hash)


<pre><code><b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher">0x1::smt_tree_hasher</a>;
<b>use</b> <a href="smt_utils.md#0x1_smt_utils">0x1::smt_utils</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_smt_proofs_BIT_RIGHT"></a>



<pre><code><b>const</b> <a href="smt_proofs.md#0x1_smt_proofs_BIT_RIGHT">BIT_RIGHT</a>: bool = <b>true</b>;
</code></pre>



<a id="0x1_smt_proofs_ERROR_COUNT_COMMON_PREFIX"></a>



<pre><code><b>const</b> <a href="smt_proofs.md#0x1_smt_proofs_ERROR_COUNT_COMMON_PREFIX">ERROR_COUNT_COMMON_PREFIX</a>: u64 = 102;
</code></pre>



<a id="0x1_smt_proofs_ERROR_KEY_ALREADY_EXISTS_IN_PROOF"></a>



<pre><code><b>const</b> <a href="smt_proofs.md#0x1_smt_proofs_ERROR_KEY_ALREADY_EXISTS_IN_PROOF">ERROR_KEY_ALREADY_EXISTS_IN_PROOF</a>: u64 = 101;
</code></pre>



<a id="0x1_smt_proofs_verify_non_membership_proof_by_key"></a>

## Function `verify_non_membership_proof_by_key`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_non_membership_proof_by_key">verify_non_membership_proof_by_key</a>(root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_non_membership_proof_by_key">verify_non_membership_proof_by_key</a>(
    root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;,
    key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool {
    <b>let</b> leaf_path = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">smt_tree_hasher::digest</a>(key);
    <a href="smt_proofs.md#0x1_smt_proofs_verify_non_membership_proof_by_leaf_path">verify_non_membership_proof_by_leaf_path</a>(root_hash, non_membership_leaf_data, side_nodes, &leaf_path)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_verify_non_membership_proof_by_leaf_path"></a>

## Function `verify_non_membership_proof_by_leaf_path`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_non_membership_proof_by_leaf_path">verify_non_membership_proof_by_leaf_path</a>(root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_non_membership_proof_by_leaf_path">verify_non_membership_proof_by_leaf_path</a>(
    root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;,
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool {
    <b>let</b> non_membership_leaf_hash = <b>if</b> (<a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;u8&gt;(non_membership_leaf_data) &gt; 0) {
        <b>let</b> (non_membership_leaf_path, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_leaf">smt_tree_hasher::parse_leaf</a>(non_membership_leaf_data);
        <b>assert</b>!(*leaf_path != *&non_membership_leaf_path, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_proofs.md#0x1_smt_proofs_ERROR_KEY_ALREADY_EXISTS_IN_PROOF">ERROR_KEY_ALREADY_EXISTS_IN_PROOF</a>));
        <b>assert</b>!(
            (<a href="smt_utils.md#0x1_smt_utils_count_common_prefix">smt_utils::count_common_prefix</a>(leaf_path, &non_membership_leaf_path) &gt;= <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(side_nodes)),
            <a href="smt_proofs.md#0x1_smt_proofs_ERROR_COUNT_COMMON_PREFIX">ERROR_COUNT_COMMON_PREFIX</a>
        );
        <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf_data">smt_tree_hasher::digest_leaf_data</a>(non_membership_leaf_data)
    } <b>else</b> {
        <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_placeholder">smt_tree_hasher::placeholder</a>()
    };
    <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(leaf_path, &non_membership_leaf_hash, side_nodes) == *root_hash
}
</code></pre>



</details>

<a id="0x1_smt_proofs_verify_membership_proof_by_key_value"></a>

## Function `verify_membership_proof_by_key_value`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_membership_proof_by_key_value">verify_membership_proof_by_key_value</a>(root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, is_raw_value: bool): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_membership_proof_by_key_value">verify_membership_proof_by_key_value</a>(
    root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;,
    key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    is_raw_value: bool
): bool {
    <b>let</b> leaf_path = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">smt_tree_hasher::digest</a>(key);
    <b>let</b> leaf_value_hash = <b>if</b> (is_raw_value) {
        &<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">smt_tree_hasher::digest</a>(value)
    } <b>else</b> {
        value
    };
    <a href="smt_proofs.md#0x1_smt_proofs_verify_membership_proof">verify_membership_proof</a>(root_hash, side_nodes, &leaf_path, leaf_value_hash)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_verify_membership_proof"></a>

## Function `verify_membership_proof`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_membership_proof">verify_membership_proof</a>(root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_verify_membership_proof">verify_membership_proof</a>(
    root_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;,
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
): bool {
    <b>let</b> (leaf_hash, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf">smt_tree_hasher::digest_leaf</a>(leaf_path, leaf_value_hash);
    <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(leaf_path, &leaf_hash, side_nodes) == *root_hash
}
</code></pre>



</details>

<a id="0x1_smt_proofs_compute_root_hash_by_leaf"></a>

## Function `compute_root_hash_by_leaf`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash_by_leaf">compute_root_hash_by_leaf</a>(leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash_by_leaf">compute_root_hash_by_leaf</a>(
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> (leaf_hash, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf">smt_tree_hasher::digest_leaf</a>(leaf_path, leaf_value_hash);
    <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(leaf_path, &leaf_hash, side_nodes)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_compute_root_hash_new_leaf_included"></a>

## Function `compute_root_hash_new_leaf_included`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash_new_leaf_included">compute_root_hash_new_leaf_included</a>(leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash_new_leaf_included">compute_root_hash_new_leaf_included</a>(
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> (new_side_nodes, leaf_node_hash) = <a href="smt_proofs.md#0x1_smt_proofs_create_membership_side_nodes">create_membership_side_nodes</a>(
        leaf_path,
        leaf_value_hash,
        non_membership_leaf_data,
        side_nodes
    );

    <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(leaf_path, &leaf_node_hash, &new_side_nodes)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_create_membership_proof"></a>

## Function `create_membership_proof`



<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_create_membership_proof">create_membership_proof</a>(leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_create_membership_proof">create_membership_proof</a>(
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;) {
    <b>let</b> (new_side_nodes, leaf_node_hash) = <a href="smt_proofs.md#0x1_smt_proofs_create_membership_side_nodes">create_membership_side_nodes</a>(
        leaf_path,
        leaf_value_hash,
        non_membership_leaf_data,
        side_nodes
    );
    <b>let</b> new_root_hash = <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(leaf_path, &leaf_node_hash, &new_side_nodes);
    (new_root_hash, new_side_nodes)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_create_membership_side_nodes"></a>

## Function `create_membership_side_nodes`



<pre><code><b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_create_membership_side_nodes">create_membership_side_nodes</a>(leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_create_membership_side_nodes">create_membership_side_nodes</a>(
    leaf_path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    leaf_value_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    non_membership_leaf_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> side_nodes_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;(side_nodes);
    <b>let</b> (new_leaf_hash, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf">smt_tree_hasher::digest_leaf</a>(leaf_path, leaf_value_hash);
    <b>let</b> new_side_nodes = <b>if</b> (<a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(non_membership_leaf_data) &gt; 0) {
        <b>let</b> (non_membership_leaf_path, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_leaf">smt_tree_hasher::parse_leaf</a>(non_membership_leaf_data);
        <b>assert</b>!(*leaf_path != *&non_membership_leaf_path, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_proofs.md#0x1_smt_proofs_ERROR_KEY_ALREADY_EXISTS_IN_PROOF">ERROR_KEY_ALREADY_EXISTS_IN_PROOF</a>));

        <b>let</b> common_prefix_count = <a href="smt_utils.md#0x1_smt_utils_count_common_prefix">smt_utils::count_common_prefix</a>(leaf_path, &non_membership_leaf_path);
        <b>let</b> old_leaf_hash = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf_data">smt_tree_hasher::digest_leaf_data</a>(non_membership_leaf_data);
        <b>let</b> new_side_nodes = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;();

        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> new_side_nodes, old_leaf_hash);
        <b>if</b> (common_prefix_count &gt; side_nodes_len) {
            <b>let</b> place_holder_len = (common_prefix_count - side_nodes_len);
            // Put placeholders
            <b>let</b> idx = 0;
            <b>while</b> (idx &lt; place_holder_len) {
                <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> new_side_nodes, <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_placeholder">smt_tree_hasher::placeholder</a>());
                idx = idx + 1;
            };
        };
        new_side_nodes
    } <b>else</b> {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;()
    };

    // Push <b>old</b> siblings into the new siblings array
    <b>let</b> idx = 0;
    <b>while</b> (idx &lt; side_nodes_len) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> new_side_nodes, *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(side_nodes, idx));
        idx = idx + 1;
    };
    (new_side_nodes, new_leaf_hash)
}
</code></pre>



</details>

<a id="0x1_smt_proofs_compute_root_hash"></a>

## Function `compute_root_hash`



<pre><code><b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, node_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="smt_proofs.md#0x1_smt_proofs_compute_root_hash">compute_root_hash</a>(
    path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    node_hash: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    side_nodes: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(side_nodes);
    <b>let</b> side_nodes_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;(side_nodes);

    <b>let</b> i = 0;
    <b>let</b> current_hash = *node_hash;
    <b>while</b> (i &lt; side_nodes_len) {
        <b>let</b> bit = <a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">smt_utils::get_bit_at_from_msb</a>(path, side_nodes_len - i - 1);
        <b>let</b> sibling_hash = <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;(side_nodes, i);
        <b>if</b> (bit == <a href="smt_proofs.md#0x1_smt_proofs_BIT_RIGHT">BIT_RIGHT</a>) {
            (current_hash, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_node">smt_tree_hasher::digest_node</a>(sibling_hash, &current_hash);
        } <b>else</b> {
            // left
            (current_hash, _) = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_node">smt_tree_hasher::digest_node</a>(&current_hash, sibling_hash);
        };
        i = i + 1;
    };
    current_hash
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
