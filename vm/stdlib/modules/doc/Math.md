
<a name="0x1_Math"></a>

# Module `0x1::Math`



-  [Function <code>sqrt</code>](#0x1_Math_sqrt)
-  [Function <code>pow</code>](#0x1_Math_pow)
-  [Function <code>percent_multi</code>](#0x1_Math_percent_multi)
-  [Specification](#@Specification_0)
    -  [Function <code>sqrt</code>](#@Specification_0_sqrt)
    -  [Function <code>pow</code>](#@Specification_0_pow)


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

<a name="0x1_Math_percent_multi"></a>

## Function `percent_multi`



<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_percent_multi">percent_multi</a>(x: u128, y: u128, z: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Math.md#0x1_Math_percent_multi">percent_multi</a>(x: u128, y: u128, z: u128): u128 {
    <b>if</b> (y &gt; z) {
        y / z * x
    }<b>else</b> {
        x * y / z
    }
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
