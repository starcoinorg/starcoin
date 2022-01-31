
<a name="0x1_FixedPoint32"></a>

# Module `0x1::FixedPoint32`

The module provide operations for FixedPoint32.


-  [Struct `FixedPoint32`](#0x1_FixedPoint32_FixedPoint32)
-  [Constants](#@Constants_0)
-  [Function `multiply_u64`](#0x1_FixedPoint32_multiply_u64)
-  [Function `divide_u64`](#0x1_FixedPoint32_divide_u64)
-  [Function `create_from_rational`](#0x1_FixedPoint32_create_from_rational)
-  [Function `create_from_raw_value`](#0x1_FixedPoint32_create_from_raw_value)
-  [Function `get_raw_value`](#0x1_FixedPoint32_get_raw_value)
-  [Specification](#@Specification_1)
    -  [Function `multiply_u64`](#@Specification_1_multiply_u64)
    -  [Function `divide_u64`](#@Specification_1_divide_u64)
    -  [Function `create_from_rational`](#@Specification_1_create_from_rational)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="0x1_FixedPoint32_FixedPoint32"></a>

## Struct `FixedPoint32`

Define a fixed-point numeric type with 32 fractional bits.
This is just a u64 integer but it is wrapped in a struct to
make a unique type.


<pre><code><b>struct</b> <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_FixedPoint32_MAX_U64"></a>



<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a name="0x1_FixedPoint32_EDENOMINATOR"></a>

The denominator provided was zero


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>: u64 = 101;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION"></a>

The quotient value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION">EDIVISION</a>: u64 = 102;
</code></pre>



<a name="0x1_FixedPoint32_EDIVISION_BY_ZERO"></a>

A division by zero was encountered


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>: u64 = 104;
</code></pre>



<a name="0x1_FixedPoint32_EMULTIPLICATION"></a>

The multiplied value would be too large to be held in a <code>u64</code>


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>: u64 = 103;
</code></pre>



<a name="0x1_FixedPoint32_ERATIO_OUT_OF_RANGE"></a>

The computed ratio when converting to a <code><a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code> would be unrepresentable


<pre><code><b>const</b> <a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>: u64 = 105;
</code></pre>



<a name="0x1_FixedPoint32_multiply_u64"></a>

## Function `multiply_u64`

Multiply a u64 integer by a fixed-point number, truncating any
fractional part of the product. This will abort if the product
overflows.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // The product of two 64 bit values <b>has</b> 128 bits, so perform the
    // multiplication <b>with</b> u128 types and keep the full 128 bit product
    // <b>to</b> avoid losing accuracy.
    <b>let</b> unscaled_product = (val <b>as</b> u128) * (multiplier.value <b>as</b> u128);
    // The unscaled product <b>has</b> 32 fractional bits (from the multiplier)
    // so rescale it by shifting away the low bits.
    <b>let</b> product = unscaled_product &gt;&gt; 32;
    // Check whether the value is too large.
    <b>assert</b>!(product &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EMULTIPLICATION">EMULTIPLICATION</a>));
    (product <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_divide_u64"></a>

## Function `divide_u64`

Divide a u64 integer by a fixed-point number, truncating any
fractional part of the quotient. This will abort if the divisor
is zero or if the quotient overflows.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    // Check for division by zero.
    <b>assert</b>!(divisor.value != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION_BY_ZERO">EDIVISION_BY_ZERO</a>));
    // First convert <b>to</b> 128 bits and then shift left <b>to</b>
    // add 32 fractional zero bits <b>to</b> the dividend.
    <b>let</b> scaled_value = (val <b>as</b> u128) &lt;&lt; 32;
    <b>let</b> quotient = scaled_value / (divisor.value <b>as</b> u128);
    // Check whether the value is too large.
    <b>assert</b>!(quotient &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDIVISION">EDIVISION</a>));
    // the value may be too large, which will cause the cast <b>to</b> fail
    // <b>with</b> an arithmetic error.
    (quotient <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_rational"></a>

## Function `create_from_rational`

Create a fixed-point value from a rational number specified by its
numerator and denominator. This function is for convenience; it is also
perfectly fine to create a fixed-point value by directly specifying the
raw value. This will abort if the denominator is zero or if the ratio is
not in the range 2^-32 .. 2^32-1.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> {
    // If the denominator is zero, this will <b>abort</b>.
    // Scale the numerator <b>to</b> have 64 fractional bits and the denominator
    // <b>to</b> have 32 fractional bits, so that the quotient will have 32
    // fractional bits.
    <b>let</b> scaled_numerator = (numerator <b>as</b> u128) &lt;&lt; 64;
    <b>let</b> scaled_denominator = (denominator <b>as</b> u128) &lt;&lt; 32;
    <b>assert</b>!(scaled_denominator != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_EDENOMINATOR">EDENOMINATOR</a>));
    <b>let</b> quotient = scaled_numerator / scaled_denominator;
    <b>assert</b>!(quotient != 0 || numerator == 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
    // Return the quotient <b>as</b> a fixed-point number. We first need <b>to</b> check whether the cast
    // can succeed.
    <b>assert</b>!(quotient &lt;= <a href="FixedPoint32.md#0x1_FixedPoint32_MAX_U64">MAX_U64</a>, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="FixedPoint32.md#0x1_FixedPoint32_ERATIO_OUT_OF_RANGE">ERATIO_OUT_OF_RANGE</a>));
    <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a> { value: (quotient <b>as</b> u64) }
}
</code></pre>



</details>

<a name="0x1_FixedPoint32_create_from_raw_value"></a>

## Function `create_from_raw_value`

create a fixedpoint 32  from u64.


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

Accessor for the raw u64 value. Other less common operations, such as
adding or subtracting FixedPoint32 values, can be done using the raw
values directly.


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_get_raw_value">get_raw_value</a>(num: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64 {
    num.value
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>


Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">Self::multiply_u64</a></code>.


<a name="0x1_FixedPoint32_spec_multiply_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">Self::divide_u64</a></code>.


<a name="0x1_FixedPoint32_spec_divide_u64"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>): u64;
</code></pre>


Uninterpreted function for <code><a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">Self::create_from_rational</a></code>.


<a name="0x1_FixedPoint32_spec_create_from_rational"></a>


<pre><code><b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a>;
</code></pre>



<a name="@Specification_1_multiply_u64"></a>

### Function `multiply_u64`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_multiply_u64">multiply_u64</a>(val: u64, multiplier: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



Currently, we ignore the actual implementation of this function in verification
and treat it as uninterpreted, which simplifies the verification problem significantly.
This way we avoid the non-linear arithmetic problem presented by this function.

Abstracting this and related functions is possible because the correctness of currency
conversion (where <code><a href="FixedPoint32.md#0x1_FixedPoint32">FixedPoint32</a></code> is used for) is not relevant for the rest of the contract
control flow, so we can assume some arbitrary (but fixed) behavior here.


<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_multiply_u64">spec_multiply_u64</a>(val, multiplier);
</code></pre>



<a name="@Specification_1_divide_u64"></a>

### Function `divide_u64`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_divide_u64">divide_u64</a>(val: u64, divisor: <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>): u64
</code></pre>



See comment at <code>Self::multiply_64</code>.


<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_divide_u64">spec_divide_u64</a>(val, divisor);
</code></pre>



<a name="@Specification_1_create_from_rational"></a>

### Function `create_from_rational`


<pre><code><b>public</b> <b>fun</b> <a href="FixedPoint32.md#0x1_FixedPoint32_create_from_rational">create_from_rational</a>(numerator: u64, denominator: u64): <a href="FixedPoint32.md#0x1_FixedPoint32_FixedPoint32">FixedPoint32::FixedPoint32</a>
</code></pre>



See comment at <code>Self::multiply_64</code>.


<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>ensures</b> result == <a href="FixedPoint32.md#0x1_FixedPoint32_spec_create_from_rational">spec_create_from_rational</a>(numerator, denominator);
</code></pre>
