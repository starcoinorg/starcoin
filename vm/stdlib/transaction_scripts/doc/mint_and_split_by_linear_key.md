
<a name="mint_and_split_by_linear_key"></a>

# Script `mint_and_split_by_linear_key`



-  [Specification](#@Specification_0)
    -  [Function `mint_and_split_by_linear_key`](#@Specification_0_mint_and_split_by_linear_key)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/Box.md#0x1_Box">0x1::Box</a>;
<b>use</b> <a href="../../modules/doc/Offer.md#0x1_Offer">0x1::Offer</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="mint_and_split_by_linear_key.md#mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="mint_and_split_by_linear_key.md#mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
    signer: &signer,
    for_address: address,
    amount: u128,
    lock_period: u64,
) {
    // 1. take key: LinearTimeMintKey&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="../../modules/doc/Box.md#0x1_Box_take">Box::take</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer);

    // 2. mint token
    <b>let</b> (tokens, new_mint_key) = <a href="../../modules/doc/Token.md#0x1_Token_split_linear_key">Token::split_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key, amount);

    // 3. deposit
    <a href="../../modules/doc/Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(signer, tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="../../modules/doc/Token.md#0x1_Token_is_empty_key">Token::is_empty_key</a>(&mint_key)) {
        <a href="../../modules/doc/Token.md#0x1_Token_destroy_empty_key">Token::destroy_empty_key</a>(mint_key);
    } <b>else</b> {
        <a href="../../modules/doc/Box.md#0x1_Box_put">Box::put</a>(signer, mint_key);
    };

    // 5. offer
    <a href="../../modules/doc/Offer.md#0x1_Offer_create">Offer::create</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer, new_mint_key, for_address, lock_period);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_mint_and_split_by_linear_key"></a>

### Function `mint_and_split_by_linear_key`


<pre><code><b>public</b> <b>fun</b> <a href="mint_and_split_by_linear_key.md#mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
