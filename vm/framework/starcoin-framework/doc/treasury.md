
<a id="0x1_treasury"></a>

# Module `0x1::treasury`

The module for the Treasury of DAO, which can hold the token of DAO.


-  [Resource `Treasury`](#0x1_treasury_Treasury)
-  [Resource `WithdrawCapability`](#0x1_treasury_WithdrawCapability)
-  [Resource `LinearWithdrawCapability`](#0x1_treasury_LinearWithdrawCapability)
-  [Struct `WithdrawEvent`](#0x1_treasury_WithdrawEvent)
-  [Struct `DepositEvent`](#0x1_treasury_DepositEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_treasury_initialize)
-  [Function `exists_at`](#0x1_treasury_exists_at)
-  [Function `balance`](#0x1_treasury_balance)
-  [Function `deposit`](#0x1_treasury_deposit)
-  [Function `do_withdraw`](#0x1_treasury_do_withdraw)
-  [Function `withdraw_with_capability`](#0x1_treasury_withdraw_with_capability)
-  [Function `withdraw`](#0x1_treasury_withdraw)
-  [Function `issue_linear_withdraw_capability`](#0x1_treasury_issue_linear_withdraw_capability)
-  [Function `withdraw_with_linear_capability`](#0x1_treasury_withdraw_with_linear_capability)
-  [Function `withdraw_by_linear`](#0x1_treasury_withdraw_by_linear)
-  [Function `split_linear_withdraw_cap`](#0x1_treasury_split_linear_withdraw_cap)
-  [Function `withdraw_amount_of_linear_cap`](#0x1_treasury_withdraw_amount_of_linear_cap)
-  [Function `is_empty_linear_withdraw_cap`](#0x1_treasury_is_empty_linear_withdraw_cap)
-  [Function `remove_withdraw_capability`](#0x1_treasury_remove_withdraw_capability)
-  [Function `add_withdraw_capability`](#0x1_treasury_add_withdraw_capability)
-  [Function `destroy_withdraw_capability`](#0x1_treasury_destroy_withdraw_capability)
-  [Function `add_linear_withdraw_capability`](#0x1_treasury_add_linear_withdraw_capability)
-  [Function `remove_linear_withdraw_capability`](#0x1_treasury_remove_linear_withdraw_capability)
-  [Function `destroy_linear_withdraw_capability`](#0x1_treasury_destroy_linear_withdraw_capability)
-  [Function `is_empty_linear_withdraw_capability`](#0x1_treasury_is_empty_linear_withdraw_capability)
-  [Function `get_linear_withdraw_capability_total`](#0x1_treasury_get_linear_withdraw_capability_total)
-  [Function `get_linear_withdraw_capability_withdraw`](#0x1_treasury_get_linear_withdraw_capability_withdraw)
-  [Function `get_linear_withdraw_capability_period`](#0x1_treasury_get_linear_withdraw_capability_period)
-  [Function `get_linear_withdraw_capability_start_time`](#0x1_treasury_get_linear_withdraw_capability_start_time)
-  [Specification](#@Specification_1)
    -  [Resource `Treasury`](#@Specification_1_Treasury)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `balance`](#@Specification_1_balance)
    -  [Function `deposit`](#@Specification_1_deposit)
    -  [Function `do_withdraw`](#@Specification_1_do_withdraw)
    -  [Function `withdraw_with_capability`](#@Specification_1_withdraw_with_capability)
    -  [Function `withdraw`](#@Specification_1_withdraw)
    -  [Function `issue_linear_withdraw_capability`](#@Specification_1_issue_linear_withdraw_capability)
    -  [Function `withdraw_with_linear_capability`](#@Specification_1_withdraw_with_linear_capability)
    -  [Function `withdraw_by_linear`](#@Specification_1_withdraw_by_linear)
    -  [Function `split_linear_withdraw_cap`](#@Specification_1_split_linear_withdraw_cap)
    -  [Function `withdraw_amount_of_linear_cap`](#@Specification_1_withdraw_amount_of_linear_cap)
    -  [Function `is_empty_linear_withdraw_cap`](#@Specification_1_is_empty_linear_withdraw_cap)
    -  [Function `remove_withdraw_capability`](#@Specification_1_remove_withdraw_capability)
    -  [Function `add_withdraw_capability`](#@Specification_1_add_withdraw_capability)
    -  [Function `destroy_withdraw_capability`](#@Specification_1_destroy_withdraw_capability)
    -  [Function `add_linear_withdraw_capability`](#@Specification_1_add_linear_withdraw_capability)
    -  [Function `remove_linear_withdraw_capability`](#@Specification_1_remove_linear_withdraw_capability)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info">0x1::type_info</a>;
</code></pre>



<a id="0x1_treasury_Treasury"></a>

## Resource `Treasury`



<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>balance: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>withdraw_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="treasury.md#0x1_treasury_WithdrawEvent">treasury::WithdrawEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury withdraw event
</dd>
<dt>
<code>deposit_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="treasury.md#0x1_treasury_DepositEvent">treasury::DepositEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury deposit event
</dd>
</dl>


</details>

<a id="0x1_treasury_WithdrawCapability"></a>

## Resource `WithdrawCapability`

A withdraw capability allows tokens of type <code>TokenT</code> to be withdraw from Treasury.


<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; <b>has</b> store, key
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

<a id="0x1_treasury_LinearWithdrawCapability"></a>

## Resource `LinearWithdrawCapability`

A linear time withdraw capability which can withdraw token from Treasury in a period by time-based linear release.


<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>total: u128</code>
</dt>
<dd>
 The total amount of tokens that can be withdrawn by this capability
</dd>
<dt>
<code>withdraw: u128</code>
</dt>
<dd>
 The amount of tokens that have been withdrawn by this capability
</dd>
<dt>
<code>start_time: u64</code>
</dt>
<dd>
 The time-based linear release start time, timestamp in seconds.
</dd>
<dt>
<code>period: u64</code>
</dt>
<dd>
  The time-based linear release period in seconds
</dd>
</dl>


</details>

<a id="0x1_treasury_WithdrawEvent"></a>

## Struct `WithdrawEvent`

Message for treasury withdraw event.


<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_WithdrawEvent">WithdrawEvent</a> <b>has</b> drop, store
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

<a id="0x1_treasury_DepositEvent"></a>

## Struct `DepositEvent`

Message for treasury deposit event.


<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_DepositEvent">DepositEvent</a> <b>has</b> drop, store
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

<a id="@Constants_0"></a>

## Constants


<a id="0x1_treasury_ERR_INVALID_PERIOD"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>: u64 = 101;
</code></pre>



<a id="0x1_treasury_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 104;
</code></pre>



<a id="0x1_treasury_ERR_TOO_BIG_AMOUNT"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>: u64 = 103;
</code></pre>



<a id="0x1_treasury_ERR_TREASURY_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>: u64 = 105;
</code></pre>



<a id="0x1_treasury_ERR_ZERO_AMOUNT"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>: u64 = 102;
</code></pre>



<a id="0x1_treasury_initialize"></a>

## Function `initialize`

Init a Treasury for TokenT. Can only be called by token issuer.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;): <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) == token_issuer, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="treasury.md#0x1_treasury_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));
    <b>let</b> treasure = <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
        balance: init_token,
        withdraw_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="treasury.md#0x1_treasury_WithdrawEvent">WithdrawEvent</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
        deposit_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="treasury.md#0x1_treasury_DepositEvent">DepositEvent</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>),
    };
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, treasure);
    <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {}
}
</code></pre>



</details>

<a id="0x1_treasury_exists_at"></a>

## Function `exists_at`

Check the Treasury of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(): bool {
    <b>let</b> token_issuer = <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_account_address">type_info::account_address</a>(&<a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_type_of">type_info::type_of</a>&lt;TokenT&gt;());
    <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_issuer)
}
</code></pre>



</details>

<a id="0x1_treasury_balance"></a>

## Function `balance`

Get the balance of TokenT's Treasury
if the Treasury do not exists, return 0.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(): u128 <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>let</b> token_issuer = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>if</b> (!<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_issuer)) {
        <b>return</b> 0
    };
    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_issuer);
    (<a href="coin.md#0x1_coin_value">coin::value</a>(&<a href="treasury.md#0x1_treasury">treasury</a>.balance) <b>as</b> u128)
}
</code></pre>



</details>

<a id="0x1_treasury_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;) <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(<a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury.md#0x1_treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));
    <b>let</b> token_address = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_address);
    <b>let</b> amount = <a href="coin.md#0x1_coin_value">coin::value</a>(&token);
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.deposit_events,
        <a href="treasury.md#0x1_treasury_DepositEvent">DepositEvent</a> {
            amount: (amount <b>as</b> u128)
        },
    );
    <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.balance, token);
}
</code></pre>



</details>

<a id="0x1_treasury_do_withdraw"></a>

## Function `do_withdraw`



<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_do_withdraw">do_withdraw</a>&lt;TokenT&gt;(amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_do_withdraw">do_withdraw</a>&lt;TokenT&gt;(amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>assert</b>!(<a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury.md#0x1_treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));
    <b>let</b> token_address = <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;TokenT&gt;();
    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(token_address);
    <b>assert</b>!(amount &lt;= (<a href="coin.md#0x1_coin_value">coin::value</a>(&<a href="treasury.md#0x1_treasury">treasury</a>.balance) <b>as</b> u128), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>));
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.withdraw_events,
        <a href="treasury.md#0x1_treasury_WithdrawEvent">WithdrawEvent</a> { amount },
    );
    <a href="coin.md#0x1_coin_extract">coin::extract</a>(&<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.balance, (amount <b>as</b> u64))
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_with_capability"></a>

## Function `withdraw_with_capability`

Withdraw tokens with given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(_cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(
    _cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <a href="treasury.md#0x1_treasury_do_withdraw">do_withdraw</a>(amount)
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw"></a>

## Function `withdraw`

Withdraw from TokenT's treasury, the signer must have WithdrawCapability<TokenT>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    amount: u128
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>, <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a> {
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    <a href="treasury.md#0x1_treasury_withdraw_with_capability">Self::withdraw_with_capability</a>(cap, amount)
}
</code></pre>



</details>

<a id="0x1_treasury_issue_linear_withdraw_capability"></a>

## Function `issue_linear_withdraw_capability`

Issue a <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code> with given <code><a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(_capability: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(
    _capability: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
    period: u64
): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
    <b>assert</b>!(period &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>));
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>let</b> start_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
        total: amount,
        withdraw: 0,
        start_time,
        period,
    }
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_with_linear_capability"></a>

## Function `withdraw_with_linear_capability`

Withdraw tokens with given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;,
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>let</b> amount = <a href="treasury.md#0x1_treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>(cap);
    <b>let</b> token = <a href="treasury.md#0x1_treasury_do_withdraw">do_withdraw</a>(amount);
    cap.withdraw = cap.withdraw + amount;
    token
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_by_linear"></a>

## Function `withdraw_by_linear`

Withdraw from TokenT's  treasury, the signer must have LinearWithdrawCapability<TokenT>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_by_linear">withdraw_by_linear</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_by_linear">withdraw_by_linear</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a> {
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">Self::withdraw_with_linear_capability</a>(cap)
}
</code></pre>



</details>

<a id="0x1_treasury_split_linear_withdraw_cap"></a>

## Function `split_linear_withdraw_cap`

Split the given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
): (<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;) <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>let</b> token = <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">Self::withdraw_with_linear_capability</a>(cap);
    <b>assert</b>!((cap.withdraw + amount) &lt;= cap.total, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>));
    cap.total = cap.total - amount;
    <b>let</b> start_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <b>let</b> new_period = cap.start_time + cap.period - start_time;
    <b>let</b> new_key = <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
        total: amount,
        withdraw: 0,
        start_time,
        period: new_period
    };
    (token, new_key)
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_amount_of_linear_cap"></a>

## Function `withdraw_amount_of_linear_cap`

Returns the amount of the LinearWithdrawCapability can mint now.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): u128 {
    <b>let</b> now = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <b>let</b> elapsed_time = now - cap.start_time;
    <b>if</b> (elapsed_time &gt;= cap.period) {
        cap.total - cap.withdraw
    } <b>else</b> {
        <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_mul_div">math128::mul_div</a>(cap.total, (elapsed_time <b>as</b> u128), (cap.period <b>as</b> u128)) - cap.withdraw
    }
}
</code></pre>



</details>

<a id="0x1_treasury_is_empty_linear_withdraw_cap"></a>

## Function `is_empty_linear_withdraw_cap`

Check if the given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code> is empty.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT&gt;(key: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT&gt;(key: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): bool {
    key.total == key.withdraw
}
</code></pre>



</details>

<a id="0x1_treasury_remove_withdraw_capability"></a>

## Function `remove_withdraw_capability`

Remove mint capability from <code><a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
): <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a> {
    <b>move_from</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>))
}
</code></pre>



</details>

<a id="0x1_treasury_add_withdraw_capability"></a>

## Function `add_withdraw_capability`

Save mint capability to <code><a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;) {
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap)
}
</code></pre>



</details>

<a id="0x1_treasury_destroy_withdraw_capability"></a>

## Function `destroy_withdraw_capability`

Destroy the given mint capability.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {} = cap;
}
</code></pre>



</details>

<a id="0x1_treasury_add_linear_withdraw_capability"></a>

## Function `add_linear_withdraw_capability`

Add LinearWithdrawCapability to <code><a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a></code>, a address only can have one LinearWithdrawCapability<T>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">add_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">add_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;) {
    <b>move_to</b>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap)
}
</code></pre>



</details>

<a id="0x1_treasury_remove_linear_withdraw_capability"></a>

## Function `remove_linear_withdraw_capability`

Remove LinearWithdrawCapability from <code><a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_linear_withdraw_capability">remove_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_linear_withdraw_capability">remove_linear_withdraw_capability</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a> {
    <b>move_from</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>))
}
</code></pre>



</details>

<a id="0x1_treasury_destroy_linear_withdraw_capability"></a>

## Function `destroy_linear_withdraw_capability`

Destroy LinearWithdrawCapability.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_destroy_linear_withdraw_capability">destroy_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_destroy_linear_withdraw_capability">destroy_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;) {
    <b>let</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a> { total: _, withdraw: _, start_time: _, period: _ } = cap;
}
</code></pre>



</details>

<a id="0x1_treasury_is_empty_linear_withdraw_capability"></a>

## Function `is_empty_linear_withdraw_capability`



<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_capability">is_empty_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_capability">is_empty_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): bool {
    cap.total == cap.withdraw
}
</code></pre>



</details>

<a id="0x1_treasury_get_linear_withdraw_capability_total"></a>

## Function `get_linear_withdraw_capability_total`

Get LinearWithdrawCapability total amount


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_total">get_linear_withdraw_capability_total</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_total">get_linear_withdraw_capability_total</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): u128 {
    cap.total
}
</code></pre>



</details>

<a id="0x1_treasury_get_linear_withdraw_capability_withdraw"></a>

## Function `get_linear_withdraw_capability_withdraw`

Get LinearWithdrawCapability withdraw amount


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_withdraw">get_linear_withdraw_capability_withdraw</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_withdraw">get_linear_withdraw_capability_withdraw</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): u128 {
    cap.withdraw
}
</code></pre>



</details>

<a id="0x1_treasury_get_linear_withdraw_capability_period"></a>

## Function `get_linear_withdraw_capability_period`

Get LinearWithdrawCapability period in seconds


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_period">get_linear_withdraw_capability_period</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_period">get_linear_withdraw_capability_period</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): u64 {
    cap.period
}
</code></pre>



</details>

<a id="0x1_treasury_get_linear_withdraw_capability_start_time"></a>

## Function `get_linear_withdraw_capability_start_time`

Get LinearWithdrawCapability start_time in seconds


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_start_time">get_linear_withdraw_capability_start_time</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_get_linear_withdraw_capability_start_time">get_linear_withdraw_capability_start_time</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;): u64 {
    cap.start_time
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_1_Treasury"></a>

### Resource `Treasury`


<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt; <b>has</b> store, key
</code></pre>



<dl>
<dt>
<code>balance: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>withdraw_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="treasury.md#0x1_treasury_WithdrawEvent">treasury::WithdrawEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury withdraw event
</dd>
<dt>
<code>deposit_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="treasury.md#0x1_treasury_DepositEvent">treasury::DepositEvent</a>&gt;</code>
</dt>
<dd>
 event handle for treasury deposit event
</dd>
</dl>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) != @0x2;
<b>aborts_if</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>ensures</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>ensures</b> result == <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {};
</code></pre>



<a id="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
</code></pre>



<a id="@Specification_1_balance"></a>

### Function `balance`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <b>if</b> (<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2))
    result == <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;()
<b>else</b>
    result == 0;
</code></pre>



<a id="@Specification_1_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>aborts_if</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() + token.value &gt; MAX_U128;
<b>ensures</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() == <b>old</b>(<a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;()) + token.value;
</code></pre>



<a id="@Specification_1_do_withdraw"></a>

### Function `do_withdraw`


<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_do_withdraw">do_withdraw</a>&lt;TokenT&gt;(amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>include</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt;;
</code></pre>




<a id="0x1_treasury_WithdrawSchema"></a>


<pre><code><b>schema</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt; {
    amount: u64;
    <b>aborts_if</b> amount &lt;= 0;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
    <b>aborts_if</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() &lt; amount;
    <b>ensures</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() == <b>old</b>(<a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;()) - amount;
}
</code></pre>



<a id="@Specification_1_withdraw_with_capability"></a>

### Function `withdraw_with_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(_cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>include</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt;;
</code></pre>



<a id="@Specification_1_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>include</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt;;
</code></pre>



<a id="@Specification_1_issue_linear_withdraw_capability"></a>

### Function `issue_linear_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(_capability: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> period == 0;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_withdraw_with_linear_capability"></a>

### Function `withdraw_with_linear_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
</code></pre>



<a id="@Specification_1_withdraw_by_linear"></a>

### Function `withdraw_by_linear`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_by_linear">withdraw_by_linear</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
</code></pre>



<a id="@Specification_1_split_linear_withdraw_cap"></a>

### Function `split_linear_withdraw_cap`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenT&gt;, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
<b>ensures</b> <b>old</b>(cap.total - cap.withdraw) ==
    result_1.value + (result_2.total - result_2.withdraw) + (cap.total - cap.withdraw);
</code></pre>



<a id="@Specification_1_withdraw_amount_of_linear_cap"></a>

### Function `withdraw_amount_of_linear_cap`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>&lt;TokenT&gt;(cap: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): u128
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> <a href="timestamp.md#0x1_timestamp_spec_now_seconds">timestamp::spec_now_seconds</a>() &lt; cap.start_time;
<b>aborts_if</b> <a href="timestamp.md#0x1_timestamp_spec_now_seconds">timestamp::spec_now_seconds</a>() - cap.start_time &gt;= cap.period && cap.total &lt; cap.withdraw;
<b>aborts_if</b> [abstract]
    <a href="timestamp.md#0x1_timestamp_spec_now_seconds">timestamp::spec_now_seconds</a>() - cap.start_time &lt; cap.period && <a href="../../starcoin-stdlib/doc/math128.md#0x1_math128_spec_mul_div">math128::spec_mul_div</a>() &lt; cap.withdraw;
<b>ensures</b> [abstract] result &lt;= cap.total - cap.withdraw;
</code></pre>



<a id="@Specification_1_is_empty_linear_withdraw_cap"></a>

### Function `is_empty_linear_withdraw_cap`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_cap">is_empty_linear_withdraw_cap</a>&lt;TokenT&gt;(key: &<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == (key.total == key.withdraw);
</code></pre>



<a id="@Specification_1_remove_withdraw_capability"></a>

### Function `remove_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_withdraw_capability">remove_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>ensures</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
</code></pre>



<a id="@Specification_1_add_withdraw_capability"></a>

### Function `add_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_withdraw_capability">add_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>ensures</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
</code></pre>



<a id="@Specification_1_destroy_withdraw_capability"></a>

### Function `destroy_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_destroy_withdraw_capability">destroy_withdraw_capability</a>&lt;TokenT&gt;(cap: <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<a id="@Specification_1_add_linear_withdraw_capability"></a>

### Function `add_linear_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">add_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, cap: <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>ensures</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
</code></pre>



<a id="@Specification_1_remove_linear_withdraw_capability"></a>

### Function `remove_linear_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_remove_linear_withdraw_capability">remove_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>ensures</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
</code></pre>




<a id="0x1_treasury_spec_balance"></a>


<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenType&gt;(): num {
   <b>global</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenType&gt;&gt;(@0x2).balance.value
}
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
