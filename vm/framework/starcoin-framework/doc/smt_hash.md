
<a id="0x1_smt_hash"></a>

# Module `0x1::smt_hash`



-  [Constants](#@Constants_0)
-  [Function `size`](#0x1_smt_hash_size)
-  [Function `hash`](#0x1_smt_hash_hash)
-  [Function `size_zero_bytes`](#0x1_smt_hash_size_zero_bytes)
-  [Function `path_bits_to_bool_vector_from_msb`](#0x1_smt_hash_path_bits_to_bool_vector_from_msb)
-  [Function `split_side_nodes_data`](#0x1_smt_hash_split_side_nodes_data)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="smt_utils.md#0x1_smt_utils">0x1::smt_utils</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_smt_hash_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH">ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH</a>: u64 = 103;
</code></pre>



<a id="0x1_smt_hash_ERROR_INVALID_PATH_BITS_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_PATH_BITS_LENGTH">ERROR_INVALID_PATH_BITS_LENGTH</a>: u64 = 102;
</code></pre>



<a id="0x1_smt_hash_ERROR_INVALID_PATH_BYTES_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_PATH_BYTES_LENGTH">ERROR_INVALID_PATH_BYTES_LENGTH</a>: u64 = 101;
</code></pre>



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

<a id="0x1_smt_hash_path_bits_to_bool_vector_from_msb"></a>

## Function `path_bits_to_bool_vector_from_msb`



<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_path_bits_to_bool_vector_from_msb">path_bits_to_bool_vector_from_msb</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_path_bits_to_bool_vector_from_msb">path_bits_to_bool_vector_from_msb</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt; {
    <b>let</b> path_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;u8&gt;(path);
    <b>assert</b>!(path_len == <a href="smt_hash.md#0x1_smt_hash_size">Self::size</a>(), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_PATH_BYTES_LENGTH">ERROR_INVALID_PATH_BYTES_LENGTH</a>));
    <b>let</b> result_vec = <a href="smt_utils.md#0x1_smt_utils_bits_to_bool_vector_from_msb">smt_utils::bits_to_bool_vector_from_msb</a>(path);
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;bool&gt;(&result_vec) == <a href="smt_hash.md#0x1_smt_hash_size">Self::size</a>() * 8,// <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size_in_bits">smt_tree_hasher::path_size_in_bits</a>(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_PATH_BITS_LENGTH">ERROR_INVALID_PATH_BITS_LENGTH</a>)
    );
    result_vec
}
</code></pre>



</details>

<a id="0x1_smt_hash_split_side_nodes_data"></a>

## Function `split_side_nodes_data`



<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_split_side_nodes_data">split_side_nodes_data</a>(side_nodes_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_hash.md#0x1_smt_hash_split_side_nodes_data">split_side_nodes_data</a>(side_nodes_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt; {
    <b>let</b> node_data_length = <a href="smt_hash.md#0x1_smt_hash_size">Self::size</a>();
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(side_nodes_data);
    <b>assert</b>!(len % node_data_length == 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_hash.md#0x1_smt_hash_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH">ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH</a>));

    <b>if</b> (len &gt; 0) {
        <b>let</b> result = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;();
        <b>let</b> size = len / node_data_length;
        <b>let</b> idx = 0;
        <b>while</b> (idx &lt; size) {
            <b>let</b> start = idx * node_data_length;
            <b>let</b> end = start + node_data_length;
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">smt_utils::sub_u8_vector</a>(side_nodes_data, start, end));
            idx = idx + 1;
        };
        result
    } <b>else</b> {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;()
    }
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
