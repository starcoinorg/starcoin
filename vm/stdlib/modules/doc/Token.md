
<a name="0x1_Token"></a>

# Module `0x1::Token`

### Table of Contents

-  [Resource `Token`](#0x1_Token_Token)
-  [Resource `MintCapability`](#0x1_Token_MintCapability)
-  [Resource `BurnCapability`](#0x1_Token_BurnCapability)
-  [Resource `ScalingFactorModifyCapability`](#0x1_Token_ScalingFactorModifyCapability)
-  [Struct `MintEvent`](#0x1_Token_MintEvent)
-  [Struct `BurnEvent`](#0x1_Token_BurnEvent)
-  [Resource `TokenInfo`](#0x1_Token_TokenInfo)
-  [Const `ETOKEN_REGISTER`](#0x1_Token_ETOKEN_REGISTER)
-  [Const `EAMOUNT_EXCEEDS_COIN_VALUE`](#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE)
-  [Function `register_token`](#0x1_Token_register_token)
-  [Function `remove_scaling_factor_modify_capability`](#0x1_Token_remove_scaling_factor_modify_capability)
-  [Function `add_scaling_factor_modify_capability`](#0x1_Token_add_scaling_factor_modify_capability)
-  [Function `destroy_scaling_factor_modify_capability`](#0x1_Token_destroy_scaling_factor_modify_capability)
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
-  [Function `share`](#0x1_Token_share)
-  [Function `split`](#0x1_Token_split)
-  [Function `split_share`](#0x1_Token_split_share)
-  [Function `withdraw`](#0x1_Token_withdraw)
-  [Function `withdraw_share`](#0x1_Token_withdraw_share)
-  [Function `join`](#0x1_Token_join)
-  [Function `deposit`](#0x1_Token_deposit)
-  [Function `destroy_zero`](#0x1_Token_destroy_zero)
-  [Function `amount_to_share`](#0x1_Token_amount_to_share)
-  [Function `share_to_amount`](#0x1_Token_share_to_amount)
-  [Function `scaling_factor`](#0x1_Token_scaling_factor)
-  [Function `base_scaling_factor`](#0x1_Token_base_scaling_factor)
-  [Function `set_scaling_factor`](#0x1_Token_set_scaling_factor)
-  [Function `set_scaling_factor_with_capability`](#0x1_Token_set_scaling_factor_with_capability)
-  [Function `fractional_part`](#0x1_Token_fractional_part)
-  [Function `market_cap`](#0x1_Token_market_cap)
-  [Function `total_share`](#0x1_Token_total_share)
-  [Function `is_registered_in`](#0x1_Token_is_registered_in)
-  [Function `is_same_token`](#0x1_Token_is_same_token)
-  [Function `token_address`](#0x1_Token_token_address)
-  [Function `token_code`](#0x1_Token_token_code)
-  [Function `code_to_bytes`](#0x1_Token_code_to_bytes)
-  [Function `name_of`](#0x1_Token_name_of)
-  [Specification](#0x1_Token_Specification)
    -  [Function `register_token`](#0x1_Token_Specification_register_token)
    -  [Function `remove_scaling_factor_modify_capability`](#0x1_Token_Specification_remove_scaling_factor_modify_capability)
    -  [Function `add_scaling_factor_modify_capability`](#0x1_Token_Specification_add_scaling_factor_modify_capability)
    -  [Function `destroy_scaling_factor_modify_capability`](#0x1_Token_Specification_destroy_scaling_factor_modify_capability)
    -  [Function `remove_mint_capability`](#0x1_Token_Specification_remove_mint_capability)
    -  [Function `add_mint_capability`](#0x1_Token_Specification_add_mint_capability)
    -  [Function `destroy_mint_capability`](#0x1_Token_Specification_destroy_mint_capability)
    -  [Function `remove_burn_capability`](#0x1_Token_Specification_remove_burn_capability)
    -  [Function `add_burn_capability`](#0x1_Token_Specification_add_burn_capability)
    -  [Function `destroy_burn_capability`](#0x1_Token_Specification_destroy_burn_capability)
    -  [Function `mint`](#0x1_Token_Specification_mint)
    -  [Function `mint_with_capability`](#0x1_Token_Specification_mint_with_capability)
    -  [Function `burn`](#0x1_Token_Specification_burn)
    -  [Function `burn_with_capability`](#0x1_Token_Specification_burn_with_capability)
    -  [Function `zero`](#0x1_Token_Specification_zero)
    -  [Function `value`](#0x1_Token_Specification_value)
    -  [Function `split`](#0x1_Token_Specification_split)
    -  [Function `split_share`](#0x1_Token_Specification_split_share)
    -  [Function `withdraw`](#0x1_Token_Specification_withdraw)
    -  [Function `withdraw_share`](#0x1_Token_Specification_withdraw_share)
    -  [Function `join`](#0x1_Token_Specification_join)
    -  [Function `deposit`](#0x1_Token_Specification_deposit)
    -  [Function `destroy_zero`](#0x1_Token_Specification_destroy_zero)
    -  [Function `amount_to_share`](#0x1_Token_Specification_amount_to_share)
    -  [Function `share_to_amount`](#0x1_Token_Specification_share_to_amount)
    -  [Function `scaling_factor`](#0x1_Token_Specification_scaling_factor)
    -  [Function `base_scaling_factor`](#0x1_Token_Specification_base_scaling_factor)
    -  [Function `set_scaling_factor`](#0x1_Token_Specification_set_scaling_factor)
    -  [Function `set_scaling_factor_with_capability`](#0x1_Token_Specification_set_scaling_factor_with_capability)
    -  [Function `fractional_part`](#0x1_Token_Specification_fractional_part)
    -  [Function `market_cap`](#0x1_Token_Specification_market_cap)
    -  [Function `total_share`](#0x1_Token_Specification_total_share)
    -  [Function `is_registered_in`](#0x1_Token_Specification_is_registered_in)
    -  [Function `is_same_token`](#0x1_Token_Specification_is_same_token)
    -  [Function `token_address`](#0x1_Token_Specification_token_address)
    -  [Function `token_code`](#0x1_Token_Specification_token_code)
    -  [Function `code_to_bytes`](#0x1_Token_Specification_code_to_bytes)
    -  [Function `name_of`](#0x1_Token_Specification_name_of)



<a name="0x1_Token_Token"></a>

## Resource `Token`

The token has a <code>TokenType</code> color that tells us what token the
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

A minting capability allows tokens of type <code>TokenType</code> to be minted


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

<a name="0x1_Token_ScalingFactorModifyCapability"></a>

## Resource `ScalingFactorModifyCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;
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
<code>base_scaling_factor: u128</code>
</dt>
<dd>

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

<a name="0x1_Token_ETOKEN_REGISTER"></a>

## Const `ETOKEN_REGISTER`

Token register's address should same as TokenType's address.


<pre><code><b>const</b> <a href="#0x1_Token_ETOKEN_REGISTER">ETOKEN_REGISTER</a>: u64 = 100;
</code></pre>



<a name="0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE"></a>

## Const `EAMOUNT_EXCEEDS_COIN_VALUE`



<pre><code><b>const</b> <a href="#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>: u64 = 102;
</code></pre>



<a name="0x1_Token_register_token"></a>

## Function `register_token`

Register the type <code>TokenType</code> as a Token and got MintCapability and BurnCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(account: &signer, base_scaling_factor: u128, fractional_part: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(
    account: &signer,
    base_scaling_factor: u128,
    fractional_part: u128,
) {
    <b>let</b> (token_address, _module_name, _token_name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == token_address, <a href="#0x1_Token_ETOKEN_REGISTER">ETOKEN_REGISTER</a>);
    // <b>assert</b>(module_name == token_name, ETOKEN_NAME);
    move_to(account, <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; {});
    move_to(account, <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; {});
    move_to(account, <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt; {});
    move_to(
        account,
        <a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt; {
            total_value: 0,
            scaling_factor: base_scaling_factor,
            base_scaling_factor,
            fractional_part,
            mint_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Token_MintEvent">MintEvent</a>&gt;(account),
            burn_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="#0x1_Token_BurnEvent">BurnEvent</a>&gt;(account),
        },
    );
}
</code></pre>



</details>

<a name="0x1_Token_remove_scaling_factor_modify_capability"></a>

## Function `remove_scaling_factor_modify_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_scaling_factor_modify_capability">remove_scaling_factor_modify_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_scaling_factor_modify_capability">remove_scaling_factor_modify_capability</a>&lt;TokenType&gt;(
    signer: &signer,
): <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a> {
    move_from&lt;<a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Token_add_scaling_factor_modify_capability"></a>

## Function `add_scaling_factor_modify_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_scaling_factor_modify_capability">add_scaling_factor_modify_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_scaling_factor_modify_capability">add_scaling_factor_modify_capability</a>&lt;TokenType&gt;(
    signer: &signer,
    cap: <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;,
) {
    move_to&lt;<a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;&gt;(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Token_destroy_scaling_factor_modify_capability"></a>

## Function `destroy_scaling_factor_modify_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_scaling_factor_modify_capability">destroy_scaling_factor_modify_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_scaling_factor_modify_capability">destroy_scaling_factor_modify_capability</a>&lt;TokenType&gt;(
    cap: <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;,
) {
    <b>let</b> <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_remove_mint_capability"></a>

## Function `remove_mint_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;
<b>acquires</b> <a href="#0x1_Token_MintCapability">MintCapability</a> {
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;) {
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
    <b>let</b> <a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_remove_burn_capability"></a>

## Function `remove_burn_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;
<b>acquires</b> <a href="#0x1_Token_BurnCapability">BurnCapability</a> {
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;) {
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
    <b>let</b> <a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Token_mint"></a>

## Function `mint`

Return <code>amount</code> tokens.
Fails if the sender does not have a published MintCapability.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>, <a href="#0x1_Token_MintCapability">MintCapability</a> {
    <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>(
        borrow_global&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
        amount,
    )
}
</code></pre>



</details>

<a name="0x1_Token_mint_with_capability"></a>

## Function `mint_with_capability`

Mint a new Token::Token worth <code>amount</code> considering current <code>scaling_factor</code>. The caller must have a reference to a MintCapability.
Only the Association account can acquire such a reference, and it can do so only via
<code>borrow_sender_mint_capability</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(
    _capability: &<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;,
    amount: u128,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    // <b>update</b> market cap <b>resource</b> <b>to</b> reflect minting
    <b>let</b> (token_address, module_name, token_name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    <b>let</b> share = <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount);
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    info.total_value = info.total_value + (share <b>as</b> u128);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.mint_events,
        <a href="#0x1_Token_MintEvent">MintEvent</a> {
            amount: share,
            token_code: <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        },
    );
    <a href="#0x1_Token">Token</a>&lt;TokenType&gt; { value: share }
}
</code></pre>



</details>

<a name="0x1_Token_burn"></a>

## Function `burn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;)
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>, <a href="#0x1_Token_BurnCapability">BurnCapability</a> {
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
    <b>let</b> <a href="#0x1_Token">Token</a> { value } = tokens;
    info.total_value = info.total_value - (value <b>as</b> u128);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> info.burn_events,
        <a href="#0x1_Token_BurnEvent">BurnEvent</a> {
            amount: value,
            token_code: <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(token_address, module_name, token_name),
        },
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

Scaled value of the token considering the <code>scaling_factor</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token">Token</a>&lt;TokenType&gt;): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <a href="#0x1_Token_share_to_amount">share_to_amount</a>&lt;TokenType&gt;(<a href="#0x1_Token_share">share</a>(token))
}
</code></pre>



</details>

<a name="0x1_Token_share"></a>

## Function `share`

Public accessor for the value of a token


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_share">share</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_share">share</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token">Token</a>&lt;TokenType&gt;): u128 {
    token.value
}
</code></pre>



</details>

<a name="0x1_Token_split"></a>

## Function `split`

Splits the given token into two and returns them both
It leverages <code><a href="#0x1_Token_split_share">Self::split_share</a></code> for any verifications of the values


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split">split</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): (<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split">split</a>&lt;TokenType&gt;(
    token: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    amount: u128,
): (<a href="#0x1_Token">Token</a>&lt;TokenType&gt;, <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <a href="#0x1_Token_split_share">split_share</a>&lt;TokenType&gt;(token, <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount))
}
</code></pre>



</details>

<a name="0x1_Token_split_share"></a>

## Function `split_share`

Splits the given token into two and returns them both
It leverages <code><a href="#0x1_Token_withdraw_share">Self::withdraw_share</a></code> for any verifications of the values.
It operates on token value directly regardless of the <code>scaling_factor</code> of the token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split_share">split_share</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, share: u128): (<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split_share">split_share</a>&lt;TokenType&gt;(
    token: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    share: u128,
): (<a href="#0x1_Token">Token</a>&lt;TokenType&gt;, <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> other = <a href="#0x1_Token_withdraw_share">withdraw_share</a>(&<b>mut</b> token, share);
    (token, other)
}
</code></pre>



</details>

<a name="0x1_Token_withdraw"></a>

## Function `withdraw`

"Divides" the given token into two, where the original token is modified in place.
This will consider the scaling_factor of the <code><a href="#0x1_Token">Token</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token">Token</a>&lt;TokenType&gt;
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <a href="#0x1_Token_withdraw_share">withdraw_share</a>&lt;TokenType&gt;(token, <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount))
}
</code></pre>



</details>

<a name="0x1_Token_withdraw_share"></a>

## Function `withdraw_share`

It operates on token value directly regardless of the <code>scaling_factor</code> of the token.
The original token will have value = original value - <code>share</code>
The new token will have a value = <code>share</code>
Fails if the tokens value is less than <code>share</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw_share">withdraw_share</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, share: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw_share">withdraw_share</a>&lt;TokenType&gt;(
    token: &<b>mut</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;,
    share: u128,
): <a href="#0x1_Token">Token</a>&lt;TokenType&gt; {
    // Check that `share` is less than the token's value
    <b>assert</b>(token.value &gt;= share, <a href="#0x1_Token_EAMOUNT_EXCEEDS_COIN_VALUE">EAMOUNT_EXCEEDS_COIN_VALUE</a>);
    token.value = token.value - share;
    <a href="#0x1_Token">Token</a> { value: share }
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
The <code>check</code> token is consumed in the process


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, check: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token">Token</a>&lt;TokenType&gt;, check: <a href="#0x1_Token">Token</a>&lt;TokenType&gt;) {
    <b>let</b> <a href="#0x1_Token">Token</a> { value } = check;
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
    <b>let</b> <a href="#0x1_Token">Token</a> { value } = token;
    <b>assert</b>(value == 0, <a href="ErrorCode.md#0x1_ErrorCode_EDESTORY_TOKEN_NON_ZERO">ErrorCode::EDESTORY_TOKEN_NON_ZERO</a>())
}
</code></pre>



</details>

<a name="0x1_Token_amount_to_share"></a>

## Function `amount_to_share`

convenient function to calculate hold of the input <code>amount</code> based the current scaling_factor.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount: u128): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> base = <a href="#0x1_Token_base_scaling_factor">base_scaling_factor</a>&lt;TokenType&gt;();
    <b>let</b> scaled = <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;();
    // shortcut <b>to</b> avoid bignumber cal.
    <b>if</b> (base == scaled) {
        amount
    } <b>else</b> {
        amount * base / scaled
    }
}
</code></pre>



</details>

<a name="0x1_Token_share_to_amount"></a>

## Function `share_to_amount`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_share_to_amount">share_to_amount</a>&lt;TokenType&gt;(hold: u128): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_share_to_amount">share_to_amount</a>&lt;TokenType&gt;(hold: u128): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> base = <a href="#0x1_Token_base_scaling_factor">base_scaling_factor</a>&lt;TokenType&gt;();
    <b>let</b> scaled = <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;();
    <b>if</b> (base == scaled) {
        hold
    } <b>else</b> {
        hold * scaled / base
    }
}
</code></pre>



</details>

<a name="0x1_Token_scaling_factor"></a>

## Function `scaling_factor`

Returns the scaling factor for the <code>TokenType</code> token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).scaling_factor
}
</code></pre>



</details>

<a name="0x1_Token_base_scaling_factor"></a>

## Function `base_scaling_factor`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_base_scaling_factor">base_scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_base_scaling_factor">base_scaling_factor</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).base_scaling_factor
}
</code></pre>



</details>

<a name="0x1_Token_set_scaling_factor"></a>

## Function `set_scaling_factor`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor">set_scaling_factor</a>&lt;TokenType&gt;(signer: &signer, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor">set_scaling_factor</a>&lt;TokenType&gt;(signer: &signer, value: u128)
<b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a>, <a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a> {
    <b>let</b> cap = borrow_global&lt;<a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;&gt;(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer),
    );
    <a href="#0x1_Token_set_scaling_factor_with_capability">set_scaling_factor_with_capability</a>(cap, value)
}
</code></pre>



</details>

<a name="0x1_Token_set_scaling_factor_with_capability"></a>

## Function `set_scaling_factor_with_capability`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor_with_capability">set_scaling_factor_with_capability</a>&lt;TokenType&gt;(_cap: &<a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor_with_capability">set_scaling_factor_with_capability</a>&lt;TokenType&gt;(
    _cap: &<a href="#0x1_Token_ScalingFactorModifyCapability">ScalingFactorModifyCapability</a>&lt;TokenType&gt;,
    value: u128,
) <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> token_address = <a href="#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;();
    <b>let</b> info = borrow_global_mut&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address);
    info.scaling_factor = value;

    // TODO: emit event
}
</code></pre>



</details>

<a name="0x1_Token_fractional_part"></a>

## Function `fractional_part`

Returns the representable fractional part for the <code>TokenType</code> token.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_fractional_part">fractional_part</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_fractional_part">fractional_part</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).fractional_part
}
</code></pre>



</details>

<a name="0x1_Token_market_cap"></a>

## Function `market_cap`

Return the total amount of token of type <code>TokenType</code> considering current <code>scaling_factor</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <a href="#0x1_Token_share_to_amount">share_to_amount</a>&lt;TokenType&gt;(<a href="#0x1_Token_total_share">total_share</a>&lt;TokenType&gt;())
}
</code></pre>



</details>

<a name="0x1_Token_total_share"></a>

## Function `total_share`

Return the total share of token minted.


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_total_share">total_share</a>&lt;TokenType&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_total_share">total_share</a>&lt;TokenType&gt;(): u128 <b>acquires</b> <a href="#0x1_Token_TokenInfo">TokenInfo</a> {
    <b>let</b> (token_address, _, _) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
    borrow_global&lt;<a href="#0x1_Token_TokenInfo">TokenInfo</a>&lt;TokenType&gt;&gt;(token_address).total_value
}
</code></pre>



</details>

<a name="0x1_Token_is_registered_in"></a>

## Function `is_registered_in`

Return true if the type <code>TokenType</code> is a registered in <code>token_address</code>.


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

Return true if the type <code>TokenType1</code> is same with <code>TokenType2</code>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool {
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


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address {
    <b>let</b> (addr, _, _) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
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
    <b>let</b> (addr, module_name, name) = <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;();
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

Return Token's module address, module name, and type name of <code>TokenType</code>.


<pre><code><b>fun</b> <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;);
</code></pre>



</details>

<a name="0x1_Token_Specification"></a>

## Specification



<pre><code>pragma verify = <b>true</b>;
pragma aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="0x1_Token_Specification_register_token"></a>

### Function `register_token`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_register_token">register_token</a>&lt;TokenType&gt;(account: &signer, base_scaling_factor: u128, fractional_part: u128)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_remove_scaling_factor_modify_capability"></a>

### Function `remove_scaling_factor_modify_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_scaling_factor_modify_capability">remove_scaling_factor_modify_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_add_scaling_factor_modify_capability"></a>

### Function `add_scaling_factor_modify_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_scaling_factor_modify_capability">add_scaling_factor_modify_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_destroy_scaling_factor_modify_capability"></a>

### Function `destroy_scaling_factor_modify_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_scaling_factor_modify_capability">destroy_scaling_factor_modify_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_remove_mint_capability"></a>

### Function `remove_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_mint_capability">remove_mint_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> !exists&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="0x1_Token_Specification_add_mint_capability"></a>

### Function `add_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_mint_capability">add_mint_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> exists&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="0x1_Token_Specification_destroy_mint_capability"></a>

### Function `destroy_mint_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_mint_capability">destroy_mint_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;)
</code></pre>




<a name="0x1_Token_Specification_remove_burn_capability"></a>

### Function `remove_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_remove_burn_capability">remove_burn_capability</a>&lt;TokenType&gt;(signer: &signer): <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> !exists&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="0x1_Token_Specification_add_burn_capability"></a>

### Function `add_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_add_burn_capability">add_burn_capability</a>&lt;TokenType&gt;(signer: &signer, cap: <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> exists&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> exists&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="0x1_Token_Specification_destroy_burn_capability"></a>

### Function `destroy_burn_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_burn_capability">destroy_burn_capability</a>&lt;TokenType&gt;(cap: <a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;)
</code></pre>




<a name="0x1_Token_Specification_mint"></a>

### Function `mint`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint">mint</a>&lt;TokenType&gt;(account: &signer, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Token_MintCapability">MintCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="0x1_Token_Specification_mint_with_capability"></a>

### Function `mint_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_mint_with_capability">mint_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_burn"></a>

### Function `burn`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn">burn</a>&lt;TokenType&gt;(account: &signer, tokens: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !exists&lt;<a href="#0x1_Token_BurnCapability">BurnCapability</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="0x1_Token_Specification_burn_with_capability"></a>

### Function `burn_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_burn_with_capability">burn_with_capability</a>&lt;TokenType&gt;(_capability: &<a href="#0x1_Token_BurnCapability">Token::BurnCapability</a>&lt;TokenType&gt;, tokens: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_zero"></a>

### Function `zero`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_zero">zero</a>&lt;TokenType&gt;(): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<a name="0x1_Token_Specification_value"></a>

### Function `value`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_value">value</a>&lt;TokenType&gt;(token: &<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_split"></a>

### Function `split`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split">split</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): (<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_split_share"></a>

### Function `split_share`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_split_share">split_share</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, share: u128): (<a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.<a href="#0x1_Token_value">value</a> &lt; share;
</code></pre>



<a name="0x1_Token_Specification_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw">withdraw</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, amount: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_withdraw_share"></a>

### Function `withdraw_share`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_withdraw_share">withdraw_share</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, share: u128): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> token.<a href="#0x1_Token_value">value</a> &lt; share;
<b>ensures</b> result.value == share;
<b>ensures</b> token.value == <b>old</b>(token).value - share;
</code></pre>



<a name="0x1_Token_Specification_join"></a>

### Function `join`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_join">join</a>&lt;TokenType&gt;(token1: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, token2: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;): <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>aborts_if</b> token1.value + token2.value &gt; max_u128();
<b>ensures</b> <b>old</b>(token1).value + <b>old</b>(token2).value == result.value;
<b>ensures</b> token1.value + token2.value == result.value;
</code></pre>



<a name="0x1_Token_Specification_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_deposit">deposit</a>&lt;TokenType&gt;(token: &<b>mut</b> <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, check: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.value + check.value &gt; max_u128();
<b>ensures</b> <b>old</b>(token).value + check.value == token.value;
</code></pre>



<a name="0x1_Token_Specification_destroy_zero"></a>

### Function `destroy_zero`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_destroy_zero">destroy_zero</a>&lt;TokenType&gt;(token: <a href="#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> token.value &gt; 0;
</code></pre>



<a name="0x1_Token_Specification_amount_to_share"></a>

### Function `amount_to_share`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_amount_to_share">amount_to_share</a>&lt;TokenType&gt;(amount: u128): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_share_to_amount"></a>

### Function `share_to_amount`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_share_to_amount">share_to_amount</a>&lt;TokenType&gt;(hold: u128): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_scaling_factor"></a>

### Function `scaling_factor`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_scaling_factor">scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_base_scaling_factor"></a>

### Function `base_scaling_factor`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_base_scaling_factor">base_scaling_factor</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_set_scaling_factor"></a>

### Function `set_scaling_factor`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor">set_scaling_factor</a>&lt;TokenType&gt;(signer: &signer, value: u128)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_set_scaling_factor_with_capability"></a>

### Function `set_scaling_factor_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_set_scaling_factor_with_capability">set_scaling_factor_with_capability</a>&lt;TokenType&gt;(_cap: &<a href="#0x1_Token_ScalingFactorModifyCapability">Token::ScalingFactorModifyCapability</a>&lt;TokenType&gt;, value: u128)
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_fractional_part"></a>

### Function `fractional_part`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_fractional_part">fractional_part</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_market_cap"></a>

### Function `market_cap`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_market_cap">market_cap</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_total_share"></a>

### Function `total_share`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_total_share">total_share</a>&lt;TokenType&gt;(): u128
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_is_registered_in"></a>

### Function `is_registered_in`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_registered_in">is_registered_in</a>&lt;TokenType&gt;(token_address: address): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_is_same_token"></a>

### Function `is_same_token`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_is_same_token">is_same_token</a>&lt;TokenType1, TokenType2&gt;(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_token_address"></a>

### Function `token_address`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_address">token_address</a>&lt;TokenType&gt;(): address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_token_code"></a>

### Function `token_code`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Token_token_code">token_code</a>&lt;TokenType&gt;(): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_code_to_bytes"></a>

### Function `code_to_bytes`


<pre><code><b>fun</b> <a href="#0x1_Token_code_to_bytes">code_to_bytes</a>(addr: address, module_name: vector&lt;u8&gt;, name: vector&lt;u8&gt;): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="0x1_Token_Specification_name_of"></a>

### Function `name_of`


<pre><code><b>fun</b> <a href="#0x1_Token_name_of">name_of</a>&lt;TokenType&gt;(): (address, vector&lt;u8&gt;, vector&lt;u8&gt;)
</code></pre>




<pre><code>pragma intrinsic = <b>true</b>;
</code></pre>
