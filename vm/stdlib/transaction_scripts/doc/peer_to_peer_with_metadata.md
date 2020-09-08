
<a name="SCRIPT"></a>

# Script `peer_to_peer_with_metadata.move`

### Table of Contents

-  [Function `peer_to_peer_with_metadata`](#SCRIPT_peer_to_peer_with_metadata)



<a name="SCRIPT_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(account: &signer, payee_public_key: vector&lt;u8&gt;, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType&gt;(
    account: &signer,
    payee_public_key: vector&lt;u8&gt;,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) {
    <b>let</b> payee = <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(<b>copy</b> payee_public_key);
    <a href="../../modules/doc/Account.md#0x1_Account_pay_from_with_metadata">Account::pay_from_with_metadata</a>&lt;TokenType&gt;(account,payee, amount, metadata)
}
</code></pre>



</details>
