
<a name="mint_token_by_linear_key"></a>

# Script `mint_token_by_linear_key`





<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Box.md#0x1_Box">0x1::Box</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="mint_token_by_linear_key.md#mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="mint_token_by_linear_key.md#mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
    signer: &signer,
) {
    // 1. take key: LinearTimeMintKey&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="../../modules/doc/Box.md#0x1_Box_take">Box::take</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer);

    // 2. mint token
    <b>let</b> tokens = <a href="../../modules/doc/Token.md#0x1_Token_mint_with_linear_key">Token::mint_with_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key);

    // 3. mint_to account
    <a href="../../modules/doc/Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(signer, tokens);

    // 4. put key
    <a href="../../modules/doc/Box.md#0x1_Box_put">Box::put</a>(signer, mint_key);
}
</code></pre>



</details>
