
<a name="0x1_Math"></a>

# Module `0x1::Math`

### Table of Contents

-  [Function `sqrt`](#0x1_Math_sqrt)
-  [Function `pow`](#0x1_Math_pow)



<a name="0x1_Math_sqrt"></a>

## Function `sqrt`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Math_sqrt">sqrt</a>(y: u128): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Math_sqrt">sqrt</a>(y: u128): u64 {
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



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Math_pow">pow</a>(x: u64, y: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Math_pow">pow</a>(x: u64, y: u64): u128 {
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
