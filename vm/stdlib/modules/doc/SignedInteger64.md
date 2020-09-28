
<a name="0x1_SignedInteger64"></a>

# Module `0x1::SignedInteger64`



-  [Struct <code><a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a></code>](#0x1_SignedInteger64_SignedInteger64)
-  [Function <code>multiply_u64</code>](#0x1_SignedInteger64_multiply_u64)
-  [Function <code>divide_u64</code>](#0x1_SignedInteger64_divide_u64)
-  [Function <code>sub_u64</code>](#0x1_SignedInteger64_sub_u64)
-  [Function <code>add_u64</code>](#0x1_SignedInteger64_add_u64)
-  [Function <code>create_from_raw_value</code>](#0x1_SignedInteger64_create_from_raw_value)
-  [Function <code>get_value</code>](#0x1_SignedInteger64_get_value)
-  [Function <code>is_negative</code>](#0x1_SignedInteger64_is_negative)
-  [Specification](#@Specification_0)
    -  [Function <code>multiply_u64</code>](#@Specification_0_multiply_u64)
    -  [Function <code>divide_u64</code>](#@Specification_0_divide_u64)
    -  [Function <code>sub_u64</code>](#@Specification_0_sub_u64)
    -  [Function <code>add_u64</code>](#@Specification_0_add_u64)
    -  [Function <code>create_from_raw_value</code>](#@Specification_0_create_from_raw_value)
    -  [Function <code>get_value</code>](#@Specification_0_get_value)
    -  [Function <code>is_negative</code>](#@Specification_0_is_negative)


<a name="0x1_SignedInteger64_SignedInteger64"></a>

## Struct `SignedInteger64`



<pre><code><b>struct</b> <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>is_negative: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SignedInteger64_multiply_u64"></a>

## Function `multiply_u64`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> {
    <b>let</b> product = multiplier.value * num;
    <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (product <b>as</b> u64), is_negative: multiplier.is_negative }
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_divide_u64"></a>

## Function `divide_u64`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_divide_u64">divide_u64</a>(num: u64, divisor: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_divide_u64">divide_u64</a>(num: u64, divisor: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> {
    <b>let</b> quotient = num / divisor.value;
    <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (quotient <b>as</b> u64), is_negative: divisor.is_negative }
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_sub_u64"></a>

## Function `sub_u64`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_sub_u64">sub_u64</a>(num: u64, minus: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_sub_u64">sub_u64</a>(num: u64, minus: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> {
    <b>if</b> (minus.is_negative) {
        <b>let</b> result = num + minus.value;
        <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>false</b> }
    } <b>else</b> {
        <b>if</b> (num &gt; minus.value)  {
            <b>let</b> result = num - minus.value;
            <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>false</b> }
        }<b>else</b> {
            <b>let</b> result = minus.value - num;
            <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>true</b> }
        }
    }
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_add_u64"></a>

## Function `add_u64`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_add_u64">add_u64</a>(num: u64, addend: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_add_u64">add_u64</a>(num: u64, addend: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> {
    <b>if</b> (addend.is_negative) {
       <b>if</b> (num &gt; addend.value)  {
           <b>let</b> result = num - addend.value;
           <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>false</b> }
       }<b>else</b> {
           <b>let</b> result = addend.value - num;
           <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>true</b> }
       }
    } <b>else</b> {
         <b>let</b> result = num + addend.value;
         <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value: (result <b>as</b> u64), is_negative: <b>false</b> }
    }
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_create_from_raw_value"></a>

## Function `create_from_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_create_from_raw_value">create_from_raw_value</a>(value: u64, is_negative: bool): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_create_from_raw_value">create_from_raw_value</a>(value: u64, is_negative: bool): <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> {
    <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value, is_negative }
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_get_value"></a>

## Function `get_value`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_get_value">get_value</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_get_value">get_value</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): u64 {
    num.value
}
</code></pre>



</details>

<a name="0x1_SignedInteger64_is_negative"></a>

## Function `is_negative`



<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_is_negative">is_negative</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_is_negative">is_negative</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a>): bool {
    num.is_negative
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_multiply_u64"></a>

### Function `multiply_u64`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>




<pre><code><b>aborts_if</b> multiplier.value * num &gt; max_u64();
</code></pre>



<a name="@Specification_0_divide_u64"></a>

### Function `divide_u64`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_divide_u64">divide_u64</a>(num: u64, divisor: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>




<pre><code><b>aborts_if</b> divisor.value == 0;
</code></pre>



<a name="@Specification_0_sub_u64"></a>

### Function `sub_u64`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_sub_u64">sub_u64</a>(num: u64, minus: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>




<pre><code><b>aborts_if</b> minus.is_negative && num + minus.value &gt; max_u64();
</code></pre>



<a name="@Specification_0_add_u64"></a>

### Function `add_u64`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_add_u64">add_u64</a>(num: u64, addend: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>




<pre><code><b>aborts_if</b> !addend.is_negative && num + addend.value &gt; max_u64();
</code></pre>



<a name="@Specification_0_create_from_raw_value"></a>

### Function `create_from_raw_value`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_create_from_raw_value">create_from_raw_value</a>(value: u64, is_negative: bool): <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <a href="SignedInteger64.md#0x1_SignedInteger64">SignedInteger64</a> { value, is_negative };
</code></pre>



<a name="@Specification_0_get_value"></a>

### Function `get_value`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_get_value">get_value</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): u64
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == num.value;
</code></pre>



<a name="@Specification_0_is_negative"></a>

### Function `is_negative`


<pre><code><b>public</b> <b>fun</b> <a href="SignedInteger64.md#0x1_SignedInteger64_is_negative">is_negative</a>(num: <a href="SignedInteger64.md#0x1_SignedInteger64_SignedInteger64">SignedInteger64::SignedInteger64</a>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == num.is_negative;
</code></pre>
