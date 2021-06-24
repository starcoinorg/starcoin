
<a name="0x1_DummyToken"></a>

# Module `0x1::DummyToken`

The module provide a dummy token implementation.


-  [Struct `DummyToken`](#0x1_DummyToken_DummyToken)
-  [Resource `SharedBurnCapability`](#0x1_DummyToken_SharedBurnCapability)
-  [Resource `SharedMintCapability`](#0x1_DummyToken_SharedMintCapability)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_DummyToken_initialize)
-  [Function `is_dummy_token`](#0x1_DummyToken_is_dummy_token)
-  [Function `burn`](#0x1_DummyToken_burn)
-  [Function `mint`](#0x1_DummyToken_mint)
-  [Function `token_address`](#0x1_DummyToken_token_address)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_DummyToken_DummyToken"></a>

## Struct `DummyToken`

The DummyToken type.


<pre><code><b>struct</b> <a href="DummyToken.md#0x1_DummyToken">DummyToken</a> has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DummyToken_SharedBurnCapability"></a>

## Resource `SharedBurnCapability`

Burn capability of the token.


<pre><code><b>struct</b> <a href="DummyToken.md#0x1_DummyToken_SharedBurnCapability">SharedBurnCapability</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;<a href="DummyToken.md#0x1_DummyToken_DummyToken">DummyToken::DummyToken</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_DummyToken_SharedMintCapability"></a>

## Resource `SharedMintCapability`

Mint capability of the token.


<pre><code><b>struct</b> <a href="DummyToken.md#0x1_DummyToken_SharedMintCapability">SharedMintCapability</a> has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;<a href="DummyToken.md#0x1_DummyToken_DummyToken">DummyToken::DummyToken</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_DummyToken_PRECISION"></a>



<pre><code><b>const</b> <a href="DummyToken.md#0x1_DummyToken_PRECISION">PRECISION</a>: u8 = 3;
</code></pre>



<a name="0x1_DummyToken_EMINT_TOO_MUCH"></a>



<pre><code><b>const</b> <a href="DummyToken.md#0x1_DummyToken_EMINT_TOO_MUCH">EMINT_TOO_MUCH</a>: u64 = 101;
</code></pre>



<a name="0x1_DummyToken_initialize"></a>

## Function `initialize`

Initialization of the module.


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_initialize">initialize</a>(account: &signer) {
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;(
        account,
        <a href="DummyToken.md#0x1_DummyToken_PRECISION">PRECISION</a>,
    );

    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;(account);
    move_to(account, <a href="DummyToken.md#0x1_DummyToken_SharedBurnCapability">SharedBurnCapability</a>{cap: burn_cap});

    <b>let</b> burn_cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;(account);
    move_to(account, <a href="DummyToken.md#0x1_DummyToken_SharedMintCapability">SharedMintCapability</a>{cap: burn_cap});
}
</code></pre>



</details>

<a name="0x1_DummyToken_is_dummy_token"></a>

## Function `is_dummy_token`

Returns true if <code>TokenType</code> is <code><a href="DummyToken.md#0x1_DummyToken_DummyToken">DummyToken::DummyToken</a></code>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_is_dummy_token">is_dummy_token</a>&lt;TokenType: store&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_is_dummy_token">is_dummy_token</a>&lt;TokenType: store&gt;(): bool {
    <a href="Token.md#0x1_Token_is_same_token">Token::is_same_token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>, TokenType&gt;()
}
</code></pre>



</details>

<a name="0x1_DummyToken_burn"></a>

## Function `burn`

Burn the given token.


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_burn">burn</a>(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="DummyToken.md#0x1_DummyToken_DummyToken">DummyToken::DummyToken</a>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_burn">burn</a>(token: <a href="Token.md#0x1_Token">Token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;) <b>acquires</b> <a href="DummyToken.md#0x1_DummyToken_SharedBurnCapability">SharedBurnCapability</a>{
    <b>let</b> cap = borrow_global&lt;<a href="DummyToken.md#0x1_DummyToken_SharedBurnCapability">SharedBurnCapability</a>&gt;(<a href="DummyToken.md#0x1_DummyToken_token_address">token_address</a>());
    <a href="Token.md#0x1_Token_burn_with_capability">Token::burn_with_capability</a>(&cap.cap, token);
}
</code></pre>



</details>

<a name="0x1_DummyToken_mint"></a>

## Function `mint`

Anyone can mint DummyToken, amount should < 10000


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_mint">mint</a>(_account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="DummyToken.md#0x1_DummyToken_DummyToken">DummyToken::DummyToken</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_mint">mint</a>(_account: &signer, amount: u128) : <a href="Token.md#0x1_Token">Token</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt; <b>acquires</b> <a href="DummyToken.md#0x1_DummyToken_SharedMintCapability">SharedMintCapability</a>{
    <b>assert</b>(amount &lt;= 10000, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="DummyToken.md#0x1_DummyToken_EMINT_TOO_MUCH">EMINT_TOO_MUCH</a>));
    <b>let</b> cap = borrow_global&lt;<a href="DummyToken.md#0x1_DummyToken_SharedMintCapability">SharedMintCapability</a>&gt;(<a href="DummyToken.md#0x1_DummyToken_token_address">token_address</a>());
    <a href="Token.md#0x1_Token_mint_with_capability">Token::mint_with_capability</a>(&cap.cap, amount)
}
</code></pre>



</details>

<a name="0x1_DummyToken_token_address"></a>

## Function `token_address`

Return the token address.


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_token_address">token_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="DummyToken.md#0x1_DummyToken_token_address">token_address</a>(): address {
    <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;<a href="DummyToken.md#0x1_DummyToken">DummyToken</a>&gt;()
}
</code></pre>



</details>
