
<a id="0x1_smt_utils"></a>

# Module `0x1::smt_utils`



-  [Constants](#@Constants_0)
-  [Function `get_bit_at_from_msb`](#0x1_smt_utils_get_bit_at_from_msb)
-  [Function `count_common_prefix`](#0x1_smt_utils_count_common_prefix)
-  [Function `count_vector_common_prefix`](#0x1_smt_utils_count_vector_common_prefix)
-  [Function `bits_to_bool_vector_from_msb`](#0x1_smt_utils_bits_to_bool_vector_from_msb)
-  [Function `concat_u8_vectors`](#0x1_smt_utils_concat_u8_vectors)
-  [Function `sub_u8_vector`](#0x1_smt_utils_sub_u8_vector)
-  [Function `sub_vector`](#0x1_smt_utils_sub_vector)
-  [Function `path_bits_to_bool_vector_from_msb`](#0x1_smt_utils_path_bits_to_bool_vector_from_msb)
-  [Function `split_side_nodes_data`](#0x1_smt_utils_split_side_nodes_data)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="smt_hash.md#0x1_smt_hash">0x1::smt_hash</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_smt_utils_BIT_LEFT"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_BIT_LEFT">BIT_LEFT</a>: bool = <b>false</b>;
</code></pre>



<a id="0x1_smt_utils_BIT_RIGHT"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_BIT_RIGHT">BIT_RIGHT</a>: bool = <b>true</b>;
</code></pre>



<a id="0x1_smt_utils_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH">ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH</a>: u64 = 103;
</code></pre>



<a id="0x1_smt_utils_ERROR_INVALID_PATH_BITS_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_PATH_BITS_LENGTH">ERROR_INVALID_PATH_BITS_LENGTH</a>: u64 = 102;
</code></pre>



<a id="0x1_smt_utils_ERROR_INVALID_PATH_BYTES_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_PATH_BYTES_LENGTH">ERROR_INVALID_PATH_BYTES_LENGTH</a>: u64 = 101;
</code></pre>



<a id="0x1_smt_utils_ERROR_VECTORS_NOT_SAME_LENGTH"></a>



<pre><code><b>const</b> <a href="smt_utils.md#0x1_smt_utils_ERROR_VECTORS_NOT_SAME_LENGTH">ERROR_VECTORS_NOT_SAME_LENGTH</a>: u64 = 103;
</code></pre>



<a id="0x1_smt_utils_get_bit_at_from_msb"></a>

## Function `get_bit_at_from_msb`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">get_bit_at_from_msb</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, position: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">get_bit_at_from_msb</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, position: u64): bool {
    <b>let</b> byte = (*<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>&lt;u8&gt;(data, position / 8) <b>as</b> u64);
    // <b>let</b> bit = BitOperators::rshift(byte, ((7 - (position % 8)) <b>as</b> u8));
    <b>let</b> bit = byte &gt;&gt; ((7 - (position % 8)) <b>as</b> u8);
    <b>if</b> (bit & 1 != 0) {
        <a href="smt_utils.md#0x1_smt_utils_BIT_RIGHT">BIT_RIGHT</a>
    } <b>else</b> {
        <a href="smt_utils.md#0x1_smt_utils_BIT_LEFT">BIT_LEFT</a>
    }
}
</code></pre>



</details>

<a id="0x1_smt_utils_count_common_prefix"></a>

## Function `count_common_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_count_common_prefix">count_common_prefix</a>(data1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data2: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_count_common_prefix">count_common_prefix</a>(data1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data2: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64 {
    <b>let</b> count = 0;
    <b>let</b> i = 0;
    <b>while</b> (i &lt; <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data1) * 8) {
        <b>if</b> (<a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">get_bit_at_from_msb</a>(data1, i) == <a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">get_bit_at_from_msb</a>(data2, i)) {
            count = count + 1;
        } <b>else</b> {
            <b>break</b>
        };
        i = i + 1;
    };
    count
}
</code></pre>



</details>

<a id="0x1_smt_utils_count_vector_common_prefix"></a>

## Function `count_vector_common_prefix`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_count_vector_common_prefix">count_vector_common_prefix</a>&lt;ElementT: <b>copy</b>, drop&gt;(vec1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;, vec2: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_count_vector_common_prefix">count_vector_common_prefix</a>&lt;ElementT: <b>copy</b> + drop&gt;(
    vec1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;,
    vec2: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;
): u64 {
    <b>let</b> vec_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;ElementT&gt;(vec1);
    <b>assert</b>!(vec_len == <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;ElementT&gt;(vec2), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_utils.md#0x1_smt_utils_ERROR_VECTORS_NOT_SAME_LENGTH">ERROR_VECTORS_NOT_SAME_LENGTH</a>));
    <b>let</b> idx = 0;
    <b>while</b> (idx &lt; vec_len) {
        <b>if</b> (*<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(vec1, idx) != *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(vec2, idx)) {
            <b>break</b>
        };
        idx = idx + 1;
    };
    idx
}
</code></pre>



</details>

<a id="0x1_smt_utils_bits_to_bool_vector_from_msb"></a>

## Function `bits_to_bool_vector_from_msb`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_bits_to_bool_vector_from_msb">bits_to_bool_vector_from_msb</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_bits_to_bool_vector_from_msb">bits_to_bool_vector_from_msb</a>(data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt; {
    <b>let</b> i = 0;
    <b>let</b> vec = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;bool&gt;();
    <b>while</b> (i &lt; <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(data) * 8) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>&lt;bool&gt;(&<b>mut</b> vec, <a href="smt_utils.md#0x1_smt_utils_get_bit_at_from_msb">get_bit_at_from_msb</a>(data, i));
        i = i + 1;
    };
    vec
}
</code></pre>



</details>

<a id="0x1_smt_utils_concat_u8_vectors"></a>

## Function `concat_u8_vectors`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">concat_u8_vectors</a>(v1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, v2: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_concat_u8_vectors">concat_u8_vectors</a>(v1: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, v2: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> data = *v1;
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> data, v2);
    data
}
</code></pre>



</details>

<a id="0x1_smt_utils_sub_u8_vector"></a>

## Function `sub_u8_vector`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">sub_u8_vector</a>(vec: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, start: u64, end: u64): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">sub_u8_vector</a>(vec: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, start: u64, end: u64): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> i = start;
    <b>let</b> result = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();
    <b>let</b> data_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(vec);
    <b>let</b> actual_end = <b>if</b> (end &lt; data_len) {
        end
    } <b>else</b> {
        data_len
    };
    <b>while</b> (i &lt; actual_end) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(vec, i));
        i = i + 1;
    };
    result
}
</code></pre>



