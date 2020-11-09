
<a name="0x1_TokenSwap"></a>

# Module `0x1::TokenSwap`

Token Swap


-  [Resource `LiquidityTokenCapability`](#0x1_TokenSwap_LiquidityTokenCapability)
-  [Resource `TokenPair`](#0x1_TokenSwap_TokenPair)
-  [Constants](#@Constants_0)
-  [Function `register_swap_pair`](#0x1_TokenSwap_register_swap_pair)
-  [Function `register_liquidity_token`](#0x1_TokenSwap_register_liquidity_token)
-  [Function `make_token_pair`](#0x1_TokenSwap_make_token_pair)
-  [Function `mint`](#0x1_TokenSwap_mint)
-  [Function `burn`](#0x1_TokenSwap_burn)
-  [Function `burn_liquidity`](#0x1_TokenSwap_burn_liquidity)
-  [Function `get_reserves`](#0x1_TokenSwap_get_reserves)
-  [Function `swap`](#0x1_TokenSwap_swap)
-  [Function `compare_token`](#0x1_TokenSwap_compare_token)
-  [Function `assert_admin`](#0x1_TokenSwap_assert_admin)
-  [Function `admin_address`](#0x1_TokenSwap_admin_address)


<pre><code><b>use</b> <a href="Compare.md#0x1_Compare">0x1::Compare</a>;
<b>use</b> <a href="TokenSwap.md#0x1_LiquidityToken">0x1::LiquidityToken</a>;
<b>use</b> <a href="Math.md#0x1_Math">0x1::Math</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_TokenSwap_LiquidityTokenCapability"></a>

## Resource `LiquidityTokenCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a>&lt;X, Y&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>mint: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken_LiquidityToken">LiquidityToken::LiquidityToken</a>&lt;X, Y&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>burn: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken_LiquidityToken">LiquidityToken::LiquidityToken</a>&lt;X, Y&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TokenSwap_TokenPair"></a>

## Resource `TokenPair`



<pre><code><b>resource</b> <b>struct</b> <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>token_x_reserve: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>token_y_reserve: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>last_block_timestamp: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>last_price_x_cumulative: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>last_price_y_cumulative: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>last_k: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_TokenSwap_DUPLICATE_TOKEN"></a>



<pre><code><b>const</b> <a href="TokenSwap.md#0x1_TokenSwap_DUPLICATE_TOKEN">DUPLICATE_TOKEN</a>: u64 = 4000;
</code></pre>



<a name="0x1_TokenSwap_INVALID_TOKEN_PAIR"></a>



<pre><code><b>const</b> <a href="TokenSwap.md#0x1_TokenSwap_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>: u64 = 4001;
</code></pre>



<a name="0x1_TokenSwap_register_swap_pair"></a>

## Function `register_swap_pair`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_register_swap_pair">register_swap_pair</a>&lt;X, Y&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_register_swap_pair">register_swap_pair</a>&lt;X, Y&gt;(signer: &signer) {
    <b>assert</b>(<a href="TokenSwap.md#0x1_TokenSwap_compare_token">compare_token</a>&lt;X, Y&gt;() == 1, <a href="TokenSwap.md#0x1_TokenSwap_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <a href="TokenSwap.md#0x1_TokenSwap_assert_admin">assert_admin</a>(signer);
    <b>let</b> token_pair = <a href="TokenSwap.md#0x1_TokenSwap_make_token_pair">make_token_pair</a>&lt;X, Y&gt;();
    move_to(signer, token_pair);
    <a href="TokenSwap.md#0x1_TokenSwap_register_liquidity_token">register_liquidity_token</a>&lt;X, Y&gt;(signer);
}
</code></pre>



</details>

<a name="0x1_TokenSwap_register_liquidity_token"></a>

## Function `register_liquidity_token`



<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_register_liquidity_token">register_liquidity_token</a>&lt;X, Y&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_register_liquidity_token">register_liquidity_token</a>&lt;X, Y&gt;(signer: &signer) {
    <a href="TokenSwap.md#0x1_TokenSwap_assert_admin">assert_admin</a>(signer);
    <a href="Token.md#0x1_Token_register_token">Token::register_token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(signer, 18);
    <b>let</b> mint_capability = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(signer);
    <b>let</b> burn_capability = <a href="Token.md#0x1_Token_remove_burn_capability">Token::remove_burn_capability</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(signer);
    move_to(signer, <a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a> { mint: mint_capability, burn: burn_capability });
}
</code></pre>



</details>

<a name="0x1_TokenSwap_make_token_pair"></a>

## Function `make_token_pair`



<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_make_token_pair">make_token_pair</a>&lt;X, Y&gt;(): <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenSwap::TokenPair</a>&lt;X, Y&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_make_token_pair">make_token_pair</a>&lt;X, Y&gt;(): <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt; {
    // TODO: <b>assert</b> X, Y is token
    <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt; {
        token_x_reserve: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;X&gt;(),
        token_y_reserve: <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;Y&gt;(),
        last_block_timestamp: 0,
        last_price_x_cumulative: 0,
        last_price_y_cumulative: 0,
        last_k: 0,
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwap_mint"></a>

## Function `mint`

Liquidity Provider's methods
type args, X, Y should be sorted.


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_mint">mint</a>&lt;X, Y&gt;(x: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, y: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken_LiquidityToken">LiquidityToken::LiquidityToken</a>&lt;X, Y&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_mint">mint</a>&lt;X, Y&gt;(
    x: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;,
    y: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;,
): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt; <b>acquires</b> <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>, <a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a> {
    <b>let</b> total_supply: u128 = <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;();
    <b>let</b> x_value = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;X&gt;(&x);
    <b>let</b> y_value = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;Y&gt;(&y);
    <b>let</b> liquidity = <b>if</b> (total_supply == 0) {
        // 1000 is the MINIMUM_LIQUIDITY
        (<a href="Math.md#0x1_Math_sqrt">Math::sqrt</a>((x_value <b>as</b> u128) * (y_value <b>as</b> u128)) <b>as</b> u128) - 1000
    } <b>else</b> {
        <b>let</b> token_pair = borrow_global&lt;<a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
        <b>let</b> x_reserve = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_x_reserve);
        <b>let</b> y_reserve = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_y_reserve);
        <b>let</b> x_liquidity = x_value * total_supply / x_reserve;
        <b>let</b> y_liquidity = y_value * total_supply / y_reserve;
        // <b>use</b> smaller one.
        <b>if</b> (x_liquidity &lt; y_liquidity) {
            x_liquidity
        } <b>else</b> {
            y_liquidity
        }
    };
    <b>assert</b>(liquidity &gt; 0, 100);
    <b>let</b> token_pair = borrow_global_mut&lt;<a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pair.token_x_reserve, x);
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pair.token_y_reserve, y);
    <b>let</b> liquidity_cap = borrow_global&lt;<a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <b>let</b> mint_token = <a href="Token.md#0x1_Token_mint_with_capability">Token::mint_with_capability</a>(&liquidity_cap.mint, liquidity);
    mint_token
}
</code></pre>



</details>

<a name="0x1_TokenSwap_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_burn">burn</a>&lt;X, Y&gt;(to_burn: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken_LiquidityToken">LiquidityToken::LiquidityToken</a>&lt;X, Y&gt;&gt;): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_burn">burn</a>&lt;X, Y&gt;(
    to_burn: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;,
): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;) <b>acquires</b> <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>, <a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a> {
    <b>let</b> to_burn_value = (<a href="Token.md#0x1_Token_value">Token::value</a>(&to_burn) <b>as</b> u128);
    <b>let</b> token_pair = borrow_global_mut&lt;<a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <b>let</b> x_reserve = (<a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_x_reserve) <b>as</b> u128);
    <b>let</b> y_reserve = (<a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_y_reserve) <b>as</b> u128);
    <b>let</b> total_supply = <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;();
    <b>let</b> x = to_burn_value * x_reserve / total_supply;
    <b>let</b> y = to_burn_value * y_reserve / total_supply;
    <b>assert</b>(x &gt; 0 && y &gt; 0, 101);
    <a href="TokenSwap.md#0x1_TokenSwap_burn_liquidity">burn_liquidity</a>(to_burn);
    <b>let</b> x_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pair.token_x_reserve, x);
    <b>let</b> y_token = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pair.token_y_reserve, y);
    (x_token, y_token)
}
</code></pre>



</details>

<a name="0x1_TokenSwap_burn_liquidity"></a>

## Function `burn_liquidity`



<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_burn_liquidity">burn_liquidity</a>&lt;X, Y&gt;(to_burn: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken_LiquidityToken">LiquidityToken::LiquidityToken</a>&lt;X, Y&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_burn_liquidity">burn_liquidity</a>&lt;X, Y&gt;(to_burn: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;)
<b>acquires</b> <a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a> {
    <b>let</b> liquidity_cap = borrow_global&lt;<a href="TokenSwap.md#0x1_TokenSwap_LiquidityTokenCapability">LiquidityTokenCapability</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <a href="Token.md#0x1_Token_burn_with_capability">Token::burn_with_capability</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(&liquidity_cap.burn, to_burn);
}
</code></pre>



</details>

<a name="0x1_TokenSwap_get_reserves"></a>

## Function `get_reserves`

Get reserves of a token pair.
The order of type args should be sorted.


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_get_reserves">get_reserves</a>&lt;X, Y&gt;(): (u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_get_reserves">get_reserves</a>&lt;X, Y&gt;(): (u128, u128) <b>acquires</b> <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a> {
    <b>let</b> token_pair = borrow_global&lt;<a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <b>let</b> x_reserve = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_x_reserve);
    <b>let</b> y_reserve = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_y_reserve);
    (x_reserve, y_reserve)
}
</code></pre>



</details>

<a name="0x1_TokenSwap_swap"></a>

## Function `swap`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_swap">swap</a>&lt;X, Y&gt;(x_in: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, y_out: u128, y_in: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;, x_out: u128): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_swap">swap</a>&lt;X, Y&gt;(
    x_in: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;,
    y_out: u128,
    y_in: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;,
    x_out: u128,
): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;X&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;Y&gt;) <b>acquires</b> <a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a> {
    <b>let</b> x_in_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&x_in);
    <b>let</b> y_in_value = <a href="Token.md#0x1_Token_value">Token::value</a>(&y_in);
    <b>assert</b>(x_in_value &gt; 0 || y_in_value &gt; 0, 400);
    <b>let</b> (x_reserve, y_reserve) = <a href="TokenSwap.md#0x1_TokenSwap_get_reserves">get_reserves</a>&lt;X, Y&gt;();
    <b>let</b> token_pair = borrow_global_mut&lt;<a href="TokenSwap.md#0x1_TokenSwap_TokenPair">TokenPair</a>&lt;X, Y&gt;&gt;(<a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>());
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pair.token_x_reserve, x_in);
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> token_pair.token_y_reserve, y_in);
    <b>let</b> x_swapped = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pair.token_x_reserve, x_out);
    <b>let</b> y_swapped = <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> token_pair.token_y_reserve, y_out);
    {
        <b>let</b> x_reserve_new = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_x_reserve);
        <b>let</b> y_reserve_new = <a href="Token.md#0x1_Token_value">Token::value</a>(&token_pair.token_y_reserve);
        <b>let</b> x_adjusted = x_reserve_new * 1000 - x_in_value * 3;
        <b>let</b> y_adjusted = y_reserve_new * 1000 - y_in_value * 3;
        <b>assert</b>(x_adjusted * y_adjusted &gt;= x_reserve * y_reserve * 1000000, 500);
    };
    (x_swapped, y_swapped)
}
</code></pre>



