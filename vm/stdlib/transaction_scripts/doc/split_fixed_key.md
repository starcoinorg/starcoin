
<a name="split_fixed_key"></a>

# Script `split_fixed_key`



-  [Specification](#@Specification_0)
    -  [Function `split_fixed_key`](#@Specification_0_split_fixed_key)


<pre><code><b>use</b> <a href="../../modules/doc/Box.md#0x1_Box">0x1::Box</a>;
<b>use</b> <a href="../../modules/doc/Offer.md#0x1_Offer">0x1::Offer</a>;
<b>use</b> <a href="../../modules/doc/Token.md#0x1_Token">0x1::Token</a>;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="split_fixed_key.md#split_fixed_key">split_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="split_fixed_key.md#split_fixed_key">split_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(
    signer: &signer,
    for_address: address,
    amount: u128,
    lock_period: u64,
) {
    // 1. take key: FixedTimeMintKey&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;
    <b>let</b> mint_key = <a href="../../modules/doc/Box.md#0x1_Box_take">Box::take</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer);

    // 2.
    <b>let</b> new_mint_key = <a href="../../modules/doc/Token.md#0x1_Token_split_fixed_key">Token::split_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(&<b>mut</b> mint_key, amount);

    // 3. put key
    <a href="../../modules/doc/Box.md#0x1_Box_put">Box::put</a>(signer, mint_key);

    // 4. offer
    <a href="../../modules/doc/Offer.md#0x1_Offer_create">Offer::create</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;&gt;(signer, new_mint_key, for_address, lock_period);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_split_fixed_key"></a>

### Function `split_fixed_key`


<pre><code><b>public</b> <b>fun</b> <a href="split_fixed_key.md#split_fixed_key">split_fixed_key</a>&lt;<a href="../../modules/doc/Token.md#0x1_Token">Token</a>&gt;(signer: &signer, for_address: address, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
