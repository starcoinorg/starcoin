
<a name="SCRIPT"></a>

# Script `mint.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;TokenType&gt;(account: &signer, payee: address, auth_key_prefix: vector&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;TokenType&gt;(account: &signer, payee: address, auth_key_prefix: vector&lt;u8&gt;, amount: u128) {
  <b>if</b> (!<a href="../../modules/doc/Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(payee, auth_key_prefix);
  <a href="../../modules/doc/Account.md#0x1_Account_mint_to_address">Account::mint_to_address</a>&lt;TokenType&gt;(account, payee, amount);
}
</code></pre>



</details>
