
<a name="0x1_MintScripts"></a>

# Module `0x1::MintScripts`



-  [Function `mint_and_split_by_linear_key`](#0x1_MintScripts_mint_and_split_by_linear_key)
-  [Function `mint_token_by_fixed_key`](#0x1_MintScripts_mint_token_by_fixed_key)
-  [Function `mint_token_by_linear_key`](#0x1_MintScripts_mint_token_by_linear_key)
-  [Function `split_fixed_key`](#0x1_MintScripts_split_fixed_key)
-  [Specification](#@Specification_0)
    -  [Function `mint_and_split_by_linear_key`](#@Specification_0_mint_and_split_by_linear_key)
    -  [Function `mint_token_by_fixed_key`](#@Specification_0_mint_token_by_fixed_key)
    -  [Function `mint_token_by_linear_key`](#@Specification_0_mint_token_by_linear_key)
    -  [Function `split_fixed_key`](#@Specification_0_split_fixed_key)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Collection.md#0x1_Collection">0x1::Collection</a>;
<b>use</b> <a href="Offer.md#0x1_Offer">0x1::Offer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_MintScripts_mint_and_split_by_linear_key"></a>

## Function `mint_and_split_by_linear_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    signer: signer,
    for_address: address,
    amount: u128,
    lock_period: u64,
) {
    // 1. take key: LinearTimeMintKey&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_take">Collection::take</a>&lt;<a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer);

    // 2. mint token
    <b>let</b> (tokens, new_mint_key) = <a href="Token.md#0x1_Token_split_linear_key">Token::split_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key, amount);

    // 3. deposit
    <a href="Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(&signer, tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="Token.md#0x1_Token_is_empty_key">Token::is_empty_key</a>(&mint_key)) {
        <a href="Token.md#0x1_Token_destroy_empty_key">Token::destroy_empty_key</a>(mint_key);
    } <b>else</b> {
        <a href="Collection.md#0x1_Collection_put">Collection::put</a>(&signer, mint_key);
    };

    // 5. offer
    <a href="Offer.md#0x1_Offer_create">Offer::create</a>&lt;<a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer, new_mint_key, for_address, lock_period);
}
</code></pre>



</details>

<a name="0x1_MintScripts_mint_token_by_fixed_key"></a>

## Function `mint_token_by_fixed_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    signer: signer,
) {
    // 1. take key: FixedTimeMintKey&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_take">Collection::take</a>&lt;<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer);

    // 2. mint token
    <b>let</b> tokens = <a href="Token.md#0x1_Token_mint_with_fixed_key">Token::mint_with_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(mint_key);

    // 3. deposit
    <a href="Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(&signer, tokens);
}
</code></pre>



</details>

<a name="0x1_MintScripts_mint_token_by_linear_key"></a>

## Function `mint_token_by_linear_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    signer: signer,
) {
    // 1. take key: LinearTimeMintKey&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_take">Collection::take</a>&lt;<a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer);

    // 2. mint token
    <b>let</b> tokens = <a href="Token.md#0x1_Token_mint_with_linear_key">Token::mint_with_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key);

    // 3. deposit
    <a href="Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(&signer, tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="Token.md#0x1_Token_is_empty_key">Token::is_empty_key</a>(&mint_key)) {
        <a href="Token.md#0x1_Token_destroy_empty_key">Token::destroy_empty_key</a>(mint_key);
    } <b>else</b> {
        <a href="Collection.md#0x1_Collection_put">Collection::put</a>(&signer, mint_key);
    }
}
</code></pre>



</details>

<a name="0x1_MintScripts_split_fixed_key"></a>

## Function `split_fixed_key`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>: store&gt;(
    signer: signer,
    for_address: address,
    amount: u128,
    lock_period: u64,
) {
    // 1. take key: FixedTimeMintKey&lt;<a href="Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="Collection.md#0x1_Collection_take">Collection::take</a>&lt;<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer);

    // 2.
    <b>let</b> new_mint_key = <a href="Token.md#0x1_Token_split_fixed_key">Token::split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key, amount);

    // 3. put key
    <a href="Collection.md#0x1_Collection_put">Collection::put</a>(&signer, mint_key);

    // 4. offer
    <a href="Offer.md#0x1_Offer_create">Offer::create</a>&lt;<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;&gt;(&signer, new_mint_key, for_address, lock_period);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_mint_and_split_by_linear_key"></a>

### Function `mint_and_split_by_linear_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_and_split_by_linear_key">mint_and_split_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_mint_token_by_fixed_key"></a>

### Function `mint_token_by_fixed_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_fixed_key">mint_token_by_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_mint_token_by_linear_key"></a>

### Function `mint_token_by_linear_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_mint_token_by_linear_key">mint_token_by_linear_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_split_fixed_key"></a>

### Function `split_fixed_key`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="MintScripts.md#0x1_MintScripts_split_fixed_key">split_fixed_key</a>&lt;<a href="Token.md#0x1_Token">Token</a>&gt;(signer: signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
