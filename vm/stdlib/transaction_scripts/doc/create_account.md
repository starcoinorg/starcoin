
<a name="create_account"></a>

# Script `create_account`



-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="create_account_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="create_account.md#create_account_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 101;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="create_account.md#create_account">create_account</a>&lt;TokenType&gt;(account: &signer, fresh_address: address, auth_key: vector&lt;u8&gt;, initial_amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="create_account.md#create_account">create_account</a>&lt;TokenType&gt;(account: &signer, fresh_address: address, auth_key: vector&lt;u8&gt;, initial_amount: u128) {
  <b>let</b> created_address = <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(auth_key);
  <b>assert</b>(fresh_address == created_address, <a href="../../modules/doc/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="create_account.md#create_account_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>));
  <b>if</b> (initial_amount &gt; 0) {
    <a href="../../modules/doc/Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(account, fresh_address, initial_amount);
  };
}
</code></pre>



</details>
