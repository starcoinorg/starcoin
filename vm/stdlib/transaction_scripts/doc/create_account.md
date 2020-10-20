
<a name="create_account"></a>

# Script `create_account`





<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="create_account.md#create_account">create_account</a>&lt;TokenType&gt;(account: &signer, fresh_address: address, public_key_vec: vector&lt;u8&gt;, initial_amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="create_account.md#create_account">create_account</a>&lt;TokenType&gt;(account: &signer, fresh_address: address, public_key_vec: vector&lt;u8&gt;, initial_amount: u128) {
    <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(fresh_address, public_key_vec);
  <b>if</b> (initial_amount &gt; 0) {
    <a href="../../modules/doc/Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(account, fresh_address, initial_amount);
  };
}
</code></pre>



</details>
