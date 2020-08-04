
<a name="0x1_Token"></a>

# Module `0x1::Token`

### Table of Contents

-  [Resource `Token`](#0x1_Token_Token)
-  [Resource `MintCapability`](#0x1_Token_MintCapability)
-  [Resource `BurnCapability`](#0x1_Token_BurnCapability)
-  [Struct `MintEvent`](#0x1_Token_MintEvent)
-  [Struct `BurnEvent`](#0x1_Token_BurnEvent)
-  [Resource `TokenInfo`](#0x1_Token_TokenInfo)
-  [Function `register_token`](#0x1_Token_register_token)
-  [Function `remove_mint_capability`](#0x1_Token_remove_mint_capability)
-  [Function `add_mint_capability`](#0x1_Token_add_mint_capability)
-  [Function `destroy_mint_capability`](#0x1_Token_destroy_mint_capability)
-  [Function `remove_burn_capability`](#0x1_Token_remove_burn_capability)
-  [Function `add_burn_capability`](#0x1_Token_add_burn_capability)
-  [Function `destroy_burn_capability`](#0x1_Token_destroy_burn_capability)
-  [Function `mint`](#0x1_Token_mint)
-  [Function `mint_with_capability`](#0x1_Token_mint_with_capability)
-  [Function `burn`](#0x1_Token_burn)
-  [Function `burn_with_capability`](#0x1_Token_burn_with_capability)
-  [Function `zero`](#0x1_Token_zero)
-  [Function `value`](#0x1_Token_value)
-  [Function `split`](#0x1_Token_split)
-  [Function `withdraw`](#0x1_Token_withdraw)
-  [Function `join`](#0x1_Token_join)
-  [Function `deposit`](#0x1_Token_deposit)
-  [Function `destroy_zero`](#0x1_Token_destroy_zero)
-  [Function `scaling_factor`](#0x1_Token_scaling_factor)
-  [Function `fractional_part`](#0x1_Token_fractional_part)
-  [Function `market_cap`](#0x1_Token_market_cap)
-  [Function `is_registered_in`](#0x1_Token_is_registered_in)
-  [Function `is_same_token`](#0x1_Token_is_same_token)
-  [Function `token_address`](#0x1_Token_token_address)
-  [Function `token_code`](#0x1_Token_token_code)
-  [Function `code_to_bytes`](#0x1_Token_code_to_bytes)
-  [Function `name_of`](#0x1_Token_name_of)



<a name="0x1_Token_Token"></a>

## Resource `Token`

The token has a
<code>TokenType</code> color that tells us what token the
<code>value</code> inside represents.


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>value: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Token_MintCapability"></a>

## Resource `MintCapability`

A minting capability allows tokens of type
<code>TokenType</code> to be minted


<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;
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

<a name="0x1_Token_BurnCapability"></a>

## Resource `BurnCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;
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

<a name="0x1_Token_MintEvent"></a>

## Struct `MintEvent`



<pre><code><b>struct</b> <a href="#0x1_Token_MintEvent">MintEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u128</code>
</dt>
<dd>
 funds added to the system
</dd>
<dt>

<code>token_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 full info of Token.
</dd>
</dl>


</details>

<a name="0x1_Token_BurnEvent"></a>

## Struct `BurnEvent`



<pre><code><b>struct</b> <a href="#0x1_Token_BurnEvent">BurnEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>amount: u128</code>
</dt>
<dd>
 funds removed from the system
</dd>
<dt>

<code>token_code: vector&lt;u8&gt;</code>
</dt>
<dd>
 full info of Token
</dd>
</dl>


</details>

<a name="0x1_Token_TokenInfo"></a>

## Resource `TokenInfo`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>total_value: u128</code>
</dt>
<dd>
 The total value for the token represented by
 <code>TokenType</code>. Mutable.
</dd>
<dt>

<code>scaling_factor: u128</code>
</dt>
<dd>
 The scaling factor for the token (i.e. the amount to multiply by
 to get to the human-readable reprentation for this token). e.g. 10^6 for Token1
</dd>
<dt>

<code>fractional_part: u128</code>
</dt>
<dd>
 The smallest fractional part (number of decimal places) to be
 used in the human-readable representation for the token (e.g.
 10^2 for Token1 cents)
</dd>
<dt>

<code>mint_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Token_MintEvent">Token::MintEvent</a>&gt;</code>
</dt>
<dd>
 event stream for minting
</dd>
<dt>

<code>burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="#0x1_Token_BurnEvent">Token::BurnEvent</a>&gt;</code>
</dt>
<dd>
 event stream for burning
</dd>
</dl>


</details>

<a name="0x1_Token_register_token"></a>

## Function `register_token`

Register the type
<code>TokenType</code> as a Token and got MintCapability and BurnCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(account: &signer, scaling_factor: u128, fractional_part: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(
    account: &signer,
    scaling_factor: u128,
    fractional_part: u128,
) {
    <b>let</b> (token_address, module_name, token_name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == token_address, ETOKEN_REGISTER);
    <b>assert</b>(module_name == token_name, ETOKEN_NAME);
    move_to(account, <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; {});
    move_to(account, <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; {});
    move_to(
        account,
        <a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt; {
            total_value: 0,
            scaling_factor,
            fractional_part,
            mint_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Token_MintEvent">MintEvent</a>&gt;(account),
            burn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Token_BurnEvent">BurnEvent</a>&gt;(account),
        },
    );
}
</code></pre>



</details>

<a name="0x1_Token_remove_mint_capability"></a>

## Function `remove_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(
    signer: &signer,
): <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_MintCapability">MintCapability</a> {
    move_from&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Token_add_mint_capability"></a>

## Function `add_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer,
cap: <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;)  {
    move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Token_destroy_mint_capability"></a>

## Function `destroy_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;{  } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_remove_burn_capability"></a>

## Function `remove_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(
    signer: &signer,
): <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_BurnCapability">BurnCapability</a> {
    move_from&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Token_add_burn_capability"></a>

## Function `add_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer,
    cap: <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;)  {
        move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Token_destroy_burn_capability"></a>

## Function `destroy_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;{  } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_mint"></a>

## Function `mint`

Return
<code>amount</code> tokens.
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint">mint</a>&lt;TokenType&gt;(
    account: &signer,
    amount: u128,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>, <a href="#0x1_Token_MintCapability">MintCapability</a> {
    <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>(
        borrow_global&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        amount,
    )
}
</code></pre>



</details>

<a name="0x1_Token_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new Token::Token worth
<code>value</code>. The caller must have a reference to a MintCapability.
Only the Association account can acquire such a reference, and it can do so only via
<code>borrow_sender_mint_capability</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, value: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(
    _capability: &<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;,
    value: u128,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> (token_address, module_name, token_name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    info.total_value = info.total_value + (value <b>as</b> u128);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.mint_events,
        <a href="#0x1_Token_MintEvent">MintEvent</a> {
            amount: value,
            token_code: <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        }
    );
    <a href="#0x1_Token">Token</a>&lt;TokenType&gt; { value }
}
</code></pre>



</details>

<a name="0x1_Token_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn">burn</a>&lt;TokenType&gt;(
    account: &signer,
    tokens: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
) <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>, <a href="#0x1_Token_BurnCapability">BurnCapability</a> {
    <a href="#0x1_Token_burn_with_capability">burn_with_capability</a>(
        borrow_global&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        tokens,
    )
}
</code></pre>



</details>

<a name="0x1_Token_burn_with_capability"></a>

## Function `burn_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;, tokens: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(
    _capability: &<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;,
    tokens: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
) <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, module_name, token_name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    <b>let</b> <a href="#0x1_Token">Token</a>{ value: value } = tokens;
    info.total_value = info.total_value - (value <b>as</b> u128);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.burn_events,
        <a href="#0x1_Token_BurnEvent">BurnEvent</a> {
            amount: value,
            token_code: <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        }
    );

}
</code></pre>



</details>

<a name="0x1_Token_zero"></a>

## Function `zero`

Create a new Token::Token<TokenType> with a value of 0


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; {
    <a href="#0x1_Token">Token</a>&lt;TokenType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x1_Token_value"></a>

## Function `value`

Public accessor for the value of a token


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token">Token</a>&lt;TokenType&gt;): u128 {
    token.value
}
</code></pre>



</details>

<a name="0x1_Token_split"></a>

## Function `split`

Splits the given token into two and returns them both
It leverages
<code><a href="#0x1_Token_withdraw">Self::withdraw</a></code> for any verifications of the values


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split">split</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): (<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split">split</a>&lt;TokenType&gt;(
    token: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    amount: u128,
): (<a href="#0x1_Token">Token</a>&lt;TokenType&gt;, <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> other = <a href="#0x1_Token_withdraw">withdraw</a>(&<b>mut</b> token, amount);
    (token, other)
}
</code></pre>



</details>

<a name="0x1_Token_withdraw"></a>

## Function `withdraw`

"Divides" the given token into two, where the original token is modified in place
The original token will have value = original value -
<code>amount</code>
The new token will have a value =
<code>amount</code>
Fails if the tokens value is less than
<code>amount</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(
    token: &<b>mut</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    amount: u128,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; {
    // Check that `amount` is less than the token's value
    <b>assert</b>(token.value &gt;= amount, EAMOUNT_EXCEEDS_COIN_VALUE);
    token.value = token.value - amount;
    <a href="#0x1_Token">Token</a> { value: amount }
}
</code></pre>



</details>

<a name="0x1_Token_join"></a>

## Function `join`

Merges two tokens of the same token and returns a new token whose
value is equal to the sum of the two inputs


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_join">join</a>&lt;TokenType&gt;(token1: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, token2: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_join">join</a>&lt;TokenType&gt;(
    token1: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    token2: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; {
    <a href="#0x1_Token_deposit">deposit</a>(&<b>mut</b> token1, token2);
    token1
}
</code></pre>



</details>

<a name="0x1_Token_deposit"></a>

## Function `deposit`

"Merges" the two tokens
The token passed in by reference will have a value equal to the sum of the two tokens
The
<code>check</code> token is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, check: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;, check: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_Token">Token</a>{ value: value } = check;
    token.value = token.value + value;
}
</code></pre>



</details>

<a name="0x1_Token_destroy_zero"></a>

## Function `destroy_zero`

Destroy a token
Fails if the value is non-zero
The amount of Token in the system is a tightly controlled property,
so you cannot "burn" any non-zero amount of Token


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_Token">Token</a>{ value: value } = token;
    <b>assert</b>(value == 0, <a href="ErrorCode.md#0x1_ErrorCode_EDESTORY_TOKEN_NON_ZERO">ErrorCode::EDESTORY_TOKEN_NON_ZERO</a>())
}
</code></pre>



</details>

<a name="0x1_Token_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling factor for the
<code>TokenType</code> token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) =<a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).scaling_factor
}
</code></pre>



</details>

<a name="0x1_Token_fractional_part"></a>

## Function `fractional_part`

Returns the representable fractional part for the
<code>TokenType</code> token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_fractional_part">fractional_part</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_fractional_part">fractional_part</a>&lt;TokenType&gt;(): u128
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) =<a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).fractional_part
}
</code></pre>



</details>

<a name="0x1_Token_market_cap"></a>

## Function `market_cap`

Return the total amount of token minted of type
<code>TokenType</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) =<a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).total_value
}
</code></pre>



</details>

<a name="0x1_Token_is_registered_in"></a>

## Function `is_registered_in`

Return true if the type
<code>TokenType</code> is a registered in
<code>token_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool {
    exists&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address)
}
</code></pre>



</details>

<a name="0x1_Token_is_same_token"></a>

## Function `is_same_token`

Return true if the type
<code>TokenType1</code> is same with
<code>TokenType2</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1,TokenType2&gt;(): bool {
    <b>return</b> <a href="#0x1_Token_token_code">token_code</a>&lt;TokenType1&gt;() == <a href="#0x1_Token_token_code">token_code</a>&lt;TokenType2&gt;()
}
</code></pre>



</details>

<a name="0x1_Token_token_address"></a>

## Function `token_address`

Return the TokenType's address


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;():address {
    <b>let</b> (addr, _, _) =<a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    addr
}
</code></pre>



</details>

<a name="0x1_Token_token_code"></a>

## Function `token_code`

Return the token code for the registered token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt; {
    <b>let</b> (addr, module_name, name) =<a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(addr, module_name, name)
}
</code></pre>



</details>

<a name="0x1_Token_code_to_bytes"></a>

## Function `code_to_bytes`



<pre><code><b>fun</b> <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt; {
    <b>let</b> code = <a href="LCS.md#0x1_LCS_to_bytes">LCS::to_bytes</a>(&addr);

    // {{addr}}::{{<b>module</b>}}::{{<b>struct</b>}}
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> code, b"::");
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> code, module_name);
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> code, b"::");
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> code, name);

    code
}
</code></pre>



</details>

<a name="0x1_Token_name_of"></a>

## Function `name_of`

Return Token's module address, module name, and type name of
<code>TokenType</code>.


<pre><code><b>fun</b> <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;);
</code></pre>



</details>
