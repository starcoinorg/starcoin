
<a name="0x1_FixedPoint32"></a>

# Module `0x1::FixedPoint32`



-  [Struct <code><a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code>](#0x1_FixedPoint32_FixedPoint32)
-  [Function <code>multiply_u64</code>](#0x1_FixedPoint32_multiply_u64)
-  [Function <code>divide_u64</code>](#0x1_FixedPoint32_divide_u64)
-  [Function <code>create_from_rational</code>](#0x1_FixedPoint32_create_from_rational)
-  [Function <code>create_from_raw_value</code>](#0x1_FixedPoint32_create_from_raw_value)
-  [Function <code>get_raw_value</code>](#0x1_FixedPoint32_get_raw_value)
-  [Specification](#@Specification_0)
    -  [Function <code>multiply_u64</code>](#@Specification_0_multiply_u64)
    -  [Function <code>divide_u64</code>](#@Specification_0_divide_u64)
    -  [Function <code>create_from_rational</code>](#@Specification_0_create_from_rational)


<a name="0x1_FixedPoint32_FixedPoint32"></a>

## Struct `FixedPoint32`



<pre><code><b>struct</b> <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>value: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`



<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // The product of two 64 bit values has 128 bits, so perform the
    // multiplication <b>with</b> u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (num <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product has 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // Convert back <b>to</b> u64. If the multiplier is larger than 1.0,
    // the value may be too large, which will cause the cast <b>to</b> fail
    // <b>with</b> an arithmetic error.
    (product <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_divide_u64"></a>

## Function `divide_u64`



<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (num <b>as</b> u128) &lt;&lt; 32;
    // Divide and convert the quotient <b>to</b> 64 bits. If the divisor is zero,
    // this will fail <b>with</b> a divide-by-zero error.
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // Convert back <b>to</b> u64. If the divisor is less than 1.0,
    // the value may be too large, which will cause the cast <b>to</b> fail
    // <b>with</b> an arithmetic error.
    (quotient <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_rational"></a>

## Function `create_from_rational`



<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    // If the denominator is zero, this will fail <b>with</b> a divide-by-zero
    // error.
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    // Check for underflow. Truncating <b>to</b> zero might be the desired result,
    // but <b>if</b> you really want a ratio of zero, it is easy <b>to</b> create that
    // from a raw value.
    <b>assert</b>(quotient != 0 || numerator == 0, 16);
    // Return the quotient <b>as</b> a fixed-point number. The cast will fail
    // <b>with</b> an arithmetic error <b>if</b> the number is too large.
    <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> { value: (quotient <b>as</b> u64) }
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_raw_value"></a>

## Function `create_from_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_raw_value">create_from_raw_value</a>(value: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> { value }
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_get_raw_value"></a>

## Function `get_raw_value`



<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    num.value
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">Self::multiply_u64</a></code>.


<a name="0x1_FixedPoint32_spec_multiply_u64"></a>


<pre><code><b>define</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">Self::divide_u64</a></code>.


<a name="0x1_FixedPoint32_spec_divide_u64"></a>


<pre><code><b>define</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">Self::create_from_rational</a></code>.


<a name="0x1_FixedPoint32_spec_create_from_rational"></a>


<pre><code><b>define</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
</code></pre>



<a name="@Specification_0_multiply_u64"></a>

### Function `multiply_u64`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(num: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



Currently, we ignore the actual implementation of this function in verification
and treat it as uninterpreted, which simplifies the verification problem significantly.
This way we avoid the non-linear arithmetic problem presented by this function.

Abstracting this and related functions is possible because the correctness of currency
conversion (where <code><a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code> is used for) is not relevant for the rest of the contract
control flow, so we can assume some arbitrary (but fixed) behavior here.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(num, multiplier);
</code></pre>



<a name="@Specification_0_divide_u64"></a>

### Function `divide_u64`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(num: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



See comment at <code>Self::multiply_64</code>.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(num, divisor);
</code></pre>



<a name="@Specification_0_create_from_rational"></a>

### Function `create_from_rational`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



See comment at <code>Self::multiply_64</code>.


<pre><code>pragma opaque = <b>true</b>;
pragma verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator, denominator);
</code></pre>
