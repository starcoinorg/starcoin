
<a name="0x1_Signature"></a>

# Module `0x1::Signature`

Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.


-  [Function `ed25519_validate_pubkey`](#0x1_Signature_ed25519_validate_pubkey)
-  [Function `ed25519_verify`](#0x1_Signature_ed25519_verify)
-  [Function `native_ecrecover`](#0x1_Signature_native_ecrecover)
-  [Function `ecrecover`](#0x1_Signature_ecrecover)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="Signature.md#0x1_EVMAddress">0x1::EVMAddress</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_Signature_ed25519_validate_pubkey"></a>

## Function `ed25519_validate_pubkey`



<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">ed25519_validate_pubkey</a>(public_key: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x1_Signature_ed25519_verify"></a>

## Function `ed25519_verify`



<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ed25519_verify">ed25519_verify</a>(signature: vector&lt;u8&gt;, public_key: vector&lt;u8&gt;, message: vector&lt;u8&gt;): bool;
</code></pre>



</details>

<a name="0x1_Signature_native_ecrecover"></a>

## Function `native_ecrecover`

recover address from ECDSA signature, if recover fail, return an empty vector<u8>


<pre><code><b>fun</b> <a href="Signature.md#0x1_Signature_native_ecrecover">native_ecrecover</a>(hash: vector&lt;u8&gt;, signature: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Signature.md#0x1_Signature_native_ecrecover">native_ecrecover</a>(hash: vector&lt;u8&gt;, signature: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>



</details>

<a name="0x1_Signature_ecrecover"></a>

## Function `ecrecover`

recover address from ECDSA signature, if recover fail, return None


<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ecrecover">ecrecover</a>(hash: vector&lt;u8&gt;, signature: vector&lt;u8&gt;): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="Signature.md#0x1_EVMAddress_EVMAddress">EVMAddress::EVMAddress</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Signature.md#0x1_Signature_ecrecover">ecrecover</a>(hash: vector&lt;u8&gt;, signature: vector&lt;u8&gt;):<a href="Option.md#0x1_Option">Option</a>&lt;<a href="Signature.md#0x1_EVMAddress">EVMAddress</a>&gt;{
    <b>let</b> bytes = <a href="Signature.md#0x1_Signature_native_ecrecover">native_ecrecover</a>(hash, signature);
    <b>if</b> (<a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&bytes)){
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="Signature.md#0x1_EVMAddress">EVMAddress</a>&gt;()
    }<b>else</b>{
        <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="Signature.md#0x1_EVMAddress_new">EVMAddress::new</a>(bytes))
    }
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>
