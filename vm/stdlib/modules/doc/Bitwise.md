
<a name="0x1_BitOperators"></a>

# Module `0x1::BitOperators`

### Table of Contents

-  [Function `and`](#0x1_BitOperators_and)
-  [Function `or`](#0x1_BitOperators_or)
-  [Function `xor`](#0x1_BitOperators_xor)
-  [Function `not`](#0x1_BitOperators_not)
-  [Function `lshift`](#0x1_BitOperators_lshift)
-  [Function `rshift`](#0x1_BitOperators_rshift)



<a name="0x1_BitOperators_and"></a>

## Function `and`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_and">and</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_and">and</a>(x: u64, y: u64): u64 {
    (x & y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_or"></a>

## Function `or`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_or">or</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_or">or</a>(x: u64, y: u64): u64 {
    (x | y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_xor"></a>

## Function `xor`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_xor">xor</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_xor">xor</a>(x: u64, y: u64): u64 {
    (x ^ y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_not"></a>

## Function `not`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_not">not</a>(x: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_not">not</a>(x: u64): u64 {
   (x ^ 18446744073709551615u64 <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_lshift"></a>

## Function `lshift`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_lshift">lshift</a>(x: u64, n: u8): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_lshift">lshift</a>(x: u64, n: u8): u64 {
    (x &lt;&lt; n  <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_rshift"></a>

## Function `rshift`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_rshift">rshift</a>(x: u64, n: u8): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_BitOperators_rshift">rshift</a>(x: u64, n: u8): u64 {
    (x &gt;&gt; n  <b>as</b> u64)
}
</code></pre>



</details>
