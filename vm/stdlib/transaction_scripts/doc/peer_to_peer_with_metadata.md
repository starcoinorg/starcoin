
<a name="peer_to_peer_with_metadata"></a>

# Script `peer_to_peer_with_metadata`






<pre><code><b>public</b> <b>fun</b> <a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee: address, payee_public_key: vector&lt;u8&gt;, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="peer_to_peer_with_metadata.md#peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(
    account: &signer,
    payee: address,
    payee_public_key: vector&lt;u8&gt;,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) {
    <b>if</b> (!<a href="../../modules/doc/Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
        <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(payee, payee_public_key);
    };
    <a href="../../modules/doc/Account.md#0x1_Account_pay_from_with_metadata">Account::pay_from_with_metadata</a>&lt;TokenType&gt;(account,payee, amount, metadata)
}
</code></pre>



</details>
