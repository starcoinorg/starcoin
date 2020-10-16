
<a name="0x1_Authenticator"></a>

# Module `0x1::Authenticator`



-  [Struct <code><a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a></code>](#0x1_Authenticator_MultiEd25519PublicKey)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_AUTHENTICATION_KEY_LENGTH">AUTHENTICATION_KEY_LENGTH</a></code>](#0x1_Authenticator_AUTHENTICATION_KEY_LENGTH)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_ED25519_SCHEME_ID">ED25519_SCHEME_ID</a></code>](#0x1_Authenticator_ED25519_SCHEME_ID)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a></code>](#0x1_Authenticator_MULTI_ED25519_SCHEME_ID)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a></code>](#0x1_Authenticator_MAX_MULTI_ED25519_KEYS)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_EWRONG_AUTHENTICATION_KEY_LENGTH">EWRONG_AUTHENTICATION_KEY_LENGTH</a></code>](#0x1_Authenticator_EWRONG_AUTHENTICATION_KEY_LENGTH)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a></code>](#0x1_Authenticator_EZERO_THRESHOLD)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a></code>](#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD)
-  [Const <code><a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a></code>](#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD)
-  [Function <code>create_multi_ed25519</code>](#0x1_Authenticator_create_multi_ed25519)
-  [Function <code>ed25519_authentication_key</code>](#0x1_Authenticator_ed25519_authentication_key)
-  [Function <code>derived_address</code>](#0x1_Authenticator_derived_address)
-  [Function <code>multi_ed25519_authentication_key</code>](#0x1_Authenticator_multi_ed25519_authentication_key)
-  [Function <code>public_keys</code>](#0x1_Authenticator_public_keys)
-  [Function <code>threshold</code>](#0x1_Authenticator_threshold)
-  [Specification](#@Specification_0)
    -  [Function <code>create_multi_ed25519</code>](#@Specification_0_create_multi_ed25519)
    -  [Function <code>ed25519_authentication_key</code>](#@Specification_0_ed25519_authentication_key)
    -  [Function <code>derived_address</code>](#@Specification_0_derived_address)
    -  [Function <code>multi_ed25519_authentication_key</code>](#@Specification_0_multi_ed25519_authentication_key)
    -  [Function <code>public_keys</code>](#@Specification_0_public_keys)
    -  [Function <code>threshold</code>](#@Specification_0_threshold)


<a name="0x1_Authenticator_MultiEd25519PublicKey"></a>

## Struct `MultiEd25519PublicKey`



<pre><code><b>struct</b> <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>public_keys: vector&lt;vector&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>threshold: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Authenticator_AUTHENTICATION_KEY_LENGTH"></a>

## Const `AUTHENTICATION_KEY_LENGTH`



<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_AUTHENTICATION_KEY_LENGTH">AUTHENTICATION_KEY_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_Authenticator_ED25519_SCHEME_ID"></a>

## Const `ED25519_SCHEME_ID`



<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_ED25519_SCHEME_ID">ED25519_SCHEME_ID</a>: u8 = 0;
</code></pre>



<a name="0x1_Authenticator_MULTI_ED25519_SCHEME_ID"></a>

## Const `MULTI_ED25519_SCHEME_ID`



<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a>: u8 = 1;
</code></pre>



<a name="0x1_Authenticator_MAX_MULTI_ED25519_KEYS"></a>

## Const `MAX_MULTI_ED25519_KEYS`

Maximum number of keys allowed in a MultiEd25519 public/private key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a>: u64 = 32;
</code></pre>



<a name="0x1_Authenticator_EWRONG_AUTHENTICATION_KEY_LENGTH"></a>

## Const `EWRONG_AUTHENTICATION_KEY_LENGTH`



<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_EWRONG_AUTHENTICATION_KEY_LENGTH">EWRONG_AUTHENTICATION_KEY_LENGTH</a>: u64 = 101;
</code></pre>



<a name="0x1_Authenticator_EZERO_THRESHOLD"></a>

## Const `EZERO_THRESHOLD`

Threshold provided was 0 which can't be used to create a <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a>: u64 = 102;
</code></pre>



<a name="0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD"></a>

## Const `ENOT_ENOUGH_KEYS_FOR_THRESHOLD`

Not enough keys were provided for the specified threshold when creating an <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a>: u64 = 103;
</code></pre>



<a name="0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD"></a>

## Const `ENUM_KEYS_ABOVE_MAX_THRESHOLD`

Too many keys were provided for the specified threshold when creating an <code>MultiEd25519</code> key


<pre><code><b>const</b> <a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a>: u64 = 104;
</code></pre>



<a name="0x1_Authenticator_create_multi_ed25519"></a>

## Function `create_multi_ed25519`

Create a a multisig policy from a vector of ed25519 public keys and a threshold.
Note: this does *not* check uniqueness of keys. Repeated keys are convenient to
encode weighted multisig policies. For example Alice AND 1 of Bob or Carol is
public_key: {alice_key, alice_key, bob_key, carol_key}, threshold: 3
Aborts if threshold is zero or bigger than the length of <code>public_keys</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(public_keys: vector&lt;vector&lt;u8&gt;&gt;, threshold: u8): <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(
    public_keys: vector&lt;vector&lt;u8&gt;&gt;,
    threshold: u8
): <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> {
    // check threshold requirements
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&public_keys);
    <b>assert</b>(threshold != 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_EZERO_THRESHOLD">EZERO_THRESHOLD</a>));
    <b>assert</b>(
        (threshold <b>as</b> u64) &lt;= len,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_ENOT_ENOUGH_KEYS_FOR_THRESHOLD">ENOT_ENOUGH_KEYS_FOR_THRESHOLD</a>)
    );
    // the multied25519 signature scheme allows at most 32 keys
    <b>assert</b>(
        len &lt;= <a href="Authenticator.md#0x1_Authenticator_MAX_MULTI_ED25519_KEYS">MAX_MULTI_ED25519_KEYS</a>,
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_ENUM_KEYS_ABOVE_MAX_THRESHOLD">ENUM_KEYS_ABOVE_MAX_THRESHOLD</a>)
    );

    <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a> { public_keys, threshold }
}
</code></pre>



</details>

<a name="0x1_Authenticator_ed25519_authentication_key"></a>

## Function `ed25519_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt; {
    // TODO: add constant <a href="Authenticator.md#0x1_Authenticator_ED25519_SCHEME_ID">ED25519_SCHEME_ID</a> = 0u8
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> public_key, <a href="Authenticator.md#0x1_Authenticator_ED25519_SCHEME_ID">ED25519_SCHEME_ID</a>);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(public_key)
}
</code></pre>



</details>

<a name="0x1_Authenticator_derived_address"></a>

## Function `derived_address`



<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_derived_address">derived_address</a>(authentication_key: vector&lt;u8&gt;): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_derived_address">derived_address</a>(authentication_key: vector&lt;u8&gt;): address {
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&authentication_key) == <a href="Authenticator.md#0x1_Authenticator_AUTHENTICATION_KEY_LENGTH">AUTHENTICATION_KEY_LENGTH</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Authenticator.md#0x1_Authenticator_EWRONG_AUTHENTICATION_KEY_LENGTH">EWRONG_AUTHENTICATION_KEY_LENGTH</a>));
    <b>let</b> address_bytes = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;u8&gt;();

    <b>let</b> i = 16;
    <b>while</b> (i &lt; 32) {
        <b>let</b> b = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&authentication_key, i);
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> address_bytes, b);
        i = i + 1;
    };

    <a href="LCS.md#0x1_LCS_to_address">LCS::to_address</a>(address_bytes)
}
</code></pre>



</details>

<a name="0x1_Authenticator_multi_ed25519_authentication_key"></a>

## Function `multi_ed25519_authentication_key`



<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): vector&lt;u8&gt; {
    <b>let</b> public_keys = &k.public_keys;
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(public_keys);
    <b>let</b> authentication_key_preimage = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> public_key = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(public_keys, i);
        <a href="Vector.md#0x1_Vector_append">Vector::append</a>(
            &<b>mut</b> authentication_key_preimage,
            public_key
        );
        i = i + 1;
    };
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> authentication_key_preimage, <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&k.threshold));
    // TODO: add constant <a href="Authenticator.md#0x1_Authenticator_MULTI_ED25519_SCHEME_ID">MULTI_ED25519_SCHEME_ID</a> = 1u8
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> authentication_key_preimage, 1u8);
    <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(authentication_key_preimage)
}
</code></pre>



</details>

<a name="0x1_Authenticator_public_keys"></a>

## Function `public_keys`



<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt; {
    &k.public_keys
}
</code></pre>



</details>

<a name="0x1_Authenticator_threshold"></a>

## Function `threshold`



<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_threshold">threshold</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_threshold">threshold</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">MultiEd25519PublicKey</a>): u8 {
    *&k.threshold
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>


We use an uninterpreted function to represent the result of key construction. The actual value
does not matter for the verification of callers.


<a name="0x1_Authenticator_spec_ed25519_authentication_key"></a>


<pre><code><b>define</b> <a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">spec_ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;;
</code></pre>


We use an uninterpreted function to represent the result of derived address. The actual value
does not matter for the verification of callers.


<a name="0x1_Authenticator_spec_derived_address"></a>


<pre><code><b>define</b> <a href="Authenticator.md#0x1_Authenticator_spec_derived_address">spec_derived_address</a>(authentication_key: vector&lt;u8&gt;): address;
</code></pre>



<a name="@Specification_0_create_multi_ed25519"></a>

### Function `create_multi_ed25519`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_create_multi_ed25519">create_multi_ed25519</a>(public_keys: vector&lt;vector&lt;u8&gt;&gt;, threshold: u8): <a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>
</code></pre>




<pre><code><b>aborts_if</b> threshold == 0;
<b>aborts_if</b> threshold &gt; <a href="Vector.md#0x1_Vector_length">Vector::length</a>(public_keys);
<b>aborts_if</b> <a href="Vector.md#0x1_Vector_length">Vector::length</a>(public_keys) &gt; 32;
</code></pre>



<a name="@Specification_0_ed25519_authentication_key"></a>

### Function `ed25519_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">ed25519_authentication_key</a>(public_key: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">spec_ed25519_authentication_key</a>(public_key);
</code></pre>



<a name="@Specification_0_derived_address"></a>

### Function `derived_address`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_derived_address">derived_address</a>(authentication_key: vector&lt;u8&gt;): address
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> len(authentication_key) != 32;
<b>ensures</b> [abstract] result == <a href="Authenticator.md#0x1_Authenticator_spec_derived_address">spec_derived_address</a>(authentication_key);
</code></pre>



<a name="@Specification_0_multi_ed25519_authentication_key"></a>

### Function `multi_ed25519_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_multi_ed25519_authentication_key">multi_ed25519_authentication_key</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_public_keys"></a>

### Function `public_keys`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_public_keys">public_keys</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): &vector&lt;vector&lt;u8&gt;&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_threshold"></a>

### Function `threshold`


<pre><code><b>public</b> <b>fun</b> <a href="Authenticator.md#0x1_Authenticator_threshold">threshold</a>(k: &<a href="Authenticator.md#0x1_Authenticator_MultiEd25519PublicKey">Authenticator::MultiEd25519PublicKey</a>): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
