
<a name="peer_to_peer_with_metadata"></a>

# Script `peer_to_peer_with_metadata`



-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="peer_to_peer_with_metadata_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 101;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, payee_auth_key: vector&lt;u8&gt;, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    payee_auth_key: vector&lt;u8&gt;,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) {
    <b>if</b> (!<a href="../../modules/doc/Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
        <b>let</b> created_address = <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(payee_auth_key);
        <b>assert</b>(payee == created_address, <a href="../../modules/doc/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>));
    };
    <a href="../../modules/doc/Account.md#0x1_Account_pay_from_with_metadata">Account::pay_from_with_metadata</a>&lt;TokenType&gt;(account,payee, amount, metadata)
}
</code></pre>



</details>
