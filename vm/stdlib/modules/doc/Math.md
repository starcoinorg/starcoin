
<a name="0x1_Math"></a>

# Module `0x1::Math`

The module provide some improved math calculations.


-  [Constants](#@Constants_0)
-  [Function `u64_max`](#0x1_Math_u64_max)
-  [Function `u128_max`](#0x1_Math_u128_max)
-  [Function `sqrt`](#0x1_Math_sqrt)
-  [Function `pow`](#0x1_Math_pow)
-  [Function `mul_div`](#0x1_Math_mul_div)
-  [Function `sum`](#0x1_Math_sum)
-  [Function `avg`](#0x1_Math_avg)
-  [Specification](#@Specification_1)
    -  [Function `sqrt`](#@Specification_1_sqrt)
    -  [Function `pow`](#@Specification_1_pow)
    -  [Function `mul_div`](#@Specification_1_mul_div)


<pre><code><b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_Math_U128_MAX"></a>



<pre><code><b>const</b> <a href="Math.md#0x1_Math_U128_MAX">U128_MAX</a>: u128 = 340282366920938463463374607431768211455;
</code></pre>



<a name="0x1_Math_U64_MAX"></a>



<pre><code><b>const</b> <a href="Math.md#0x1_Math_U64_MAX">U64_MAX</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_Math_u64_max"></a>

## Function `u64_max`

u64::MAX


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_u64_max">u64_max</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_u64_max">u64_max</a>(): u64 {
    <a href="Math.md#0x1_Math_U64_MAX">U64_MAX</a>
}
</code></pre>



</details>

<a name="0x1_Math_u128_max"></a>

## Function `u128_max`

u128::MAX


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_u128_max">u128_max</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_u128_max">u128_max</a>(): u128 {
    <a href="Math.md#0x1_Math_U128_MAX">U128_MAX</a>
}
</code></pre>



</details>

<a name="0x1_Math_sqrt"></a>

## Function `sqrt`

babylonian method (https://en.wikipedia.org/wiki/Methods_of_computing_square_roots#Babylonian_method)


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sqrt">sqrt</a>(y: u128): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sqrt">sqrt</a>(y: u128): u64 {
    <b>if</b> (y &lt; 4) {
        <b>if</b> (y == 0) {
            0u64
        } <b>else</b> {
            1u64
        }
    } <b>else</b> {
        <b>let</b> z = y;
        <b>let</b> x = y / 2 + 1;
        <b>while</b> (x &lt; z) {
            z = x;
            x = (y / x + x) / 2;
        };
        (z <b>as</b> u64)
    }
}
</code></pre>



</details>

<a name="0x1_Math_pow"></a>

## Function `pow`

calculate the <code>y</code> pow of <code>x</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_pow">pow</a>(x: u64, y: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_pow">pow</a>(x: u64, y: u64): u128 {
    <b>let</b> result = 1u128;
    <b>let</b> z = y;
    <b>let</b> u = (x <b>as</b> u128);
    <b>while</b> (z &gt; 0) {
        <b>if</b> (z % 2 == 1) {
            result = (u * result <b>as</b> u128);
        };
        u = (u * u <b>as</b> u128);
        z = z / 2;
    };
    result
}
</code></pre>



</details>

<a name="0x1_Math_mul_div"></a>

## Function `mul_div`

https://medium.com/coinmonks/math-in-solidity-part-3-percents-and-proportions-4db014e080b1
calculate x * y /z with as little loss of precision as possible and avoid overflow


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_mul_div">mul_div</a>(x: u128, y: u128, z: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_mul_div">mul_div</a>(x: u128, y: u128, z: u128): u128 {
    <b>if</b> (y == z) {
        <b>return</b> x
    };
    <b>if</b> (x == z) {
        <b>return</b> y
    };
    <b>let</b> a = x / z;
    <b>let</b> b = x % z;
    //x = a * z + b;
    <b>let</b> c = y / z;
    <b>let</b> d = y % z;
    //y = c * z + d;
    a * c * z + a * d + b * c + b * d / z
}
</code></pre>



</details>

<a name="0x1_Math_sum"></a>

## Function `sum`

calculate sum of nums


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sum">sum</a>(nums: &vector&lt;u128&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sum">sum</a>(nums: &vector&lt;u128&gt;): u128 {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(nums);
    <b>let</b> i = 0;
    <b>let</b> sum = 0;
    <b>while</b> (i &lt; len){
        sum = sum + *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(nums, i);
        i = i + 1;
    };
    sum
}
</code></pre>



</details>

<a name="0x1_Math_avg"></a>

## Function `avg`

calculate average of nums


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_avg">avg</a>(nums: &vector&lt;u128&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_avg">avg</a>(nums: &vector&lt;u128&gt;): u128{
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(nums);
    <b>let</b> sum = <a href="Math.md#0x1_Math_sum">sum</a>(nums);
    sum/(len <b>as</b> u128)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_sqrt"></a>

### Function `sqrt`


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sqrt">sqrt</a>(y: u128): u64
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Math.md#0x1_Math_spec_sqrt">spec_sqrt</a>();
</code></pre>


We use an uninterpreted function to represent the result of sqrt. The actual value
does not matter for the verification of callers.


<a name="0x1_Math_spec_sqrt"></a>


<pre><code><b>fun</b> <a href="Math.md#0x1_Math_spec_sqrt">spec_sqrt</a>(): u128;
</code></pre>



<a name="@Specification_1_pow"></a>

### Function `pow`


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_pow">pow</a>(x: u64, y: u64): u128
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Math.md#0x1_Math_spec_pow">spec_pow</a>();
</code></pre>


We use an uninterpreted function to represent the result of pow. The actual value
does not matter for the verification of callers.


<a name="0x1_Math_spec_pow"></a>


<pre><code><b>fun</b> <a href="Math.md#0x1_Math_spec_pow">spec_pow</a>(): u128;
</code></pre>



<a name="@Specification_1_mul_div"></a>

### Function `mul_div`


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_mul_div">mul_div</a>(x: u128, y: u128, z: u128): u128
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>include</b> <a href="Math.md#0x1_Math_MulDivAbortsIf">MulDivAbortsIf</a>;
<b>aborts_if</b> [abstract] <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Math.md#0x1_Math_spec_mul_div">spec_mul_div</a>();
</code></pre>




<a name="0x1_Math_MulDivAbortsIf"></a>


<pre><code><b>schema</b> <a href="Math.md#0x1_Math_MulDivAbortsIf">MulDivAbortsIf</a> {
    x: u128;
    y: u128;
    z: u128;
    <b>aborts_if</b> y != z && x &gt; z && z == 0;
    <b>aborts_if</b> y != z && x &gt; z && z!=0 && x/z*y &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && z == 0;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (x % z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (x % z) * z &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (y % z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (x % z) * z + x / z * (y % z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x % z * (y / z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x % z * (y % z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x % z * (y % z) / z &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) &gt; MAX_U128;
    <b>aborts_if</b> y != z && x &lt;= z && x / z * (x % z) * z + x / z * (y % z) + x % z * (y / z) + x % z * (y % z) / z &gt; MAX_U128;
}
</code></pre>




<a name="0x1_Math_spec_mul_div"></a>


<pre><code><b>fun</b> <a href="Math.md#0x1_Math_spec_mul_div">spec_mul_div</a>(): u128;
</code></pre>
