
<a id="0x1_smt_tree_hasher"></a>

# Module `0x1::smt_tree_hasher`



-  [Constants](#@Constants_0)
-  [Function `parse_leaf`](#0x1_smt_tree_hasher_parse_leaf)
-  [Function `parse_node`](#0x1_smt_tree_hasher_parse_node)
-  [Function `digest_leaf`](#0x1_smt_tree_hasher_digest_leaf)
-  [Function `create_leaf_data`](#0x1_smt_tree_hasher_create_leaf_data)
-  [Function `digest_leaf_data`](#0x1_smt_tree_hasher_digest_leaf_data)
-  [Function `digest_node`](#0x1_smt_tree_hasher_digest_node)
-  [Function `path`](#0x1_smt_tree_hasher_path)
-  [Function `digest`](#0x1_smt_tree_hasher_digest)
-  [Function `path_size`](#0x1_smt_tree_hasher_path_size)
-  [Function `path_size_in_bits`](#0x1_smt_tree_hasher_path_size_in_bits)
-  [Function `placeholder`](#0x1_smt_tree_hasher_placeholder)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="smt_hash.md#0x1_smt_hash">0x1::smt_hash</a>;
<b>use</b> <a href="smt_utils.md#0x1_smt_utils">0x1::smt_utils</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA">ERROR_INVALID_LEAF_DATA</a>: u64 = 102;
</code></pre>



<a id="0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA_LENGTH">ERROR_INVALID_LEAF_DATA_LENGTH</a>: u64 = 104;
</code></pre>



<a id="0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA">ERROR_INVALID_NODE_DATA</a>: u64 = 103;
</code></pre>



<a id="0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA_LENGTH">ERROR_INVALID_NODE_DATA_LENGTH</a>: u64 = 105;
</code></pre>



<a id="0x1_smt_tree_hasher_LEAF_PREFIX"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [0];
</code></pre>



<a id="0x1_smt_tree_hasher_NODE_PREFIX"></a>



<pre><code><b>const</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_NODE_PREFIX">NODE_PREFIX</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [1];
</code></pre>



<a id="0x1_smt_tree_hasher_parse_leaf"></a>

## Function `parse_leaf`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_leaf">parse_leaf</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_leaf">parse_leaf</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> data_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data);

    <b>let</b> prefix_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>);
    <b>assert</b>!(data_len &gt;= prefix_len + <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>(), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA">ERROR_INVALID_LEAF_DATA</a>));
    <b>assert</b>!(<a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, 0, prefix_len) == <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA">ERROR_INVALID_LEAF_DATA</a>));

    <b>let</b> start = 0;
    <b>let</b> end = prefix_len;
    _ = start;//<b>let</b> prefix = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);

    start = end;
    end = start + <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>();
    <b>let</b> leaf_node_path = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);

    start = end;
    end = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data);
    <b>let</b> leaf_node_value = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);
    (leaf_node_path, leaf_node_value)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_parse_node"></a>

## Function `parse_node`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_node">parse_node</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_parse_node">parse_node</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> data_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data);
    <b>let</b> prefix_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_NODE_PREFIX">NODE_PREFIX</a>);
    <b>assert</b>!(data_len == prefix_len + <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>() * 2, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA">ERROR_INVALID_NODE_DATA</a>));
    <b>assert</b>!(<a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, 0, prefix_len) == <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_NODE_PREFIX">NODE_PREFIX</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA">ERROR_INVALID_NODE_DATA</a>));

    <b>let</b> start = 0;
    <b>let</b> end = prefix_len;
    _ = start;//<b>let</b> prefix = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);

    start = end;
    end = start + <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>();
    <b>let</b> left_data = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);

    start = end;
    end = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data);
    <b>let</b> right_data = <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, start, end);
    (left_data, right_data)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_digest_leaf"></a>

## Function `digest_leaf`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf">digest_leaf</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf">digest_leaf</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> value = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>;
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *path);
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *leaf_value);
    (<a href="smt_hash.md#0x1_smt_hash_hash">smt_hash::hash</a>(&value), value)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_create_leaf_data"></a>

## Function `create_leaf_data`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_create_leaf_data">create_leaf_data</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_create_leaf_data">create_leaf_data</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, leaf_value: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> value = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>;
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *path);
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *leaf_value);
    value
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_digest_leaf_data"></a>

## Function `digest_leaf_data`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf_data">digest_leaf_data</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_leaf_data">digest_leaf_data</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> data_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data);
    <b>let</b> prefix_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>);
    <b>assert</b>!(data_len &gt;= prefix_len + <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>(), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA_LENGTH">ERROR_INVALID_LEAF_DATA_LENGTH</a>));
    <b>assert</b>!(<a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(data, 0, prefix_len) == <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_LEAF_PREFIX">LEAF_PREFIX</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_LEAF_DATA">ERROR_INVALID_LEAF_DATA</a>));
    <a href="smt_hash.md#0x1_smt_hash_hash">smt_hash::hash</a>(data)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_digest_node"></a>

## Function `digest_node`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_node">digest_node</a>(left_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, right_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest_node">digest_node</a>(left_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, right_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): (<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> node_left_right_data_length = <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>();
    <b>assert</b>!(<a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(left_data) == node_left_right_data_length, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA_LENGTH">ERROR_INVALID_NODE_DATA_LENGTH</a>));
    <b>assert</b>!(<a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(right_data) == node_left_right_data_length, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_tree_hasher.md#0x1_smt_tree_hasher_ERROR_INVALID_NODE_DATA_LENGTH">ERROR_INVALID_NODE_DATA_LENGTH</a>));

    <b>let</b> value = <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_NODE_PREFIX">NODE_PREFIX</a>;
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *left_data);
    value = <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">smt_utils::concat_u8_vectors</a>(&value, *right_data);
    (<a href="smt_hash.md#0x1_smt_hash_hash">smt_hash::hash</a>(&value), value)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_path"></a>

## Function `path`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path">path</a>(key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path">path</a>(key: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">digest</a>(key)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_digest"></a>

## Function `digest`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">digest</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_digest">digest</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="smt_hash.md#0x1_smt_hash_hash">smt_hash::hash</a>(data)
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_path_size"></a>

## Function `path_size`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size">path_size</a>(): u64 {
    <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>()
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_path_size_in_bits"></a>

## Function `path_size_in_bits`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size_in_bits">path_size_in_bits</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size_in_bits">path_size_in_bits</a>(): u64 {
    <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>() * 8
}
</code></pre>



</details>

<a id="0x1_smt_tree_hasher_placeholder"></a>

## Function `placeholder`



<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_placeholder">placeholder</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_placeholder">placeholder</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <a href="smt_hash.md#0x1_smt_hash_size_zero_bytes">smt_hash::size_zero_bytes</a>()
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
