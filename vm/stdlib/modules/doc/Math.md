
<a name="0x1_Math"></a>

# Module `0x1::Math`



-  [Const <code><a href="Math.md#0x1_Math_U64_MAX">U64_MAX</a></code>](#0x1_Math_U64_MAX)
-  [Const <code><a href="Math.md#0x1_Math_U128_MAX">U128_MAX</a></code>](#0x1_Math_U128_MAX)
-  [Function <code>u64_max</code>](#0x1_Math_u64_max)
-  [Function <code>u128_max</code>](#0x1_Math_u128_max)
-  [Function <code>sqrt</code>](#0x1_Math_sqrt)
-  [Function <code>pow</code>](#0x1_Math_pow)
-  [Function <code>mul_div</code>](#0x1_Math_mul_div)
-  [Specification](#@Specification_0)
    -  [Function <code>sqrt</code>](#@Specification_0_sqrt)
    -  [Function <code>pow</code>](#@Specification_0_pow)


<a name="0x1_Math_U64_MAX"></a>

## Const `U64_MAX`



<pre><code><b>const</b> <a href="Math.md#0x1_Math_U64_MAX">U64_MAX</a>: u64 = 18446744073709551615;
</code></pre>



<a name="0x1_Math_U128_MAX"></a>

## Const `U128_MAX`



<pre><code><b>const</b> <a href="Math.md#0x1_Math_U128_MAX">U128_MAX</a>: u128 = 340282366920938463463374607431768211455;
</code></pre>



<a name="0x1_Math_u64_max"></a>

## Function `u64_max`



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



<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_mul_div">mul_div</a>(x: u128, y: u128, z: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_mul_div">mul_div</a>(x: u128, y: u128, z: u128): u128 {
    <b>if</b> ( y  == z ) {
        <b>return</b> x
    };
    <b>if</b> ( x &gt; z) {
        <b>return</b> x/z*y
    };
    <b>let</b> a = x / z;
    <b>let</b> b = x % z;
    //x = a * z + b;
    <b>let</b> c = y / z;
    <b>let</b> d = y % z;
    //y = c * z + d;
    a * b * z + a * d + b * c + b * d / z
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_sqrt"></a>

### Function `sqrt`


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_sqrt">sqrt</a>(y: u128): u64
</code></pre>




<pre><code>pragma verify = <b>false</b>;
pragma timeout = 120;
<b>aborts_if</b> y &gt;= 4 && y / (y/2 +1) + y/2 +1 &gt; max_u128();
<b>aborts_if</b> y &gt;= 4 && y / (y/2 +1) &gt; max_u128();
</code></pre>



<a name="@Specification_0_pow"></a>

### Function `pow`


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_pow">pow</a>(x: u64, y: u64): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
