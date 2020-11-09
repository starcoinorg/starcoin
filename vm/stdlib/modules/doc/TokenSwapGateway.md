
<a name="0x1_TokenSwapGateway"></a>

# Module `0x1::TokenSwapGateway`



-  [Constants](#@Constants_0)
-  [Function `liquidity`](#0x1_TokenSwapGateway_liquidity)
-  [Function `total_liquidity`](#0x1_TokenSwapGateway_total_liquidity)
-  [Function `add_liquidity`](#0x1_TokenSwapGateway_add_liquidity)
-  [Function `_add_liquidity`](#0x1_TokenSwapGateway__add_liquidity)
-  [Function `_calculate_amount_for_liquidity`](#0x1_TokenSwapGateway__calculate_amount_for_liquidity)
-  [Function `remove_liquidity`](#0x1_TokenSwapGateway_remove_liquidity)
-  [Function `_remove_liquidity`](#0x1_TokenSwapGateway__remove_liquidity)
-  [Function `swap_exact_token_for_token`](#0x1_TokenSwapGateway_swap_exact_token_for_token)
-  [Function `swap_token_for_exact_token`](#0x1_TokenSwapGateway_swap_token_for_exact_token)
-  [Function `get_reserves`](#0x1_TokenSwapGateway_get_reserves)
-  [Function `quote`](#0x1_TokenSwapGateway_quote)
-  [Function `get_amount_out`](#0x1_TokenSwapGateway_get_amount_out)
-  [Function `get_amount_in`](#0x1_TokenSwapGateway_get_amount_in)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="TokenSwap.md#0x1_LiquidityToken">0x1::LiquidityToken</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="TokenSwap.md#0x1_TokenSwap">0x1::TokenSwap</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_TokenSwapGateway_INVALID_TOKEN_PAIR"></a>



<pre><code><b>const</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>: u64 = 4001;
</code></pre>



<a name="0x1_TokenSwapGateway_INSUFFICIENT_X_AMOUNT"></a>



<pre><code><b>const</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INSUFFICIENT_X_AMOUNT">INSUFFICIENT_X_AMOUNT</a>: u64 = 1010;
</code></pre>



<a name="0x1_TokenSwapGateway_INSUFFICIENT_Y_AMOUNT"></a>



<pre><code><b>const</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INSUFFICIENT_Y_AMOUNT">INSUFFICIENT_Y_AMOUNT</a>: u64 = 1011;
</code></pre>



<a name="0x1_TokenSwapGateway_liquidity"></a>

## Function `liquidity`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_liquidity">liquidity</a>&lt;X, Y&gt;(account: address): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_liquidity">liquidity</a>&lt;X, Y&gt;(account: address): u128 {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <b>if</b> (order == 1) {
        <a href="Account.md#0x1_Account_balance">Account::balance</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(account)
    } <b>else</b> {
        <a href="Account.md#0x1_Account_balance">Account::balance</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;Y, X&gt;&gt;(account)
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_total_liquidity"></a>

## Function `total_liquidity`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_total_liquidity">total_liquidity</a>&lt;X, Y&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_total_liquidity">total_liquidity</a>&lt;X, Y&gt;(): u128 {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <b>if</b> (order == 1) {
        <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;()
    } <b>else</b> {
        <a href="Token.md#0x1_Token_market_cap">Token::market_cap</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;Y, X&gt;&gt;()
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_add_liquidity"></a>

## Function `add_liquidity`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_add_liquidity">add_liquidity</a>&lt;X, Y&gt;(signer: &signer, amount_x_desired: u128, amount_y_desired: u128, amount_x_min: u128, amount_y_min: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_add_liquidity">add_liquidity</a>&lt;X, Y&gt;(
    signer: &signer,
    amount_x_desired: u128,
    amount_y_desired: u128,
    amount_x_min: u128,
    amount_y_min: u128,
) {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <b>if</b> (order == 1) {
        <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__add_liquidity">_add_liquidity</a>&lt;X, Y&gt;(
            signer,
            amount_x_desired,
            amount_y_desired,
            amount_x_min,
            amount_y_min,
        );
    } <b>else</b> {
        <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__add_liquidity">_add_liquidity</a>&lt;Y, X&gt;(
            signer,
            amount_y_desired,
            amount_x_desired,
            amount_y_min,
            amount_x_min,
        );
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway__add_liquidity"></a>

## Function `_add_liquidity`



<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__add_liquidity">_add_liquidity</a>&lt;X, Y&gt;(signer: &signer, amount_x_desired: u128, amount_y_desired: u128, amount_x_min: u128, amount_y_min: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__add_liquidity">_add_liquidity</a>&lt;X, Y&gt;(
    signer: &signer,
    amount_x_desired: u128,
    amount_y_desired: u128,
    amount_x_min: u128,
    amount_y_min: u128,
) {
    <b>let</b> (amount_x, amount_y) = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__calculate_amount_for_liquidity">_calculate_amount_for_liquidity</a>&lt;X, Y&gt;(
        amount_x_desired,
        amount_y_desired,
        amount_x_min,
        amount_y_min,
    );
    <b>let</b> x_token = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;X&gt;(signer, amount_x);
    <b>let</b> y_token = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;Y&gt;(signer, amount_y);
    <b>let</b> liquidity_token = <a href="TokenSwap.md#0x1_TokenSwap_mint">TokenSwap::mint</a>(x_token, y_token);
    <b>if</b> (!<a href="Account.md#0x1_Account_is_accepts_token">Account::is_accepts_token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))) {
        <a href="Account.md#0x1_Account_accept_token">Account::accept_token</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(signer);
    };
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), liquidity_token);
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway__calculate_amount_for_liquidity"></a>

## Function `_calculate_amount_for_liquidity`



<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__calculate_amount_for_liquidity">_calculate_amount_for_liquidity</a>&lt;X, Y&gt;(amount_x_desired: u128, amount_y_desired: u128, amount_x_min: u128, amount_y_min: u128): (u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__calculate_amount_for_liquidity">_calculate_amount_for_liquidity</a>&lt;X, Y&gt;(
    amount_x_desired: u128,
    amount_y_desired: u128,
    amount_x_min: u128,
    amount_y_min: u128,
): (u128, u128) {
    <b>let</b> (reserve_x, reserve_y) = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_reserves">get_reserves</a>&lt;X, Y&gt;();
    <b>if</b> (reserve_x == 0 && reserve_y == 0) {
        <b>return</b> (amount_x_desired, amount_y_desired)
    } <b>else</b> {
        <b>let</b> amount_y_optimal = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_quote">quote</a>(amount_x_desired, reserve_x, reserve_y);
        <b>if</b> (amount_y_optimal &lt;= amount_y_desired) {
            <b>assert</b>(amount_y_optimal &gt;= amount_y_min, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INSUFFICIENT_Y_AMOUNT">INSUFFICIENT_Y_AMOUNT</a>);
            <b>return</b> (amount_x_desired, amount_y_optimal)
        } <b>else</b> {
            <b>let</b> amount_x_optimal = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_quote">quote</a>(amount_y_desired, reserve_y, reserve_x);
            <b>assert</b>(amount_x_optimal &lt;= amount_x_desired, 1000);
            <b>assert</b>(amount_x_optimal &gt;= amount_x_min, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INSUFFICIENT_X_AMOUNT">INSUFFICIENT_X_AMOUNT</a>);
            <b>return</b> (amount_x_optimal, amount_y_desired)
        }
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_remove_liquidity"></a>

## Function `remove_liquidity`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_remove_liquidity">remove_liquidity</a>&lt;X, Y&gt;(signer: &signer, liquidity: u128, amount_x_min: u128, amount_y_min: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_remove_liquidity">remove_liquidity</a>&lt;X, Y&gt;(
    signer: &signer,
    liquidity: u128,
    amount_x_min: u128,
    amount_y_min: u128,
) {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <b>if</b> (order == 1) {
        <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__remove_liquidity">_remove_liquidity</a>&lt;X, Y&gt;(signer, liquidity, amount_x_min, amount_y_min);
    } <b>else</b> {
        <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__remove_liquidity">_remove_liquidity</a>&lt;Y, X&gt;(signer, liquidity, amount_y_min, amount_x_min);
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway__remove_liquidity"></a>

## Function `_remove_liquidity`



<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__remove_liquidity">_remove_liquidity</a>&lt;X, Y&gt;(signer: &signer, liquidity: u128, amount_x_min: u128, amount_y_min: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway__remove_liquidity">_remove_liquidity</a>&lt;X, Y&gt;(
    signer: &signer,
    liquidity: u128,
    amount_x_min: u128,
    amount_y_min: u128,
) {
    <b>let</b> liquidity_token = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;<a href="TokenSwap.md#0x1_LiquidityToken">LiquidityToken</a>&lt;X, Y&gt;&gt;(signer, liquidity);
    <b>let</b> (token_x, token_y) = <a href="TokenSwap.md#0x1_TokenSwap_burn">TokenSwap::burn</a>(liquidity_token);
    <b>assert</b>(<a href="Token.md#0x1_Token_value">Token::value</a>(&token_x) &gt;= amount_x_min, 1000);
    <b>assert</b>(<a href="Token.md#0x1_Token_value">Token::value</a>(&token_y) &gt;= amount_y_min, 1000);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), token_x);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), token_y);
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_swap_exact_token_for_token"></a>

## Function `swap_exact_token_for_token`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_swap_exact_token_for_token">swap_exact_token_for_token</a>&lt;X, Y&gt;(signer: &signer, amount_x_in: u128, amount_y_out_min: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_swap_exact_token_for_token">swap_exact_token_for_token</a>&lt;X, Y&gt;(
    signer: &signer,
    amount_x_in: u128,
    amount_y_out_min: u128,
) {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    // calculate actual y out
    <b>let</b> (reserve_x, reserve_y) = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_reserves">get_reserves</a>&lt;X, Y&gt;();
    <b>let</b> y_out = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_out">get_amount_out</a>(amount_x_in, reserve_x, reserve_y);
    <b>assert</b>(y_out &gt;= amount_y_out_min, 4000);
    // do actual swap
    <b>let</b> token_x = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;X&gt;(signer, amount_x_in);
    <b>let</b> (token_x_out, token_y_out);
    <b>if</b> (order == 1) {
        (token_x_out, token_y_out) = <a href="TokenSwap.md#0x1_TokenSwap_swap">TokenSwap::swap</a>&lt;X, Y&gt;(token_x, y_out, <a href="Token.md#0x1_Token_zero">Token::zero</a>(), 0);
    } <b>else</b> {
        (token_y_out, token_x_out) = <a href="TokenSwap.md#0x1_TokenSwap_swap">TokenSwap::swap</a>&lt;Y, X&gt;(<a href="Token.md#0x1_Token_zero">Token::zero</a>(), 0, token_x, y_out);
    };
    <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(token_x_out);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), token_y_out);
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_swap_token_for_exact_token"></a>

## Function `swap_token_for_exact_token`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_swap_token_for_exact_token">swap_token_for_exact_token</a>&lt;X, Y&gt;(signer: &signer, amount_x_in_max: u128, amount_y_out: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_swap_token_for_exact_token">swap_token_for_exact_token</a>&lt;X, Y&gt;(
    signer: &signer,
    amount_x_in_max: u128,
    amount_y_out: u128,
) {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    // calculate actual y out
    <b>let</b> (reserve_x, reserve_y) = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_reserves">get_reserves</a>&lt;X, Y&gt;();
    <b>let</b> x_in = <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_in">get_amount_in</a>(amount_y_out, reserve_x, reserve_y);
    <b>assert</b>(x_in &lt;= amount_x_in_max, 4000);
    // do actual swap
    <b>let</b> token_x = <a href="Account.md#0x1_Account_withdraw">Account::withdraw</a>&lt;X&gt;(signer, x_in);
    <b>let</b> (token_x_out, token_y_out);
    <b>if</b> (order == 1) {
        (token_x_out, token_y_out) =
            <a href="TokenSwap.md#0x1_TokenSwap_swap">TokenSwap::swap</a>&lt;X, Y&gt;(token_x, amount_y_out, <a href="Token.md#0x1_Token_zero">Token::zero</a>(), 0);
    } <b>else</b> {
        (token_y_out, token_x_out) =
            <a href="TokenSwap.md#0x1_TokenSwap_swap">TokenSwap::swap</a>&lt;Y, X&gt;(<a href="Token.md#0x1_Token_zero">Token::zero</a>(), 0, token_x, amount_y_out);
    };
    <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>(token_x_out);
    <a href="Account.md#0x1_Account_deposit">Account::deposit</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer), token_y_out);
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_get_reserves"></a>

## Function `get_reserves`

Get reserves of a token pair.
The order of <code>X</code>, <code>Y</code> doesn't need to be sorted.
And the order of return values are based on the order of type parameters.


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_reserves">get_reserves</a>&lt;X, Y&gt;(): (u128, u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_reserves">get_reserves</a>&lt;X, Y&gt;(): (u128, u128) {
    <b>let</b> order = <a href="TokenSwap.md#0x1_TokenSwap_compare_token">TokenSwap::compare_token</a>&lt;X, Y&gt;();
    <b>assert</b>(order != 0, <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_INVALID_TOKEN_PAIR">INVALID_TOKEN_PAIR</a>);
    <b>if</b> (order == 1) {
        <a href="TokenSwap.md#0x1_TokenSwap_get_reserves">TokenSwap::get_reserves</a>&lt;X, Y&gt;()
    } <b>else</b> {
        <b>let</b> (y, x) = <a href="TokenSwap.md#0x1_TokenSwap_get_reserves">TokenSwap::get_reserves</a>&lt;Y, X&gt;();
        (x, y)
    }
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_quote"></a>

## Function `quote`

Return amount_y needed to provide liquidity given <code>amount_x</code>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_quote">quote</a>(amount_x: u128, reserve_x: u128, reserve_y: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_quote">quote</a>(amount_x: u128, reserve_x: u128, reserve_y: u128): u128 {
    <b>assert</b>(amount_x &gt; 0, 400);
    <b>assert</b>(reserve_x &gt; 0 && reserve_y &gt; 0, 410);
    <b>let</b> amount_y = amount_x * reserve_y / reserve_x;
    amount_y
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_get_amount_out"></a>

## Function `get_amount_out`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_out">get_amount_out</a>(amount_in: u128, reserve_in: u128, reserve_out: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_out">get_amount_out</a>(amount_in: u128, reserve_in: u128, reserve_out: u128): u128 {
    <b>assert</b>(amount_in &gt; 0, 400);
    <b>assert</b>(reserve_in &gt; 0 && reserve_out &gt; 0, 410);
    <b>let</b> amount_in_with_fee = amount_in * 997;
    <b>let</b> numerator = amount_in_with_fee * reserve_out;
    <b>let</b> denominator = reserve_in * 1000 + amount_in_with_fee;
    numerator / denominator
}
</code></pre>



</details>

<a name="0x1_TokenSwapGateway_get_amount_in"></a>

## Function `get_amount_in`



<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_in">get_amount_in</a>(amount_out: u128, reserve_in: u128, reserve_out: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TokenSwapGateway.md#0x1_TokenSwapGateway_get_amount_in">get_amount_in</a>(amount_out: u128, reserve_in: u128, reserve_out: u128): u128 {
    <b>assert</b>(amount_out &gt; 0, 400);
    <b>assert</b>(reserve_in &gt; 0 && reserve_out &gt; 0, 410);
    <b>let</b> numerator = reserve_in * amount_out * 1000;
    <b>let</b> denominator = (reserve_out - amount_out) * 997;
    numerator / denominator + 1
}
</code></pre>



</details>
