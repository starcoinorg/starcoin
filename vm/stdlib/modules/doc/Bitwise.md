
<a name="0x1_BitOperators"></a>

# Module `0x1::BitOperators`



-  [Function <code>and</code>](#0x1_BitOperators_and)
-  [Function <code>or</code>](#0x1_BitOperators_or)
-  [Function <code>xor</code>](#0x1_BitOperators_xor)
-  [Function <code>not</code>](#0x1_BitOperators_not)
-  [Function <code>lshift</code>](#0x1_BitOperators_lshift)
-  [Function <code>rshift</code>](#0x1_BitOperators_rshift)
-  [Specification](#@Specification_0)


<a name="0x1_BitOperators_and"></a>

## Function `and`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_and">and</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_and">and</a>(x: u64, y: u64): u64 {
    (x & y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_or"></a>

## Function `or`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_or">or</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_or">or</a>(x: u64, y: u64): u64 {
    (x | y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_xor"></a>

## Function `xor`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_xor">xor</a>(x: u64, y: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_xor">xor</a>(x: u64, y: u64): u64 {
    (x ^ y <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_not"></a>

## Function `not`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_not">not</a>(x: u64): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_not">not</a>(x: u64): u64 {
   (x ^ 18446744073709551615u64 <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_lshift"></a>

## Function `lshift`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_lshift">lshift</a>(x: u64, n: u8): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_lshift">lshift</a>(x: u64, n: u8): u64 {
    (x &lt;&lt; n  <b>as</b> u64)
}
</code></pre>



</details>

<a name="0x1_BitOperators_rshift"></a>

## Function `rshift`



<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_rshift">rshift</a>(x: u64, n: u8): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Bitwise.md#0x1_BitOperators_rshift">rshift</a>(x: u64, n: u8): u64 {
    (x &gt;&gt; n  <b>as</b> u64)
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
</code></pre>
