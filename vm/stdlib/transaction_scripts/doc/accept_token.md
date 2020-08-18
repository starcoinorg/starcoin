
<a name="SCRIPT"></a>

# Script `accept_token.move`

### Table of Contents

-  [Function `accept_token`](#SCRIPT_accept_token)



<a name="SCRIPT_accept_token"></a>

## Function `accept_token`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_accept_token">accept_token</a>&lt;TokenType&gt;(account: &signer) {
    <a href="../../modules/doc/Account.md#0x1_Account_accept_token">Account::accept_token</a>&lt;TokenType&gt;(account);
}
</code></pre>



</details>
