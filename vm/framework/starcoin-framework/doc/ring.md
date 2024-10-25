
<a id="0x1_ring"></a>

# Module `0x1::ring`

A ring-shaped container that can hold any type, indexed from 0
The capacity is fixed at creation time, and the accessible index is constantly growing


-  [Struct `Ring`](#0x1_ring_Ring)
-  [Constants](#@Constants_0)
-  [Function `create_with_capacity`](#0x1_ring_create_with_capacity)
-  [Function `is_full`](#0x1_ring_is_full)
-  [Function `capacity`](#0x1_ring_capacity)
-  [Function `push`](#0x1_ring_push)
-  [Function `borrow`](#0x1_ring_borrow)
-  [Function `borrow_mut`](#0x1_ring_borrow_mut)
-  [Function `index_of`](#0x1_ring_index_of)
-  [Function `destroy`](#0x1_ring_destroy)
-  [Specification](#@Specification_1)
    -  [Function `create_with_capacity`](#@Specification_1_create_with_capacity)
    -  [Function `is_full`](#@Specification_1_is_full)
    -  [Function `capacity`](#@Specification_1_capacity)
    -  [Function `push`](#@Specification_1_push)
    -  [Function `borrow`](#@Specification_1_borrow)
    -  [Function `borrow_mut`](#@Specification_1_borrow_mut)
    -  [Function `index_of`](#@Specification_1_index_of)
    -  [Function `destroy`](#@Specification_1_destroy)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
</code></pre>



<a id="0x1_ring_Ring"></a>

## Struct `Ring`



<pre><code><b>struct</b> <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt; <b>has</b> store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>data: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>insertion_index: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>external_index: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS"></a>

The index into the vector is out of bounds


<pre><code><b>const</b> <a href="ring.md#0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS">ERROR_RING_INDEX_OUT_OF_BOUNDS</a>: u64 = 101;
</code></pre>



<a id="0x1_ring_create_with_capacity"></a>

## Function `create_with_capacity`

Create a Ring with capacity.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_create_with_capacity">create_with_capacity</a>&lt;Element&gt;(len: u64): <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_create_with_capacity">create_with_capacity</a>&lt;Element&gt;(len: u64): <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt; {
    <b>let</b> data = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;&gt;();
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> data, <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;Element&gt;());
        i = i + 1;
    };
    <a href="ring.md#0x1_ring_Ring">Ring</a> {
        data: data,
        insertion_index: 0,
        external_index: 0,
    }
}
</code></pre>



</details>

<a id="0x1_ring_is_full"></a>

## Function `is_full`

is Ring full


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_is_full">is_full</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_is_full">is_full</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;): bool {
    <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&r.data, r.insertion_index))
}
</code></pre>



</details>

<a id="0x1_ring_capacity"></a>

## Function `capacity`

Return the capacity of the Ring.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;): u64 {
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&r.data)
}
</code></pre>



</details>

<a id="0x1_ring_push"></a>

## Function `push`

Add element <code>e</code> to the insertion_index of the Ring <code>r</code>.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_push">push</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, e: Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_push">push</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;, e: Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt; {
    <b>let</b> op_e = <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>&lt;<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;&gt;(&<b>mut</b> r.data, r.insertion_index);
    <b>let</b> res = <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>&lt;Element&gt;(op_e)) {
        <a href="../../move-stdlib/doc/option.md#0x1_option_fill">option::fill</a>(op_e, e);
        <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;Element&gt;()
    }<b>else</b> {
        <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>&lt;Element&gt;(<a href="../../move-stdlib/doc/option.md#0x1_option_swap">option::swap</a>(op_e, e))
    };
    r.insertion_index = (r.insertion_index + 1) % <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&r.data);
    r.external_index = r.external_index + 1;
    res
}
</code></pre>



</details>

<a id="0x1_ring_borrow"></a>

## Function `borrow`

Return a reference to the <code>i</code>th element in the Ring <code>r</code>.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow">borrow</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, i: u64): &<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow">borrow</a>&lt;Element&gt;(r: & <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;, i: u64): &<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt; {
    <b>let</b> len = <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r);
    <b>if</b> (r.external_index &gt; len - 1) {
        <b>assert</b>!(
            i &gt;= r.external_index - len && i &lt; r.external_index,
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="ring.md#0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS">ERROR_RING_INDEX_OUT_OF_BOUNDS</a>)
        );
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&r.data, i % len)
    }<b>else</b> {
        <b>assert</b>!(i &lt; len, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="ring.md#0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS">ERROR_RING_INDEX_OUT_OF_BOUNDS</a>));
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&r.data, i)
    }
}
</code></pre>



</details>

<a id="0x1_ring_borrow_mut"></a>

## Function `borrow_mut`

Return a mutable reference to the <code>i</code>th element in the Ring <code>r</code>.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow_mut">borrow_mut</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, i: u64): &<b>mut</b> <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow_mut">borrow_mut</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;, i: u64): &<b>mut</b> <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt; {
    <b>let</b> len = <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r);
    <b>if</b> (r.external_index &gt; len - 1) {
        <b>assert</b>!(
            i &gt;= r.external_index - len && i &lt; r.external_index,
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="ring.md#0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS">ERROR_RING_INDEX_OUT_OF_BOUNDS</a>)
        );
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> r.data, i % len)
    }<b>else</b> {
        <b>assert</b>!(i &lt; len, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="ring.md#0x1_ring_ERROR_RING_INDEX_OUT_OF_BOUNDS">ERROR_RING_INDEX_OUT_OF_BOUNDS</a>));
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow_mut">vector::borrow_mut</a>(&<b>mut</b> r.data, i)
    }
}
</code></pre>



