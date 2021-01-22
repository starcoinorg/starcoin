
<a name="0x1_Compare"></a>

# Module `0x1::Compare`



-  [Constants](#@Constants_0)
-  [Function `cmp_bcs_bytes`](#0x1_Compare_cmp_bcs_bytes)
-  [Function `cmp_u8`](#0x1_Compare_cmp_u8)
-  [Function `cmp_u64`](#0x1_Compare_cmp_u64)
-  [Specification](#@Specification_1)
    -  [Function `cmp_bcs_bytes`](#@Specification_1_cmp_bcs_bytes)
    -  [Function `cmp_u8`](#@Specification_1_cmp_u8)
    -  [Function `cmp_u64`](#@Specification_1_cmp_u64)


<pre><code><b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_Compare_EQUAL"></a>



<pre><code><b>const</b> <a href="Compare.md#0x1_Compare_EQUAL">EQUAL</a>: u8 = 0;
</code></pre>



<a name="0x1_Compare_GREATER_THAN"></a>



<pre><code><b>const</b> <a href="Compare.md#0x1_Compare_GREATER_THAN">GREATER_THAN</a>: u8 = 2;
</code></pre>



<a name="0x1_Compare_LESS_THAN"></a>



<pre><code><b>const</b> <a href="Compare.md#0x1_Compare_LESS_THAN">LESS_THAN</a>: u8 = 1;
</code></pre>



<a name="0x1_Compare_cmp_bcs_bytes"></a>

## Function `cmp_bcs_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="Compare.md#0x1_Compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Compare.md#0x1_Compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8 {
    <b>let</b> i1 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v1);
    <b>let</b> i2 = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(v2);
    <b>let</b> len_cmp = <a href="Compare.md#0x1_Compare_cmp_u64">cmp_u64</a>(i1, i2);

    // <a href="BCS.md#0x1_BCS">BCS</a> uses little endian encoding for all integer types, so we choose <b>to</b> compare from left
    // <b>to</b> right. Going right <b>to</b> left would make the behavior of <a href="Compare.md#0x1_Compare">Compare</a>.cmp diverge from the
    // bytecode operators &lt; and &gt; on integer values (which would be confusing).
    <b>while</b> (i1 &gt; 0 && i2 &gt; 0) {
        i1 = i1 - 1;
        i2 = i2 - 1;
        <b>let</b> elem_cmp = <a href="Compare.md#0x1_Compare_cmp_u8">cmp_u8</a>(*<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v1, i1), *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(v2, i2));
        <b>if</b> (elem_cmp != 0) <b>return</b> elem_cmp
        // <b>else</b>, compare next element
    };
    // all compared elements equal; <b>use</b> length comparison <b>to</b> <b>break</b> the tie
    len_cmp
}
</code></pre>



</details>

<a name="0x1_Compare_cmp_u8"></a>

## Function `cmp_u8`



<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8 {
    <b>if</b> (i1 == i2) <a href="Compare.md#0x1_Compare_EQUAL">EQUAL</a>
    <b>else</b> <b>if</b> (i1 &lt; i2) <a href="Compare.md#0x1_Compare_LESS_THAN">LESS_THAN</a>
    <b>else</b> <a href="Compare.md#0x1_Compare_GREATER_THAN">GREATER_THAN</a>
}
</code></pre>



</details>

<a name="0x1_Compare_cmp_u64"></a>

## Function `cmp_u64`



<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8 {
    <b>if</b> (i1 == i2) <a href="Compare.md#0x1_Compare_EQUAL">EQUAL</a>
    <b>else</b> <b>if</b> (i1 &lt; i2) <a href="Compare.md#0x1_Compare_LESS_THAN">LESS_THAN</a>
    <b>else</b> <a href="Compare.md#0x1_Compare_GREATER_THAN">GREATER_THAN</a>
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_cmp_bcs_bytes"></a>

### Function `cmp_bcs_bytes`


<pre><code><b>public</b> <b>fun</b> <a href="Compare.md#0x1_Compare_cmp_bcs_bytes">cmp_bcs_bytes</a>(v1: &vector&lt;u8&gt;, v2: &vector&lt;u8&gt;): u8
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_cmp_u8"></a>

### Function `cmp_u8`


<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u8">cmp_u8</a>(i1: u8, i2: u8): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_cmp_u64"></a>

### Function `cmp_u64`


<pre><code><b>fun</b> <a href="Compare.md#0x1_Compare_cmp_u64">cmp_u64</a>(i1: u64, i2: u64): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