</details>

<a name="0x1_TokenSwap_compare_token"></a>

## Function `compare_token`

Caller should call this function to determine the order of A, B


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_compare_token">compare_token</a>&lt;A, B&gt;(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_compare_token">compare_token</a>&lt;A, B&gt;(): u8 {
    // <b>let</b> a_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&Token::token_id&lt;A&gt;());
    // <b>let</b> b_bytes = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&Token::token_id&lt;B&gt;());
    <b>let</b> a_bytes = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;A&gt;();
    <b>let</b> b_bytes = <a href="Token.md#0x1_Token_token_code">Token::token_code</a>&lt;B&gt;();
    <a href="Compare.md#0x1_Compare_cmp_lcs_bytes">Compare::cmp_lcs_bytes</a>(&a_bytes, &b_bytes)
}
</code></pre>



</details>

<a name="0x1_TokenSwap_assert_admin"></a>

## Function `assert_admin`



<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_assert_admin">assert_admin</a>(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_assert_admin">assert_admin</a>(signer: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == <a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>(), 401);
}
</code></pre>



</details>

<a name="0x1_TokenSwap_admin_address"></a>

## Function `admin_address`



<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwap.md#0x1_TokenSwap_admin_address">admin_address</a>(): address {
    0x1
}
</code></pre>



</details>
