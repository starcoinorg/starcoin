
<a name="0x1_Token"></a>

# Module `0x1::Token`



-  [Resource <code><a href="Token.md#0x1_Token">Token</a></code>](#0x1_Token_Token)
-  [Resource <code><a href="Token.md#0x1_Token_MintCapability">MintCapability</a></code>](#0x1_Token_MintCapability)
-  [Resource <code><a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a></code>](#0x1_Token_FixedTimeMintKey)
-  [Resource <code><a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a></code>](#0x1_Token_LinearTimeMintKey)
-  [Resource <code><a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a></code>](#0x1_Token_BurnCapability)
-  [Struct <code><a href="Token.md#0x1_Token_MintEvent">MintEvent</a></code>](#0x1_Token_MintEvent)
-  [Struct <code><a href="Token.md#0x1_Token_BurnEvent">BurnEvent</a></code>](#0x1_Token_BurnEvent)
-  [Resource <code><a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a></code>](#0x1_Token_TokenInfo)
-  [Const <code><a href="Token.md#0x1_Token_MAX_PRECISION">MAX_PRECISION</a></code>](#0x1_Token_MAX_PRECISION)
-  [Function <code>ETOKEN_REGISTER</code>](#0x1_Token_ETOKEN_REGISTER)
-  [Function <code>EAMOUNT_EXCEEDS_COIN_VALUE</code>](#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE)
-  [Function <code>EMINT_KEY_TIME_LIMIT</code>](#0x1_Token_EMINT_KEY_TIME_LIMIT)
-  [Function <code>EDESTROY_KEY_NOT_EMPTY</code>](#0x1_Token_EDESTROY_KEY_NOT_EMPTY)
-  [Function <code>EPRECISION_TOO_LARGE</code>](#0x1_Token_EPRECISION_TOO_LARGE)
-  [Function <code>register_token</code>](#0x1_Token_register_token)
-  [Function <code>remove_mint_capability</code>](#0x1_Token_remove_mint_capability)
-  [Function <code>add_mint_capability</code>](#0x1_Token_add_mint_capability)
-  [Function <code>destroy_mint_capability</code>](#0x1_Token_destroy_mint_capability)
-  [Function <code>remove_burn_capability</code>](#0x1_Token_remove_burn_capability)
-  [Function <code>add_burn_capability</code>](#0x1_Token_add_burn_capability)
-  [Function <code>destroy_burn_capability</code>](#0x1_Token_destroy_burn_capability)
-  [Function <code>mint</code>](#0x1_Token_mint)
-  [Function <code>mint_with_capability</code>](#0x1_Token_mint_with_capability)
-  [Function <code>do_mint</code>](#0x1_Token_do_mint)
-  [Function <code>issue_fixed_mint_key</code>](#0x1_Token_issue_fixed_mint_key)
-  [Function <code>issue_linear_mint_key</code>](#0x1_Token_issue_linear_mint_key)
-  [Function <code>mint_with_fixed_key</code>](#0x1_Token_mint_with_fixed_key)
-  [Function <code>mint_with_linear_key</code>](#0x1_Token_mint_with_linear_key)
-  [Function <code>mint_amount_of_linear_key</code>](#0x1_Token_mint_amount_of_linear_key)
-  [Function <code>mint_amount_of_fixed_key</code>](#0x1_Token_mint_amount_of_fixed_key)
-  [Function <code>end_time_of_key</code>](#0x1_Token_end_time_of_key)
-  [Function <code>destroy_empty_key</code>](#0x1_Token_destroy_empty_key)
-  [Function <code>burn</code>](#0x1_Token_burn)
-  [Function <code>burn_with_capability</code>](#0x1_Token_burn_with_capability)
-  [Function <code>zero</code>](#0x1_Token_zero)
-  [Function <code>value</code>](#0x1_Token_value)
-  [Function <code>split</code>](#0x1_Token_split)
-  [Function <code>withdraw</code>](#0x1_Token_withdraw)
-  [Function <code>join</code>](#0x1_Token_join)
-  [Function <code>deposit</code>](#0x1_Token_deposit)
-  [Function <code>destroy_zero</code>](#0x1_Token_destroy_zero)
-  [Function <code>scaling_factor</code>](#0x1_Token_scaling_factor)
-  [Function <code>market_cap</code>](#0x1_Token_market_cap)
-  [Function <code>is_registered_in</code>](#0x1_Token_is_registered_in)
-  [Function <code>is_same_token</code>](#0x1_Token_is_same_token)
-  [Function <code>token_address</code>](#0x1_Token_token_address)
-  [Function <code>token_code</code>](#0x1_Token_token_code)
-  [Function <code>code_to_bytes</code>](#0x1_Token_code_to_bytes)
-  [Function <code>name_of</code>](#0x1_Token_name_of)
-  [Function <code>name_of_token</code>](#0x1_Token_name_of_token)
-  [Specification](#@Specification_0)
    -  [Function <code>register_token</code>](#@Specification_0_register_token)
    -  [Function <code>remove_mint_capability</code>](#@Specification_0_remove_mint_capability)
    -  [Function <code>add_mint_capability</code>](#@Specification_0_add_mint_capability)
    -  [Function <code>destroy_mint_capability</code>](#@Specification_0_destroy_mint_capability)
    -  [Function <code>remove_burn_capability</code>](#@Specification_0_remove_burn_capability)
    -  [Function <code>add_burn_capability</code>](#@Specification_0_add_burn_capability)
    -  [Function <code>destroy_burn_capability</code>](#@Specification_0_destroy_burn_capability)
    -  [Function <code>mint</code>](#@Specification_0_mint)
    -  [Function <code>mint_with_capability</code>](#@Specification_0_mint_with_capability)
    -  [Function <code>do_mint</code>](#@Specification_0_do_mint)
    -  [Function <code>issue_fixed_mint_key</code>](#@Specification_0_issue_fixed_mint_key)
    -  [Function <code>issue_linear_mint_key</code>](#@Specification_0_issue_linear_mint_key)
    -  [Function <code>mint_with_fixed_key</code>](#@Specification_0_mint_with_fixed_key)
    -  [Function <code>mint_with_linear_key</code>](#@Specification_0_mint_with_linear_key)
    -  [Function <code>mint_amount_of_linear_key</code>](#@Specification_0_mint_amount_of_linear_key)
    -  [Function <code>mint_amount_of_fixed_key</code>](#@Specification_0_mint_amount_of_fixed_key)
    -  [Function <code>destroy_empty_key</code>](#@Specification_0_destroy_empty_key)
    -  [Function <code>burn</code>](#@Specification_0_burn)
    -  [Function <code>burn_with_capability</code>](#@Specification_0_burn_with_capability)
    -  [Function <code>zero</code>](#@Specification_0_zero)
    -  [Function <code>value</code>](#@Specification_0_value)
    -  [Function <code>split</code>](#@Specification_0_split)
    -  [Function <code>withdraw</code>](#@Specification_0_withdraw)
    -  [Function <code>join</code>](#@Specification_0_join)
    -  [Function <code>deposit</code>](#@Specification_0_deposit)
    -  [Function <code>destroy_zero</code>](#@Specification_0_destroy_zero)
    -  [Function <code>scaling_factor</code>](#@Specification_0_scaling_factor)
    -  [Function <code>market_cap</code>](#@Specification_0_market_cap)
    -  [Function <code>is_registered_in</code>](#@Specification_0_is_registered_in)
    -  [Function <code>is_same_token</code>](#@Specification_0_is_same_token)
    -  [Function <code>token_address</code>](#@Specification_0_token_address)
    -  [Function <code>token_code</code>](#@Specification_0_token_code)
    -  [Function <code>code_to_bytes</code>](#@Specification_0_code_to_bytes)
    -  [Function <code>name_of</code>](#@Specification_0_name_of)
    -  [Function <code>name_of_token</code>](#@Specification_0_name_of_token)


<a name="0x1_Token_Token"></a>

## Resource `Token`

The token has a <code>TokenType</code> color that tells us what token the
<code>value</code> inside represents.


<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
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

A minting capability allows tokens of type <code>TokenType</code> to be minted


<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;
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

<a name="0x1_Token_FixedTimeMintKey"></a>

## Resource `FixedTimeMintKey`



<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>end_time: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Token_LinearTimeMintKey"></a>

## Resource `LinearTimeMintKey`



<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>minted: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>peroid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Token_BurnCapability"></a>

## Resource `BurnCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;
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



<pre><code><b>struct</b> <a href="Token.md#0x1_Token_MintEvent">MintEvent</a>
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



<pre><code><b>struct</b> <a href="Token.md#0x1_Token_BurnEvent">BurnEvent</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;
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
 The scaling factor for the coin (i.e. the amount to divide by
 to get to the human-readable representation for this currency).
 e.g. 10^6 for <code>Coin1</code>
</dd>
<dt>
<code>mint_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Token.md#0x1_Token_MintEvent">Token::MintEvent</a>&gt;</code>
</dt>
<dd>
 event stream for minting
</dd>
<dt>
<code>burn_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Token.md#0x1_Token_BurnEvent">Token::BurnEvent</a>&gt;</code>
</dt>
<dd>
 event stream for burning
</dd>
</dl>


</details>

<a name="0x1_Token_MAX_PRECISION"></a>

## Const `MAX_PRECISION`

2^128 < 10**39


<pre><code><b>const</b> <a href="Token.md#0x1_Token_MAX_PRECISION">MAX_PRECISION</a>: u8 = 38;
</code></pre>



<a name="0x1_Token_ETOKEN_REGISTER"></a>

## Function `ETOKEN_REGISTER`

Token register's address should same as TokenType's address.


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_ETOKEN_REGISTER">ETOKEN_REGISTER</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_ETOKEN_REGISTER">ETOKEN_REGISTER</a>(): u64 {
    <a href="Errors.md#0x1_Errors_ECODE_BASE">Errors::ECODE_BASE</a>() + 1
}
</code></pre>



</details>

<a name="0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE"></a>

## Function `EAMOUNT_EXCEEDS_COIN_VALUE`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>(): u64 {
    <a href="Errors.md#0x1_Errors_ECODE_BASE">Errors::ECODE_BASE</a>() + 2
}
</code></pre>



</details>

<a name="0x1_Token_EMINT_KEY_TIME_LIMIT"></a>

## Function `EMINT_KEY_TIME_LIMIT`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EMINT_KEY_TIME_LIMIT">EMINT_KEY_TIME_LIMIT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EMINT_KEY_TIME_LIMIT">EMINT_KEY_TIME_LIMIT</a>(): u64 {
    <a href="Errors.md#0x1_Errors_ECODE_BASE">Errors::ECODE_BASE</a>() + 3
}
</code></pre>



</details>

<a name="0x1_Token_EDESTROY_KEY_NOT_EMPTY"></a>

## Function `EDESTROY_KEY_NOT_EMPTY`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>(): u64 {
    <a href="Errors.md#0x1_Errors_ECODE_BASE">Errors::ECODE_BASE</a>() + 4
}
</code></pre>



</details>

<a name="0x1_Token_EPRECISION_TOO_LARGE"></a>

## Function `EPRECISION_TOO_LARGE`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EPRECISION_TOO_LARGE">EPRECISION_TOO_LARGE</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_EPRECISION_TOO_LARGE">EPRECISION_TOO_LARGE</a>(): u64 {
    <a href="Errors.md#0x1_Errors_ECODE_BASE">Errors::ECODE_BASE</a>() + 5
}
</code></pre>



</details>

<a name="0x1_Token_register_token"></a>

## Function `register_token`

Register the type <code>TokenType</code> as a Token and got MintCapability and BurnCapability.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(account: &signer, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(
    account: &signer,
    precision: u8,
) {
    <b>assert</b>(precision &lt;= <a href="Token.md#0x1_Token_MAX_PRECISION">MAX_PRECISION</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Token.md#0x1_Token_EPRECISION_TOO_LARGE">EPRECISION_TOO_LARGE</a>()));
    <b>let</b> scaling_factor = <a href="Math.md#0x1_Math_pow">Math::pow</a>(10, (precision <b>as</b> u64));
    <b>let</b> token_address = <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == token_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Token.md#0x1_Token_ETOKEN_REGISTER">ETOKEN_REGISTER</a>()));
    move_to(account, <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; {});
    move_to(account, <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; {});
    move_to(
        account,
        <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt; {
            total_value: 0,
            scaling_factor,
            mint_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Token.md#0x1_Token_MintEvent">MintEvent</a>&gt;(account),
            burn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Token.md#0x1_Token_BurnEvent">BurnEvent</a>&gt;(account),
        },
    );
}
</code></pre>



</details>

<a name="0x1_Token_remove_mint_capability"></a>

## Function `remove_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Token.md#0x1_Token_MintCapability">MintCapability</a> {
    move_from&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Token_add_mint_capability"></a>

## Function `add_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;) {
    move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Token_destroy_mint_capability"></a>

## Function `destroy_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_remove_burn_capability"></a>

## Function `remove_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a> {
    move_from&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Token_add_burn_capability"></a>

## Function `add_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;) {
    move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Token_destroy_burn_capability"></a>

## Function `destroy_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_mint"></a>

## Function `mint`

Return <code>amount</code> tokens.
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>, <a href="Token.md#0x1_Token_MintCapability">MintCapability</a> {
    <a href="Token.md#0x1_Token_mint_with_capability">mint_with_capability</a>(
        borrow_global&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        amount,
    )
}
</code></pre>



</details>

<a name="0x1_Token_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new Token::Token worth <code>amount</code>.
The caller must have a reference to a MintCapability.
Only the Association account can acquire such a reference, and it can do so only via
<code>borrow_sender_mint_capability</code>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(
    _capability: &<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;,
    amount: u128,
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <a href="Token.md#0x1_Token_do_mint">do_mint</a>(amount)
}
</code></pre>



</details>

<a name="0x1_Token_do_mint"></a>

## Function `do_mint`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_do_mint">do_mint</a>&lt;TokenType&gt;(amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_do_mint">do_mint</a>&lt;TokenType&gt;(amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> (token_address, module_name, token_name) = <a href="Token.md#0x1_Token_name_of_token">name_of_token</a>&lt;TokenType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    info.total_value = info.total_value + amount;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.mint_events,
        <a href="Token.md#0x1_Token_MintEvent">MintEvent</a> {
            amount,
            token_code: <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        },
    );
    <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: amount }
}
</code></pre>



</details>

<a name="0x1_Token_issue_fixed_mint_key"></a>

## Function `issue_fixed_mint_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_fixed_mint_key">issue_fixed_mint_key</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128, peroid: u64): <a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_fixed_mint_key">issue_fixed_mint_key</a>&lt;TokenType&gt;( _capability: &<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;,
                                 amount: u128, peroid: u64): <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;{
    <b>assert</b>(peroid &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">Errors::EINVALID_ARGUMENT</a>()));
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">Errors::EINVALID_ARGUMENT</a>()));
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> end_time = now + peroid;
    <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>{
        total: amount,
        end_time,
    }
}
</code></pre>



</details>

<a name="0x1_Token_issue_linear_mint_key"></a>

## Function `issue_linear_mint_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_linear_mint_key">issue_linear_mint_key</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128, peroid: u64): <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_linear_mint_key">issue_linear_mint_key</a>&lt;TokenType&gt;( _capability: &<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;,
                                            amount: u128, peroid: u64): <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt;{
    <b>assert</b>(peroid &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">Errors::EINVALID_ARGUMENT</a>()));
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_EINVALID_ARGUMENT">Errors::EINVALID_ARGUMENT</a>()));
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt; {
        total: amount,
        minted: 0,
        start_time,
        peroid
    }
}
</code></pre>



</details>

<a name="0x1_Token_mint_with_fixed_key"></a>

## Function `mint_with_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_fixed_key">mint_with_fixed_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_fixed_key">mint_with_fixed_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> amount = <a href="Token.md#0x1_Token_mint_amount_of_fixed_key">mint_amount_of_fixed_key</a>(&key);
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Token.md#0x1_Token_EMINT_KEY_TIME_LIMIT">EMINT_KEY_TIME_LIMIT</a>()));
    <b>let</b> <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a> { total, end_time:_} = key;
    <a href="Token.md#0x1_Token_do_mint">do_mint</a>(total)
}
</code></pre>



</details>

<a name="0x1_Token_mint_with_linear_key"></a>

## Function `mint_with_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_linear_key">mint_with_linear_key</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_linear_key">mint_with_linear_key</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> amount = <a href="Token.md#0x1_Token_mint_amount_of_linear_key">mint_amount_of_linear_key</a>(key);
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Token.md#0x1_Token_EMINT_KEY_TIME_LIMIT">EMINT_KEY_TIME_LIMIT</a>()));
    <b>let</b> token = <a href="Token.md#0x1_Token_do_mint">do_mint</a>(amount);
    key.minted = key.minted + amount;
    token
}
</code></pre>



</details>

<a name="0x1_Token_mint_amount_of_linear_key"></a>

## Function `mint_amount_of_linear_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_linear_key">mint_amount_of_linear_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_linear_key">mint_amount_of_linear_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - key.start_time;
    <b>if</b> (elapsed_time &gt;= key.peroid) {
        key.total - key.minted
    }<b>else</b> {
        <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>(key.total, (elapsed_time <b>as</b> u128), (key.peroid <b>as</b> u128)) - key.minted
    }
}
</code></pre>



</details>

<a name="0x1_Token_mint_amount_of_fixed_key"></a>

## Function `mint_amount_of_fixed_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_fixed_key">mint_amount_of_fixed_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_fixed_key">mint_amount_of_fixed_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>if</b> (now &gt;= key.end_time) {
        key.total
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_Token_end_time_of_key"></a>

## Function `end_time_of_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_end_time_of_key">end_time_of_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_end_time_of_key">end_time_of_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;): u64 {
    key.end_time
}
</code></pre>



</details>

<a name="0x1_Token_destroy_empty_key"></a>

## Function `destroy_empty_key`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_empty_key">destroy_empty_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_empty_key">destroy_empty_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="Token.md#0x1_Token_LinearTimeMintKey">LinearTimeMintKey</a>&lt;TokenType&gt; { total, minted, start_time: _, peroid: _ } = key;
    <b>assert</b>(total == minted, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Token.md#0x1_Token_EDESTROY_KEY_NOT_EMPTY">EDESTROY_KEY_NOT_EMPTY</a>()));
}
</code></pre>



</details>

<a name="0x1_Token_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>, <a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a> {
    <a href="Token.md#0x1_Token_burn_with_capability">burn_with_capability</a>(
        borrow_global&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        tokens,
    )
}
</code></pre>



</details>

<a name="0x1_Token_burn_with_capability"></a>

## Function `burn_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;, tokens: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(
    _capability: &<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;,
    tokens: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
) <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, module_name, token_name) = <a href="Token.md#0x1_Token_name_of_token">name_of_token</a>&lt;TokenType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    <b>let</b> <a href="Token.md#0x1_Token">Token</a> { value } = tokens;
    info.total_value = info.total_value - value;
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.burn_events,
        <a href="Token.md#0x1_Token_BurnEvent">BurnEvent</a> {
            amount: value,
            token_code: <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        },
    );
}
</code></pre>



</details>

<a name="0x1_Token_zero"></a>

## Function `zero`

Create a new Token::Token<TokenType> with a value of 0


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
    <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; { value: 0 }
}
</code></pre>



</details>

<a name="0x1_Token_value"></a>

## Function `value`

Public accessor for the value of a token


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;): u128 {
    token.value
}
</code></pre>



</details>

<a name="0x1_Token_split"></a>

## Function `split`

Splits the given token into two and returns them both


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_split">split</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, value: u128): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_split">split</a>&lt;TokenType&gt;(
    token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    value: u128,
): (<a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> other = <a href="Token.md#0x1_Token_withdraw">withdraw</a>(&<b>mut</b> token, value);
    (token, other)
}
</code></pre>



</details>

<a name="0x1_Token_withdraw"></a>

## Function `withdraw`

"Divides" the given token into two, where the original token is modified in place.
The original token will have value = original value - <code>value</code>
The new token will have a value = <code>value</code>
Fails if the tokens value is less than <code>value</code>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, value: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(
    token: &<b>mut</b> <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    value: u128,
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
    // Check that `value` is less than the token's value
    <b>assert</b>(token.value &gt;= value, <a href="Errors.md#0x1_Errors_limit_exceeded">Errors::limit_exceeded</a>(<a href="Token.md#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>()));
    token.value = token.value - value;
    <a href="Token.md#0x1_Token">Token</a> { value: value }
}
</code></pre>



</details>

<a name="0x1_Token_join"></a>

## Function `join`

Merges two tokens of the same token and returns a new token whose
value is equal to the sum of the two inputs


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_join">join</a>&lt;TokenType&gt;(token1: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, token2: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_join">join</a>&lt;TokenType&gt;(
    token1: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
    token2: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;,
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; {
    <a href="Token.md#0x1_Token_deposit">deposit</a>(&<b>mut</b> token1, token2);
    token1
}
</code></pre>



</details>

<a name="0x1_Token_deposit"></a>

## Function `deposit`

"Merges" the two tokens
The token passed in by reference will have a value equal to the sum of the two tokens
The <code>check</code> token is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, check: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, check: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="Token.md#0x1_Token">Token</a> { value } = check;
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


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="Token.md#0x1_Token">Token</a> { value } = token;
    <b>assert</b>(value == 0, <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Errors.md#0x1_Errors_EDESTORY_TOKEN_NON_ZERO">Errors::EDESTORY_TOKEN_NON_ZERO</a>()))
}
</code></pre>



</details>

<a name="0x1_Token_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling_factor for the <code>TokenType</code> token.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> token_address = <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).scaling_factor
}
</code></pre>



</details>

<a name="0x1_Token_market_cap"></a>

## Function `market_cap`

Return the total amount of token of type <code>TokenType</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> token_address = <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).total_value
}
</code></pre>



</details>

<a name="0x1_Token_is_registered_in"></a>

## Function `is_registered_in`

Return true if the type <code>TokenType</code> is a registered in <code>token_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool {
    <b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address)
}
</code></pre>



</details>

<a name="0x1_Token_is_same_token"></a>

## Function `is_same_token`

Return true if the type <code>TokenType1</code> is same with <code>TokenType2</code>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool {
    <b>return</b> <a href="Token.md#0x1_Token_token_code">token_code</a>&lt;TokenType1&gt;() == <a href="Token.md#0x1_Token_token_code">token_code</a>&lt;TokenType2&gt;()
}
</code></pre>



</details>

<a name="0x1_Token_token_address"></a>

## Function `token_address`

Return the TokenType's address


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address {
    <b>let</b> (addr, _, _) = <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    addr
}
</code></pre>



</details>

<a name="0x1_Token_token_code"></a>

## Function `token_code`

Return the token code for the registered token.


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt; {
    <b>let</b> (addr, module_name, name) = <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(addr, module_name, name)
}
</code></pre>



</details>

<a name="0x1_Token_code_to_bytes"></a>

## Function `code_to_bytes`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt; {
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

Return Token's module address, module name, and type name of <code>TokenType</code>.


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;);
</code></pre>



</details>

<a name="0x1_Token_name_of_token"></a>

## Function `name_of_token`



<pre><code><b>fun</b> <a href="Token.md#0x1_Token_name_of_token">name_of_token</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_name_of_token">name_of_token</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;) {
    <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;()
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify = <b>true</b>;
pragma aborts_if_is_strict = <b>true</b>;
</code></pre>


We use an uninterpreted function to represent the result of derived address. The actual value
does not matter for the verification of callers.


<a name="0x1_Token_spec_token_code"></a>


<pre><code><b>define</b> <a href="Token.md#0x1_Token_spec_token_code">spec_token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt;;
</code></pre>



<a name="@Specification_0_register_token"></a>

### Function `register_token`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(account: &signer, precision: u8)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> precision &gt; <a href="Token.md#0x1_Token_MAX_PRECISION">MAX_PRECISION</a>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_remove_mint_capability"></a>

### Function `remove_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_0_add_mint_capability"></a>

### Function `add_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_0_destroy_mint_capability"></a>

### Function `destroy_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>




<a name="@Specification_0_remove_burn_capability"></a>

### Function `remove_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_0_add_burn_capability"></a>

### Function `add_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> <b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_0_destroy_burn_capability"></a>

### Function `destroy_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>




<a name="@Specification_0_mint"></a>

### Function `mint`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() + amount &gt; MAX_U128;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_0_mint_with_capability"></a>

### Function `mint_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() + amount &gt; MAX_U128;
<b>ensures</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() ==
        <b>old</b>(<b>global</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>()).total_value) + amount;
</code></pre>



<a name="@Specification_0_do_mint"></a>

### Function `do_mint`


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_do_mint">do_mint</a>&lt;TokenType&gt;(amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>());
<b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() + amount &gt; MAX_U128;
</code></pre>



<a name="@Specification_0_issue_fixed_mint_key"></a>

### Function `issue_fixed_mint_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_fixed_mint_key">issue_fixed_mint_key</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128, peroid: u64): <a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> peroid == 0;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() + peroid &gt; MAX_U64;
</code></pre>



<a name="@Specification_0_issue_linear_mint_key"></a>

### Function `issue_linear_mint_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_issue_linear_mint_key">issue_linear_mint_key</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128, peroid: u64): <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> peroid == 0;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_0_mint_with_fixed_key"></a>

### Function `mint_with_fixed_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_fixed_key">mint_with_fixed_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="Token.md#0x1_Token_spec_mint_amount_of_fixed_key">spec_mint_amount_of_fixed_key</a>&lt;TokenType&gt;(key) == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>());
<b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() + key.total &gt; MAX_U128;
</code></pre>



<a name="@Specification_0_mint_with_linear_key"></a>

### Function `mint_with_linear_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_with_linear_key">mint_with_linear_key</a>&lt;TokenType&gt;(key: &<b>mut</b> <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_mint_amount_of_linear_key"></a>

### Function `mint_amount_of_linear_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_linear_key">mint_amount_of_linear_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() &lt; key.start_time;
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() - key.start_time &gt;= key.peroid && key.total &lt; key.minted;
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() - key.start_time &lt; key.peroid && <a href="Math.md#0x1_Math_spec_mul_div">Math::spec_mul_div</a>(key.total) &lt; key.minted;
</code></pre>



<a name="@Specification_0_mint_amount_of_fixed_key"></a>

### Function `mint_amount_of_fixed_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_mint_amount_of_fixed_key">mint_amount_of_fixed_key</a>&lt;TokenType&gt;(key: &<a href="Token.md#0x1_Token_FixedTimeMintKey">Token::FixedTimeMintKey</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>




<a name="0x1_Token_spec_mint_amount_of_fixed_key"></a>


<pre><code><b>define</b> <a href="Token.md#0x1_Token_spec_mint_amount_of_fixed_key">spec_mint_amount_of_fixed_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_FixedTimeMintKey">FixedTimeMintKey</a>&lt;TokenType&gt;): u128 {
<b>if</b> (<a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() &gt;= key.end_time) {
   key.total
}<b>else</b>{
   0
}
}
</code></pre>



<a name="@Specification_0_destroy_empty_key"></a>

### Function `destroy_empty_key`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_empty_key">destroy_empty_key</a>&lt;TokenType&gt;(key: <a href="Token.md#0x1_Token_LinearTimeMintKey">Token::LinearTimeMintKey</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> key.total != key.minted;
</code></pre>



<a name="@Specification_0_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() - tokens.<a href="Token.md#0x1_Token_value">value</a> &lt; 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Token.md#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_burn_with_capability"></a>

### Function `burn_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="Token.md#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;, tokens: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() - tokens.<a href="Token.md#0x1_Token_value">value</a> &lt; 0;
<b>ensures</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;() ==
        <b>old</b>(<b>global</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>()).total_value) - tokens.value;
</code></pre>



<a name="@Specification_0_zero"></a>

### Function `zero`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<a name="@Specification_0_value"></a>

### Function `value`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_split"></a>

### Function `split`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_split">split</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, value: u128): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.<a href="Token.md#0x1_Token_value">value</a> &lt; value;
<b>ensures</b> <b>old</b>(token.value) == result_1.value + result_2.value;
</code></pre>



<a name="@Specification_0_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, value: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> token.<a href="Token.md#0x1_Token_value">value</a> &lt; value;
<b>ensures</b> result.value == value;
<b>ensures</b> token.value == <b>old</b>(token).value - value;
</code></pre>



<a name="@Specification_0_join"></a>

### Function `join`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_join">join</a>&lt;TokenType&gt;(token1: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, token2: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> token1.value + token2.value &gt; max_u128();
<b>ensures</b> <b>old</b>(token1).value + <b>old</b>(token2).value == result.value;
<b>ensures</b> token1.value + token2.value == result.value;
</code></pre>



<a name="@Specification_0_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, check: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.value + check.value &gt; max_u128();
<b>ensures</b> <b>old</b>(token).value + check.value == token.value;
</code></pre>



<a name="@Specification_0_destroy_zero"></a>

### Function `destroy_zero`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.value &gt; 0;
</code></pre>



<a name="@Specification_0_scaling_factor"></a>

### Function `scaling_factor`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_market_cap"></a>

### Function `market_cap`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_is_registered_in"></a>

### Function `is_registered_in`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_is_same_token"></a>

### Function `is_same_token`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_token_address"></a>

### Function `token_address`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [abstract] <b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(result);
<b>ensures</b> [abstract] result == <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>();
<b>ensures</b> [abstract] <b>global</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(result).total_value == 100000000u128;
</code></pre>



<a name="@Specification_0_token_code"></a>

### Function `token_code`


<pre><code><b>public</b> <b>fun</b> <a href="Token.md#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt;
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [abstract] result == <a href="Token.md#0x1_Token_spec_token_code">spec_token_code</a>&lt;TokenType&gt;();
</code></pre>



<a name="@Specification_0_code_to_bytes"></a>

### Function `code_to_bytes`


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_name_of"></a>

### Function `name_of`


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_name_of_token"></a>

### Function `name_of_token`


<pre><code><b>fun</b> <a href="Token.md#0x1_Token_name_of_token">name_of_token</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma opaque = <b>true</b>;
<b>aborts_if</b> <b>false</b>;
<b>ensures</b> [abstract] <b>exists</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(result_1);
<b>ensures</b> [abstract] result_1 == <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>();
<b>ensures</b> [abstract] <b>global</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(result_1).total_value == 100000000u128;
</code></pre>




<a name="0x1_Token_SPEC_TOKEN_TEST_ADDRESS"></a>


<pre><code><b>define</b> <a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>(): address {
    0x2
}
<a name="0x1_Token_spec_abstract_total_value"></a>
<b>define</b> <a href="Token.md#0x1_Token_spec_abstract_total_value">spec_abstract_total_value</a>&lt;TokenType&gt;(): u128 {
    <b>global</b>&lt;<a href="Token.md#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">SPEC_TOKEN_TEST_ADDRESS</a>()).total_value
}
</code></pre>
