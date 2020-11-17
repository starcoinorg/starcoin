
<a name="mint_token_by_fixed_key"></a>

# Script `mint_token_by_fixed_key`



-  [Specification](#@Specification_0)
    -  [Function `mint_token_by_fixed_key`](#@Specification_0_mint_token_by_fixed_key)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Box.md#0x1_Box">0x1::Box</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="mint_token_by_fixed_key.md#mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="mint_token_by_fixed_key.md#mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
    signer: &signer,
) {
    // 1. take key: FixedTimeMintKey&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="../../modules/doc/Box.md#0x1_Box_take">Box::take</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer);

    // 2. mint token
    <b>let</b> tokens = <a href="../../modules/doc/Token.md#0x1_Token_mint_with_fixed_key">Token::mint_with_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(mint_key);

    // 3. deposit
    <a href="../../modules/doc/Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(signer, tokens);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_mint_token_by_fixed_key"></a>

### Function `mint_token_by_fixed_key`


<pre><code><b>public</b> <b>fun</b> <a href="mint_token_by_fixed_key.md#mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
