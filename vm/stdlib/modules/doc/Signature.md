
<a name="0x1_Signature"></a>

# Module `0x1::Signature`

Contains functions for [ed25519](https://en.wikipedia.org/wiki/EdDSA) digital signatures.


-  [Function `ed25519_validate_pubkey`](#0x1_Signature_ed25519_validate_pubkey)
-  [Function `ed25519_verify`](#0x1_Signature_ed25519_verify)
-  [Specification](#@Specification_0)


<pre><code></code></pre>



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

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> intrinsic = <b>true</b>;
</code></pre>
