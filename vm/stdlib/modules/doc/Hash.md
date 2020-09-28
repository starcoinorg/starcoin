
<a name="0x1_Hash"></a>

# Module `0x1::Hash`



-  [Function <code>sha2_256</code>](#0x1_Hash_sha2_256)
-  [Function <code>sha3_256</code>](#0x1_Hash_sha3_256)
-  [Specification](#@Specification_0)


<a name="0x1_Hash_sha2_256"></a>

## Function `sha2_256`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha2_256">sha2_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha2_256">sha2_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_Hash_sha3_256"></a>

## Function `sha3_256`



<pre><code><b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha3_256">sha3_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Hash.md#0x1_Hash_sha3_256">sha3_256</a>(data: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>