</details>

<a id="0x1_ring_index_of"></a>

## Function `index_of`

Return <code><a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;</code> if <code>e</code> is in the Ring <code>r</code> at index <code>i</code>.
Otherwise, returns <code><a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;u64&gt;</code>.


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_index_of">index_of</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, e: &Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_index_of">index_of</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;, e: &Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt; {
    <b>let</b> i = 0;
    <b>let</b> len = <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r);
    <b>while</b> (i &lt; len) {
        <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&r.data, i)) == e) <b>return</b> <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(i + r.external_index - len);
        i = i + 1;
    };
    <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;u64&gt;()
}
</code></pre>



</details>

<a id="0x1_ring_destroy"></a>

## Function `destroy`

Destroy the Ring <code>r</code>.
Returns the vector<Element> saved by ring


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_destroy">destroy</a>&lt;Element&gt;(r: <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;Element&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_destroy">destroy</a>&lt;Element&gt;(r: <a href="ring.md#0x1_ring_Ring">Ring</a>&lt;Element&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;Element&gt; {
    <b>let</b> <a href="ring.md#0x1_ring_Ring">Ring</a> {
        data: data,
        insertion_index: _,
        external_index: _,
    } = r ;
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&data);
    <b>let</b> i = 0;
    <b>let</b> vec = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;Element&gt;();
    <b>while</b> (i &lt; len) {
        <b>let</b> op_e = <a href="../../move-stdlib/doc/vector.md#0x1_vector_pop_back">vector::pop_back</a>(&<b>mut</b> data);
        <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&op_e)) {
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> vec, <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(op_e))
        }<b>else</b> {
            <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_none">option::destroy_none</a>(op_e)
        };
        i = i + 1;
    };
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_destroy_empty">vector::destroy_empty</a>(data);
    vec
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification


<a id="@Specification_1_create_with_capacity"></a>

### Function `create_with_capacity`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_create_with_capacity">create_with_capacity</a>&lt;Element&gt;(len: u64): <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



<a id="@Specification_1_is_full"></a>

### Function `is_full`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_is_full">is_full</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): bool
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



<a id="@Specification_1_capacity"></a>

### Function `capacity`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_capacity">capacity</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): u64
</code></pre>





<a id="@Specification_1_push"></a>

### Function `push`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_push">push</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, e: Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



<a id="@Specification_1_borrow"></a>

### Function `borrow`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow">borrow</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, i: u64): &<a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



<a id="@Specification_1_borrow_mut"></a>

### Function `borrow_mut`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_borrow_mut">borrow_mut</a>&lt;Element&gt;(r: &<b>mut</b> <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, i: u64): &<b>mut</b> <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;Element&gt;
</code></pre>




<a id="@Specification_1_index_of"></a>

### Function `index_of`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_index_of">index_of</a>&lt;Element&gt;(r: &<a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;, e: &Element): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>



<a id="@Specification_1_destroy"></a>

### Function `destroy`


<pre><code><b>public</b> <b>fun</b> <a href="ring.md#0x1_ring_destroy">destroy</a>&lt;Element&gt;(r: <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;Element&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;Element&gt;
</code></pre>




<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
