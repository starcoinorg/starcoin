
<a name="0x1_SharedEd25519PublicKey"></a>

# Module `0x1::SharedEd25519PublicKey`

Each address that holds a <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource can rotate the public key stored in
this resource, but the account's authentication key will be updated in lockstep. This ensures
that the two keys always stay in sync.


-  [Resource `SharedEd25519PublicKey`](#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey)
-  [Constants](#@Constants_0)
-  [Function `publish`](#0x1_SharedEd25519PublicKey_publish)
-  [Function `rotate_key_`](#0x1_SharedEd25519PublicKey_rotate_key_)
-  [Function `rotate_key`](#0x1_SharedEd25519PublicKey_rotate_key)
-  [Function `key`](#0x1_SharedEd25519PublicKey_key)
-  [Function `exists_at`](#0x1_SharedEd25519PublicKey_exists_at)
-  [Specification](#@Specification_1)
    -  [Function `publish`](#@Specification_1_publish)
    -  [Function `rotate_key_`](#@Specification_1_rotate_key_)
    -  [Function `rotate_key`](#@Specification_1_rotate_key)
    -  [Function `key`](#@Specification_1_key)
    -  [Function `exists_at`](#@Specification_1_exists_at)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Authenticator.md#0x1_Authenticator">0x1::Authenticator</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signature.md#0x1_Signature">0x1::Signature</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_SharedEd25519PublicKey_SharedEd25519PublicKey"></a>

## Resource `SharedEd25519PublicKey`

A resource that forces the account associated with <code>rotation_cap</code> to use a ed25519
authentication key derived from <code>key</code>


<pre><code><b>struct</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>key: vector&lt;u8&gt;</code>
</dt>
<dd>
 32 byte ed25519 public key
</dd>
<dt>
<code>rotation_cap: <a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a></code>
</dt>
<dd>
 rotation capability for an account whose authentication key is always derived from <code>key</code>
</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY"></a>



<pre><code><b>const</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">EMALFORMED_PUBLIC_KEY</a>: u64 = 101;
</code></pre>



<a name="0x1_SharedEd25519PublicKey_publish"></a>

## Function `publish`

(1) Rotate the authentication key of the sender to <code>key</code>
(2) Publish a resource containing a 32-byte ed25519 public key and the rotation capability
of the sender under the <code>account</code>'s address.
Aborts if the sender already has a <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.
Aborts if the length of <code>new_public_key</code> is not 32.


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">publish</a>(account: &signer, key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">publish</a>(account: &signer, key: vector&lt;u8&gt;) {
    <b>let</b> t = <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
        key: x"",
        rotation_cap: <a href="Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(account)
    };
    <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(&<b>mut</b> t, key);
    <b>move_to</b>(account, t);
}
</code></pre>



</details>

<a name="0x1_SharedEd25519PublicKey_rotate_key_"></a>

## Function `rotate_key_`



<pre><code><b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(shared_key: &<b>mut</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a>, new_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(shared_key: &<b>mut</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>, new_public_key: vector&lt;u8&gt;) {
    // Cryptographic check of <b>public</b> key validity
    <b>assert</b>!(
        <a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(<b>copy</b> new_public_key),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_EMALFORMED_PUBLIC_KEY">EMALFORMED_PUBLIC_KEY</a>)
    );
    <a href="Account.md#0x1_Account_rotate_authentication_key_with_capability">Account::rotate_authentication_key_with_capability</a>(
        &shared_key.rotation_cap,
        <a href="Authenticator.md#0x1_Authenticator_ed25519_authentication_key">Authenticator::ed25519_authentication_key</a>(<b>copy</b> new_public_key)
    );
    shared_key.key = new_public_key;
}
</code></pre>



</details>

<a name="0x1_SharedEd25519PublicKey_rotate_key"></a>

## Function `rotate_key`

(1) rotate the public key stored <code>account</code>'s <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource to
<code>new_public_key</code>
(2) rotate the authentication key using the capability stored in the <code>account</code>'s
<code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> to a new value derived from <code>new_public_key</code>
Aborts if the sender does not have a <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.
Aborts if the length of <code>new_public_key</code> is not 32.


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">rotate_key</a>(account: &signer, new_public_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">rotate_key</a>(account: &signer, new_public_key: vector&lt;u8&gt;) <b>acquires</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
    <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(<b>borrow_global_mut</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)), new_public_key);
}
</code></pre>



</details>

<a name="0x1_SharedEd25519PublicKey_key"></a>

## Function `key`

Return the public key stored under <code>addr</code>.
Aborts if <code>addr</code> does not hold a <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_key">key</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_key">key</a>(addr: <b>address</b>): vector&lt;u8&gt; <b>acquires</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a> {
    *&<b>borrow_global</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(addr).key
}
</code></pre>



</details>

<a name="0x1_SharedEd25519PublicKey_exists_at"></a>

## Function `exists_at`

Returns true if <code>addr</code> holds a <code><a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a></code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_exists_at">exists_at</a>(addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(addr)
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_publish"></a>

### Function `publish`


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_publish">publish</a>(account: &signer, key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> StarcoinFramework::Option::is_none(<b>global</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).key_rotation_capability);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(
          StarcoinFramework::Option::borrow&lt;<a href="Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a>&gt;(
              <b>global</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account))
              .key_rotation_capability
          ).account_address);
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(key);
<b>aborts_if</b> <b>exists</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> len(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(key)) != 32;
</code></pre>



<a name="@Specification_1_rotate_key_"></a>

### Function `rotate_key_`


<pre><code><b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key_">rotate_key_</a>(shared_key: &<b>mut</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_SharedEd25519PublicKey">SharedEd25519PublicKey::SharedEd25519PublicKey</a>, new_public_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(shared_key.rotation_cap.account_address);
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(new_public_key);
<b>aborts_if</b> len(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(new_public_key)) != 32;
</code></pre>



<a name="@Specification_1_rotate_key"></a>

### Function `rotate_key`


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_rotate_key">rotate_key</a>(account: &signer, new_public_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(<b>global</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).rotation_cap.account_address);
<b>aborts_if</b> !<a href="Signature.md#0x1_Signature_ed25519_validate_pubkey">Signature::ed25519_validate_pubkey</a>(new_public_key);
<b>aborts_if</b> len(<a href="Authenticator.md#0x1_Authenticator_spec_ed25519_authentication_key">Authenticator::spec_ed25519_authentication_key</a>(new_public_key)) != 32;
</code></pre>



<a name="@Specification_1_key"></a>

### Function `key`


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_key">key</a>(addr: <b>address</b>): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey">SharedEd25519PublicKey</a>&gt;(addr);
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="SharedEd25519PublicKey.md#0x1_SharedEd25519PublicKey_exists_at">exists_at</a>(addr: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>
