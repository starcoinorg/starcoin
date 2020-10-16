
<a name="0x1_LCS"></a>

# Module `0x1::LCS`



-  [Function `to_bytes`](#0x1_LCS_to_bytes)
-  [Function `to_address`](#0x1_LCS_to_address)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



<a name="0x1_LCS_to_bytes"></a>

## Function `to_bytes`



<pre><code><b>public</b> <b>fun</b> <a href="LCS.md#0x1_LCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="LCS.md#0x1_LCS_to_bytes">to_bytes</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_LCS_to_address"></a>

## Function `to_address`



<pre><code><b>public</b> <b>fun</b> <a href="LCS.md#0x1_LCS_to_address">to_address</a>(key_bytes: vector&lt;u8&gt;): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="LCS.md#0x1_LCS_to_address">to_address</a>(key_bytes: vector&lt;u8&gt;): address;
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a name="0x1_LCS_serialize"></a>


<pre><code><b>native</b> <b>define</b> <a href="LCS.md#0x1_LCS_serialize">serialize</a>&lt;MoveValue&gt;(v: &MoveValue): vector&lt;u8&gt;;
</code></pre>
