
<a name="peer_to_peer"></a>

# Script `peer_to_peer`





<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="peer_to_peer.md#peer_to_peer">peer_to_peer</a>&lt;TokenType&gt;(account: &signer, payee: address, payee_auth_key: vector&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="peer_to_peer.md#peer_to_peer">peer_to_peer</a>&lt;TokenType&gt;(account: &signer, payee: address, payee_auth_key: vector&lt;u8&gt;, amount: u128) {
  <b>if</b> (!<a href="../../modules/doc/Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(payee, payee_auth_key);
  <a href="../../modules/doc/Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(account, payee, amount)
}
</code></pre>



</details>
