
<a name="0x1_Treasury"></a>

# Module `0x1::Treasury`

The module for the Treasury of DAO, which can hold the token of DAO.


-  [Resource `Treasury`](#0x1_Treasury_Treasury)
-  [Resource `WithdrawCapability`](#0x1_Treasury_WithdrawCapability)
-  [Resource `LinearTimeWithdrawCapability`](#0x1_Treasury_LinearTimeWithdrawCapability)
-  [Struct `WithdrawEvent`](#0x1_Treasury_WithdrawEvent)
-  [Struct `DepositEvent`](#0x1_Treasury_DepositEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_Treasury_initialize)
-  [Function `exists_at`](#0x1_Treasury_exists_at)
-  [Function `deposit`](#0x1_Treasury_deposit)
-  [Function `do_withdraw`](#0x1_Treasury_do_withdraw)
-  [Function `withdraw_with_cap`](#0x1_Treasury_withdraw_with_cap)
-  [Function `withdraw`](#0x1_Treasury_withdraw)
-  [Function `issue_linear_withdraw_capability`](#0x1_Treasury_issue_linear_withdraw_capability)
-  [Function `withdraw_with_linear_cap`](#0x1_Treasury_withdraw_with_linear_cap)
-  [Function `withdraw_by_linear`](#0x1_Treasury_withdraw_by_linear)
-  [Function `split_linear_withdraw_cap`](#0x1_Treasury_split_linear_withdraw_cap)
-  [Function `withdraw_amount_of_linear_cap`](#0x1_Treasury_withdraw_amount_of_linear_cap)
-  [Function `is_empty_linear_withdraw_cap`](#0x1_Treasury_is_empty_linear_withdraw_cap)
-  [Function `remove_withdraw_capability`](#0x1_Treasury_remove_withdraw_capability)
-  [Function `add_withdraw_capability`](#0x1_Treasury_add_withdraw_capability)
-  [Function `destroy_withdraw_capability`](#0x1_Treasury_destroy_withdraw_capability)
-  [Function `add_linear_withdraw_capability`](#0x1_Treasury_add_linear_withdraw_capability)
-  [Function `remove_liner_withdraw_capability`](#0x1_Treasury_remove_liner_withdraw_capability)
-  [Function `destroy_liner_withdraw_capability`](#0x1_Treasury_destroy_liner_withdraw_capability)
-  [Specification](#@Specification_1)
    -  [Function `do_withdraw`](#@Specification_1_do_withdraw)
    -  [Function `issue_linear_withdraw_capability`](#@Specification_1_issue_linear_withdraw_capability)
    -  [Function `withdraw_with_linear_cap`](#@Specification_1_withdraw_with_linear_cap)
    -  [Function `split_linear_withdraw_cap`](#@Specification_1_split_linear_withdraw_cap)
    -  [Function `withdraw_amount_of_linear_cap`](#@Specification_1_withdraw_amount_of_linear_cap)
    -  [Function `is_empty_linear_withdraw_cap`](#@Specification_1_is_empty_linear_withdraw_cap)
    -  [Function `remove_withdraw_capability`](#@Specification_1_remove_withdraw_capability)
    -  [Function `add_withdraw_capability`](#@Specification_1_add_withdraw_capability)
    -  [Function `destroy_withdraw_capability`](#@Specification_1_destroy_withdraw_capability)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Math.md#0x1_Math">0x1::Math</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_Treasury_Treasury"></a>

## Resource `Treasury`



<pre><code><b>resource</b> <b>struct</b> <a href="Treasury.md#0x1_Treasury">Treasury</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>balance: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>withdraw_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawEvent">Treasury::WithdrawEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury withdraw event
</dd>
<dt>
<code>deposit_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Treasury.md#0x1_Treasury_DepositEvent">Treasury::DepositEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury deposit event
</dd>
</dl>


</details>

<a name="0x1_Treasury_WithdrawCapability"></a>

## Resource `WithdrawCapability`

A withdraw capability allows tokens of type <code>TokenT</code> to be withdraw from Treasury.


<pre><code><b>resource</b> <b>struct</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;
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

<a name="0x1_Treasury_LinearTimeWithdrawCapability"></a>

## Resource `LinearTimeWithdrawCapability`

A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.


<pre><code><b>resource</b> <b>struct</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;
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
<code>withdraw: u128</code>
</dt>
<dd>

</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>period: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Treasury_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Message for treasury withdraw event.


<pre><code><b>struct</b> <a href="Treasury.md#0x1_Treasury_WithdrawEvent">WithdrawEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Treasury_DepositEvent"></a>

## Struct `DepositEvent`

Message for treasury deposit event.


<pre><code><b>struct</b> <a href="Treasury.md#0x1_Treasury_DepositEvent">DepositEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>amount: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Treasury_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="Treasury.md#0x1_Treasury_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 3;
</code></pre>



<a name="0x1_Treasury_ERR_INVALID_AMOUNT"></a>



<pre><code><b>const</b> <a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>: u64 = 2;
</code></pre>



<a name="0x1_Treasury_ERR_INVALID_PERIOD"></a>



<pre><code><b>const</b> <a href="Treasury.md#0x1_Treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>: u64 = 1;
</code></pre>



<a name="0x1_Treasury_ERR_TREASURY_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="Treasury.md#0x1_Treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>: u64 = 4;
</code></pre>



<a name="0x1_Treasury_initialize"></a>

## Function `initialize`

Init a Treasury for TokenT,can only be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_initialize">initialize</a>&lt;TokenT&gt;(signer: &signer, init_token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_initialize">initialize</a>&lt;TokenT:store&gt;(signer: &signer, init_token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt;) :<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Treasury.md#0x1_Treasury_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> treasure = <a href="Treasury.md#0x1_Treasury">Treasury</a>{
        balance: init_token,
        withdraw_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawEvent">WithdrawEvent</a>&gt;(signer),
        deposit_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Treasury.md#0x1_Treasury_DepositEvent">DepositEvent</a>&gt;(signer),
    };
    move_to(signer,treasure);
    <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;{}
}
</code></pre>



</details>

<a name="0x1_Treasury_exists_at"></a>

## Function `exists_at`

Check the Treasury of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_exists_at">exists_at</a>&lt;TokenT&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_exists_at">exists_at</a>&lt;TokenT:store&gt;(): bool {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a name="0x1_Treasury_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_deposit">deposit</a>&lt;TokenT&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_deposit">deposit</a>&lt;TokenT:store&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt;) <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a>{
    <b>assert</b>(<a href="Treasury.md#0x1_Treasury_exists_at">exists_at</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Treasury.md#0x1_Treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));
    <b>let</b> token_address = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>let</b> treasury = borrow_global_mut&lt;<a href="Treasury.md#0x1_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_address);
    <b>let</b> amount = <a href="Token.md#0x1_Token_value">Token::value</a>(&token);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
            &<b>mut</b> treasury.deposit_events,
            <a href="Treasury.md#0x1_Treasury_DepositEvent">DepositEvent</a> {
                amount,
            },
        );
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> treasury.balance, token);
}
</code></pre>



</details>

<a name="0x1_Treasury_do_withdraw"></a>

## Function `do_withdraw`



<pre><code><b>fun</b> <a href="Treasury.md#0x1_Treasury_do_withdraw">do_withdraw</a>&lt;TokenT&gt;(amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Treasury.md#0x1_Treasury_do_withdraw">do_withdraw</a>&lt;TokenT:store&gt;(amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a> {
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>));
    <b>assert</b>(<a href="Treasury.md#0x1_Treasury_exists_at">exists_at</a>&lt;TokenT&gt;(), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Treasury.md#0x1_Treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));
    <b>let</b> token_address = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>let</b> treasury = borrow_global_mut&lt;<a href="Treasury.md#0x1_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_address);
    <b>assert</b>(amount &lt;= <a href="Token.md#0x1_Token_value">Token::value</a>(&treasury.balance) , <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>));
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(
        &<b>mut</b> treasury.withdraw_events,
        <a href="Treasury.md#0x1_Treasury_WithdrawEvent">WithdrawEvent</a> {
            amount,
        },
    );
    <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> treasury.balance, amount)
}
</code></pre>



</details>

<a name="0x1_Treasury_withdraw_with_cap"></a>

## Function `withdraw_with_cap`

Withdraw tokens with given <code><a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_with_cap">withdraw_with_cap</a>&lt;TokenT&gt;(_cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_with_cap">withdraw_with_cap</a>&lt;TokenT:store&gt;(_cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a> {
    <b>let</b> token = <a href="Treasury.md#0x1_Treasury_do_withdraw">do_withdraw</a>(amount);
    token
}
</code></pre>



</details>

<a name="0x1_Treasury_withdraw"></a>

## Function `withdraw`

Withdraw from TokenT's  treasury, the signer must have WithdrawCapability<TokenT>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw">withdraw</a>&lt;TokenT&gt;(signer: &signer, amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw">withdraw</a>&lt;TokenT:store&gt;(signer: &signer, amount: u128) : <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a>, <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>{
    <b>let</b> cap = borrow_global_mut&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <a href="Treasury.md#0x1_Treasury_withdraw_with_cap">Self::withdraw_with_cap</a>(cap, amount)
}
</code></pre>



</details>

<a name="0x1_Treasury_issue_linear_withdraw_capability"></a>

## Function `issue_linear_withdraw_capability`

Issue a <code><a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a></code> with given <code><a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(_capability: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT: store&gt;( _capability: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;,
                                            amount: u128, period: u64): <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;{
    <b>assert</b>(period &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>));
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>));
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt; {
        total: amount,
        withdraw: 0,
        start_time,
        period
    }
}
</code></pre>



</details>

<a name="0x1_Treasury_withdraw_with_linear_cap"></a>

## Function `withdraw_with_linear_cap`

Withdraw tokens with given <code><a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_cap">withdraw_with_linear_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_cap">withdraw_with_linear_cap</a>&lt;TokenT: store&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a> {
    <b>let</b> amount = <a href="Treasury.md#0x1_Treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>(cap);
    <b>let</b> token = <a href="Treasury.md#0x1_Treasury_do_withdraw">do_withdraw</a>(amount);
    cap.withdraw = cap.withdraw + amount;
    token
}
</code></pre>



</details>

<a name="0x1_Treasury_withdraw_by_linear"></a>

## Function `withdraw_by_linear`

Withdraw from TokenT's  treasury, the signer must have LinearTimeWithdrawCapability<TokenT>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_by_linear">withdraw_by_linear</a>&lt;TokenT&gt;(signer: &signer): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_by_linear">withdraw_by_linear</a>&lt;TokenT:store&gt;(signer: &signer) : <a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt; <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a>, <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>{
    <b>let</b> cap = borrow_global_mut&lt;<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer));
    <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_cap">Self::withdraw_with_linear_cap</a>(cap)
}
</code></pre>



</details>

<a name="0x1_Treasury_split_linear_withdraw_cap"></a>

## Function `split_linear_withdraw_cap`

Split the given <code><a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT: store&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="Token.md#0x1_Token">Token</a>&lt;TokenT&gt;, <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;) <b>acquires</b> <a href="Treasury.md#0x1_Treasury">Treasury</a> {
    <b>assert</b>(amount &gt; 0, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>));
    <b>let</b> token = <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_cap">Self::withdraw_with_linear_cap</a>(cap);
    <b>assert</b>((cap.withdraw + amount) &lt;= cap.total, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Treasury.md#0x1_Treasury_ERR_INVALID_AMOUNT">ERR_INVALID_AMOUNT</a>));
    cap.total = cap.total - amount;
    <b>let</b> start_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> new_period = cap.start_time + cap.period - start_time;
    <b>let</b> new_key = <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt; {
        total: amount,
        withdraw: 0,
        start_time,
        period: new_period
    };
    (token, new_key)
}
</code></pre>



</details>

<a name="0x1_Treasury_withdraw_amount_of_linear_cap"></a>

## Function `withdraw_amount_of_linear_cap`

Returns the amount of the LinearTimeWithdrawCapability can mint now.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT&gt;(cap: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT: store&gt;(cap: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): u128 {
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - cap.start_time;
    <b>if</b> (elapsed_time &gt;= cap.period) {
        cap.total - cap.withdraw
    }<b>else</b> {
        <a href="Math.md#0x1_Math_mul_div">Math::mul_div</a>(cap.total, (elapsed_time <b>as</b> u128), (cap.period <b>as</b> u128)) - cap.withdraw
    }
}
</code></pre>



</details>

<a name="0x1_Treasury_is_empty_linear_withdraw_cap"></a>

## Function `is_empty_linear_withdraw_cap`

Check if the given <code><a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a></code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT&gt;(key: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT:store&gt;(key: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;) : bool {
    key.total == key.withdraw
}
</code></pre>



</details>

<a name="0x1_Treasury_remove_withdraw_capability"></a>

## Function `remove_withdraw_capability`

Remove mint capability from <code>signer</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT: store&gt;(signer: &signer): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;
<b>acquires</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a> {
    move_from&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Treasury_add_withdraw_capability"></a>

## Function `add_withdraw_capability`

Save mint capability to <code>signer</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;) {
    move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Treasury_destroy_withdraw_capability"></a>

## Function `destroy_withdraw_capability`

Destroy the given mint capability.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT: store&gt;(cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; { } = cap;
}
</code></pre>



</details>

<a name="0x1_Treasury_add_linear_withdraw_capability"></a>

## Function `add_linear_withdraw_capability`

Add LinearTimeWithdrawCapability to <code>signer</code>, a address only can have one LinearTimeWithdrawCapability<T>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">add_linear_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">add_linear_withdraw_capability</a>&lt;TokenT: store&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;){
    move_to(signer, cap)
}
</code></pre>



</details>

<a name="0x1_Treasury_remove_liner_withdraw_capability"></a>

## Function `remove_liner_withdraw_capability`

Remove LinearTimeWithdrawCapability from <code>signer</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_remove_liner_withdraw_capability">remove_liner_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer): <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_remove_liner_withdraw_capability">remove_liner_withdraw_capability</a>&lt;TokenT: store&gt;(signer: &signer): <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;
<b>acquires</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a> {
    move_from&lt;<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer))
}
</code></pre>



</details>

<a name="0x1_Treasury_destroy_liner_withdraw_capability"></a>

## Function `destroy_liner_withdraw_capability`

Destroy LinearTimeWithdrawCapability.


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_destroy_liner_withdraw_capability">destroy_liner_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_destroy_liner_withdraw_capability">destroy_liner_withdraw_capability</a>&lt;TokenT: store&gt;(cap: <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">LinearTimeWithdrawCapability</a>{ total: _, withdraw: _, start_time: _, period: _ } = cap;
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_do_withdraw"></a>

### Function `do_withdraw`


<pre><code><b>fun</b> <a href="Treasury.md#0x1_Treasury_do_withdraw">do_withdraw</a>&lt;TokenT&gt;(amount: u128): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_SPEC_TOKEN_TEST_ADDRESS">Token::SPEC_TOKEN_TEST_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_issue_linear_withdraw_capability"></a>

### Function `issue_linear_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(_capability: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> period == 0;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_withdraw_with_linear_cap"></a>

### Function `withdraw_with_linear_cap`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_cap">withdraw_with_linear_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_split_linear_withdraw_cap"></a>

### Function `split_linear_withdraw_cap`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenT&gt;, <a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_withdraw_amount_of_linear_cap"></a>

### Function `withdraw_amount_of_linear_cap`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT&gt;(cap: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">0x1::CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() &lt; cap.start_time;
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() - cap.start_time &gt;= cap.period && cap.total &lt; cap.withdraw;
<b>aborts_if</b> [abstract] <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">Timestamp::spec_now_seconds</a>() - cap.start_time &lt; cap.period && <a href="Math.md#0x1_Math_spec_mul_div">Math::spec_mul_div</a>() &lt; cap.withdraw;
</code></pre>



<a name="@Specification_1_is_empty_linear_withdraw_cap"></a>

### Function `is_empty_linear_withdraw_cap`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT&gt;(key: &<a href="Treasury.md#0x1_Treasury_LinearTimeWithdrawCapability">Treasury::LinearTimeWithdrawCapability</a>&lt;TokenT&gt;): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_remove_withdraw_capability"></a>

### Function `remove_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer): <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> !<b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_1_add_withdraw_capability"></a>

### Function `add_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT&gt;(signer: &signer, cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
<b>ensures</b> <b>exists</b>&lt;<a href="Treasury.md#0x1_Treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(signer));
</code></pre>



<a name="@Specification_1_destroy_withdraw_capability"></a>

### Function `destroy_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="Treasury.md#0x1_Treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="Treasury.md#0x1_Treasury_WithdrawCapability">Treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>
