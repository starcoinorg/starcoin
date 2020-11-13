
<a name="rotate_authentication_key"></a>

# Script `rotate_authentication_key`



-  [Summary](#@Summary_0)
-  [Technical Description](#@Technical_Description_1)
-  [Parameters](#@Parameters_2)
-  [Common Abort Conditions](#@Common_Abort_Conditions_3)
-  [Specification](#@Specification_4)
    -  [Function `rotate_authentication_key`](#@Specification_4_rotate_authentication_key)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
</code></pre>



<a name="@Summary_0"></a>

## Summary

Rotates the transaction sender's authentication key to the supplied new authentication key. May
be sent by any account.


<a name="@Technical_Description_1"></a>

## Technical Description

Rotate the <code>account</code>'s <code><a href="../../modules/doc/Account.md#0x1_Account_Account">Account::Account</a></code> <code>authentication_key</code> field to <code>new_key</code>.
<code>new_key</code> must be a valid ed25519 public key, and <code>account</code> must not have previously delegated
its <code><a href="../../modules/doc/Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a></code>.


<a name="@Parameters_2"></a>

## Parameters

| Name      | Type         | Description                                                 |
| ------    | ------       | -------------                                               |
| <code>account</code> | <code>&signer</code>    | Signer reference of the sending account of the transaction. |
| <code>new_key</code> | <code>vector&lt;u8&gt;</code> | New ed25519 public key to be used for <code>account</code>.            |


<a name="@Common_Abort_Conditions_3"></a>

## Common Abort Conditions

| Error Category             | Error Reason                                               | Description                                                                              |
| ----------------           | --------------                                             | -------------                                                                            |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_STATE">Errors::INVALID_STATE</a></code>    | <code><a href="../../modules/doc/Account.md#0x1_Account_EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED">Account::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED</a></code> | <code>account</code> has already delegated/extracted its <code><a href="../../modules/doc/Account.md#0x1_Account_KeyRotationCapability">Account::KeyRotationCapability</a></code>.     |
| <code><a href="../../modules/doc/Errors.md#0x1_Errors_INVALID_ARGUMENT">Errors::INVALID_ARGUMENT</a></code> | <code><a href="../../modules/doc/Account.md#0x1_Account_EMALFORMED_AUTHENTICATION_KEY">Account::EMALFORMED_AUTHENTICATION_KEY</a></code>              | <code>new_key</code> was an invalid length.                                                         |


<pre><code><b>public</b> <b>fun</b> <a href="rotate_authentication_key.md#rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="rotate_authentication_key.md#rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;) {
    <b>let</b> key_rotation_capability = <a href="../../modules/doc/Account.md#0x1_Account_extract_key_rotation_capability">Account::extract_key_rotation_capability</a>(account);
    <a href="../../modules/doc/Account.md#0x1_Account_rotate_authentication_key">Account::rotate_authentication_key</a>(&key_rotation_capability, new_key);
    <a href="../../modules/doc/Account.md#0x1_Account_restore_key_rotation_capability">Account::restore_key_rotation_capability</a>(key_rotation_capability);
}
</code></pre>



</details>

<a name="@Specification_4"></a>

## Specification


<a name="@Specification_4_rotate_authentication_key"></a>

### Function `rotate_authentication_key`


<pre><code><b>public</b> <b>fun</b> <a href="rotate_authentication_key.md#rotate_authentication_key">rotate_authentication_key</a>(account: &signer, new_key: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