</details>

<a id="0x1_smt_utils_sub_vector"></a>

## Function `sub_vector`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_sub_vector">sub_vector</a>&lt;ElementT: <b>copy</b>&gt;(vec: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;, start: u64, end: u64): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_sub_vector">sub_vector</a>&lt;ElementT: <b>copy</b>&gt;(vec: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt;, start: u64, end: u64): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;ElementT&gt; {
    <b>let</b> i = start;
    <b>let</b> result = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;ElementT&gt;();
    <b>let</b> data_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(vec);
    <b>let</b> actual_end = <b>if</b> (end &lt; data_len) {
        end
    } <b>else</b> {
        data_len
    };
    <b>while</b> (i &lt; actual_end) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(vec, i));
        i = i + 1;
    };
    result
}
</code></pre>



</details>

<a id="0x1_smt_utils_path_bits_to_bool_vector_from_msb"></a>

## Function `path_bits_to_bool_vector_from_msb`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_path_bits_to_bool_vector_from_msb">path_bits_to_bool_vector_from_msb</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_path_bits_to_bool_vector_from_msb">path_bits_to_bool_vector_from_msb</a>(path: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;bool&gt; {
    <b>let</b> path_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;u8&gt;(path);
    <b>assert</b>!(path_len == <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>(), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_PATH_BYTES_LENGTH">ERROR_INVALID_PATH_BYTES_LENGTH</a>));
    <b>let</b> result_vec = <a href="smt_utils.md#0x1_smt_utils_bits_to_bool_vector_from_msb">bits_to_bool_vector_from_msb</a>(path);
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>&lt;bool&gt;(&result_vec) == <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>() * 8,// <a href="smt_tree_hasher.md#0x1_smt_tree_hasher_path_size_in_bits">smt_tree_hasher::path_size_in_bits</a>(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_PATH_BITS_LENGTH">ERROR_INVALID_PATH_BITS_LENGTH</a>)
    );
    result_vec
}
</code></pre>



</details>

<a id="0x1_smt_utils_split_side_nodes_data"></a>

## Function `split_side_nodes_data`



<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_split_side_nodes_data">split_side_nodes_data</a>(side_nodes_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="smt_utils.md#0x1_smt_utils_split_side_nodes_data">split_side_nodes_data</a>(side_nodes_data: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt; {
    <b>let</b> node_data_length = <a href="smt_hash.md#0x1_smt_hash_size">smt_hash::size</a>();
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(side_nodes_data);
    <b>assert</b>!(len % node_data_length == 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="smt_utils.md#0x1_smt_utils_ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH">ERROR_INVALID_NODES_DATA_PACKAGE_LENGTH</a>));

    <b>if</b> (len &gt; 0) {
        <b>let</b> result = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;();
        <b>let</b> size = len / node_data_length;
        <b>let</b> idx = 0;
        <b>while</b> (idx &lt; size) {
            <b>let</b> start = idx * node_data_length;
            <b>let</b> end = start + node_data_length;
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> result, <a href="smt_utils.md#0x1_smt_utils_sub_u8_vector">sub_u8_vector</a>(side_nodes_data, start, end));
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
