
<a name="0x1_DummyTokenScripts"></a>

# Module `0x1::DummyTokenScripts`



-  [Function `mint`](#0x1_DummyTokenScripts_mint)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="DummyToken.md#0x1_DummyToken">0x1::DummyToken</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_DummyTokenScripts_mint"></a>

## Function `mint`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="DummyToken.md#0x1_DummyTokenScripts_mint">mint</a>(sender: signer, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="DummyToken.md#0x1_DummyTokenScripts_mint">mint</a>(sender: signer, amount: u128){
    <b>let</b> token = <a href="DummyToken.md#0x1_DummyToken_mint">DummyToken::mint</a>(&sender, amount);
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&sender);
    <b>if</b>(<a href="Account.md#0x1_Account_is_accept_token">Account::is_accept_token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;(sender_addr)){
        <a href="Account.md#0x1_Account_do_accept_token">Account::do_accept_token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;(&sender);
    };
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(sender_addr, token);
}
</code></pre>



</details>
