
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
-  [Function `inner_do_withdraw`](#0x1_treasury_inner_do_withdraw)
-  [Function `withdraw_with_capability`](#0x1_treasury_withdraw_with_capability)
-  [Function `withdraw`](#0x1_treasury_withdraw)
-  [Function `issue_linear_withdraw_capability`](#0x1_treasury_issue_linear_withdraw_capability)
-  [Function `withdraw_with_linear_capability`](#0x1_treasury_withdraw_with_linear_capability)
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
    -  [Function `withdraw_with_capability`](#@Specification_1_withdraw_with_capability)
    -  [Function `withdraw`](#@Specification_1_withdraw)
    -  [Function `issue_linear_withdraw_capability`](#@Specification_1_issue_linear_withdraw_capability)
    -  [Function `withdraw_with_linear_capability`](#@Specification_1_withdraw_with_linear_capability)
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
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_treasury_Treasury"></a>

## Resource `Treasury`



<pre><code><b>struct</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fa_store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>store_owner: <b>address</b></code>
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
<code>owner: <b>address</b></code>
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
<code>owner: <b>address</b></code>
</dt>
<dd>

</dd>
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


<a id="0x1_treasury_ERR_INITA_ASSET_NOT_MATCH"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_INITA_ASSET_NOT_MATCH">ERR_INITA_ASSET_NOT_MATCH</a>: u64 = 107;
</code></pre>



<a id="0x1_treasury_ERR_INVALID_PERIOD"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>: u64 = 101;
</code></pre>



<a id="0x1_treasury_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 104;
</code></pre>



<a id="0x1_treasury_ERR_TOKEN_NOT_CREATE_TOKEN_PAIR"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_TOKEN_NOT_CREATE_TOKEN_PAIR">ERR_TOKEN_NOT_CREATE_TOKEN_PAIR</a>: u64 = 106;
</code></pre>



<a id="0x1_treasury_ERR_TOO_BIG_AMOUNT"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>: u64 = 103;
</code></pre>



<a id="0x1_treasury_ERR_TREASURY_INIAIZLIED"></a>



<pre><code><b>const</b> <a href="treasury.md#0x1_treasury_ERR_TREASURY_INIAIZLIED">ERR_TREASURY_INIAIZLIED</a>: u64 = 108;
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


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, initia_fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    initia_fa: FungibleAsset,
): <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>let</b> account_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(!<a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(account_addr), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="treasury.md#0x1_treasury_ERR_TREASURY_INIAIZLIED">ERR_TREASURY_INIAIZLIED</a>));

    <b>let</b> coin_metadata_opt = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;TokenT&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&coin_metadata_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="treasury.md#0x1_treasury_ERR_TOKEN_NOT_CREATE_TOKEN_PAIR">ERR_TOKEN_NOT_CREATE_TOKEN_PAIR</a>));

    <b>let</b> asset_metadata = <a href="fungible_asset.md#0x1_fungible_asset_asset_metadata">fungible_asset::asset_metadata</a>(&initia_fa);
    <b>assert</b>!(
        asset_metadata == <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(coin_metadata_opt),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="treasury.md#0x1_treasury_ERR_INITA_ASSET_NOT_MATCH">ERR_INITA_ASSET_NOT_MATCH</a>)
    );

    <b>let</b> constructor_ref = <a href="object.md#0x1_object_create_object">object::create_object</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
    <b>let</b> fa_store = <a href="fungible_asset.md#0x1_fungible_asset_create_store">fungible_asset::create_store</a>(&constructor_ref, asset_metadata);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(fa_store, initia_fa);

    // Check fungible asset
    <b>move_to</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(<a href="account.md#0x1_account">account</a>, <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
        fa_store,
        store_owner: <a href="object.md#0x1_object_address_from_constructor_ref">object::address_from_constructor_ref</a>(&constructor_ref),
        withdraw_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="treasury.md#0x1_treasury_WithdrawEvent">WithdrawEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
        deposit_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="treasury.md#0x1_treasury_DepositEvent">DepositEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
    });

    <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; {
        owner: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>),
    }
}
</code></pre>



</details>

<a id="0x1_treasury_exists_at"></a>

## Function `exists_at`

Check the Treasury of TokenT is exists.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(owner: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(owner: <b>address</b>): bool <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner);

    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner);
    <a href="fungible_asset.md#0x1_fungible_asset_store_exists">fungible_asset::store_exists</a>(<a href="object.md#0x1_object_owner">object::owner</a>(<a href="treasury.md#0x1_treasury">treasury</a>.fa_store))
}
</code></pre>



</details>

<a id="0x1_treasury_balance"></a>

## Function `balance`

Get the balance of TokenT's Treasury
if the Treasury do not exists, return 0.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(owner: <b>address</b>): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(owner: <b>address</b>): u128 <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>if</b> (!<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner)) {
        <b>return</b> 0
    };
    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner);
    (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(<a href="treasury.md#0x1_treasury">treasury</a>.fa_store) <b>as</b> u128)
}
</code></pre>



</details>

<a id="0x1_treasury_deposit"></a>

## Function `deposit`



<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(owner: <b>address</b>, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(owner: <b>address</b>, fa: FungibleAsset) <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(<a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury.md#0x1_treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));

    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner);

    <b>let</b> amount = <a href="fungible_asset.md#0x1_fungible_asset_amount">fungible_asset::amount</a>(&fa);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(<a href="treasury.md#0x1_treasury">treasury</a>.fa_store, fa);
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.deposit_events,
        <a href="treasury.md#0x1_treasury_DepositEvent">DepositEvent</a> {
            amount: (amount <b>as</b> u128)
        },
    );
}
</code></pre>



</details>

<a id="0x1_treasury_inner_do_withdraw"></a>

## Function `inner_do_withdraw`



<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_inner_do_withdraw">inner_do_withdraw</a>&lt;TokenT&gt;(owner: <b>address</b>, amount: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="treasury.md#0x1_treasury_inner_do_withdraw">inner_do_withdraw</a>&lt;TokenT&gt;(owner: <b>address</b>, amount: u128): FungibleAsset <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>assert</b>!(<a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="treasury.md#0x1_treasury_ERR_TREASURY_NOT_EXIST">ERR_TREASURY_NOT_EXIST</a>));

    <b>let</b> <a href="treasury.md#0x1_treasury">treasury</a> = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(owner);
    <b>assert</b>!(
        amount &lt;= (<a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(<a href="treasury.md#0x1_treasury">treasury</a>.fa_store) <b>as</b> u128),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>)
    );
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(
        &<b>mut</b> <a href="treasury.md#0x1_treasury">treasury</a>.withdraw_events,
        <a href="treasury.md#0x1_treasury_WithdrawEvent">WithdrawEvent</a> { amount },
    );
    <b>let</b> store_signer = <a href="create_signer.md#0x1_create_signer">create_signer</a>(<a href="object.md#0x1_object_owner">object::owner</a>(<a href="treasury.md#0x1_treasury">treasury</a>.fa_store));
    <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(&store_signer, <a href="treasury.md#0x1_treasury">treasury</a>.fa_store, (amount <b>as</b> u64))
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw_with_capability"></a>

## Function `withdraw_with_capability`

Withdraw tokens with given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
): FungibleAsset <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <a href="treasury.md#0x1_treasury_inner_do_withdraw">inner_do_withdraw</a>&lt;TokenT&gt;(cap.owner, amount)
}
</code></pre>



</details>

<a id="0x1_treasury_withdraw"></a>

## Function `withdraw`

Withdraw from TokenT's treasury, the signer must have WithdrawCapability<TokenT>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    amount: u128
): FungibleAsset <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a>, <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a> {
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
    <a href="treasury.md#0x1_treasury_withdraw_with_capability">Self::withdraw_with_capability</a>(cap, amount)
}
</code></pre>



</details>

<a id="0x1_treasury_issue_linear_withdraw_capability"></a>

## Function `issue_linear_withdraw_capability`

Issue a <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code> with given <code><a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
    period: u64
): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
    <b>assert</b>!(period &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_INVALID_PERIOD">ERR_INVALID_PERIOD</a>));
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>let</b> start_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
        owner: cap.owner,
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


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;,
): FungibleAsset <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>let</b> amount = <a href="treasury.md#0x1_treasury_withdraw_amount_of_linear_cap">withdraw_amount_of_linear_cap</a>(cap);
    <b>let</b> fa = <a href="treasury.md#0x1_treasury_inner_do_withdraw">Self::inner_do_withdraw</a>&lt;TokenT&gt;(cap.owner, amount);
    cap.withdraw = cap.withdraw + amount;
    fa
}
</code></pre>



</details>

<a id="0x1_treasury_split_linear_withdraw_cap"></a>

## Function `split_linear_withdraw_cap`

Split the given <code><a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(
    cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;,
    amount: u128,
): (FungibleAsset, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt;) <b>acquires</b> <a href="treasury.md#0x1_treasury_Treasury">Treasury</a> {
    <b>assert</b>!(amount &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_ZERO_AMOUNT">ERR_ZERO_AMOUNT</a>));
    <b>let</b> token = <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">Self::withdraw_with_linear_capability</a>(cap);
    <b>assert</b>!((cap.withdraw + amount) &lt;= cap.total, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="treasury.md#0x1_treasury_ERR_TOO_BIG_AMOUNT">ERR_TOO_BIG_AMOUNT</a>));
    cap.total = cap.total - amount;
    <b>let</b> start_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <b>let</b> new_period = cap.start_time + cap.period - start_time;
    <b>let</b> new_key = <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a>&lt;TokenT&gt; {
        owner: cap.owner,
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
    <b>let</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; { owner: _ } = cap;
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
    <b>let</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">LinearWithdrawCapability</a> { owner: _, total: _, withdraw: _, start_time: _, period: _ } = cap;
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
<code>fa_store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>store_owner: <b>address</b></code>
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


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_initialize">initialize</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, initia_fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>): <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != @0x2;
<b>aborts_if</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>ensures</b> <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>ensures</b> result == <a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt; { owner: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) };
</code></pre>



<a id="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_exists_at">exists_at</a>&lt;TokenT&gt;(owner: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == <b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
</code></pre>



<a id="@Specification_1_balance"></a>

### Function `balance`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_balance">balance</a>&lt;TokenT&gt;(owner: <b>address</b>): u128
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> <b>if</b> (<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2))
    result == <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;()
<b>else</b>
    result == 0;
</code></pre>



<a id="@Specification_1_deposit"></a>

### Function `deposit`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_deposit">deposit</a>&lt;TokenT&gt;(owner: <b>address</b>, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenT&gt;&gt;(@0x2);
<b>aborts_if</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() + fa.amount &gt; MAX_U128;
<b>ensures</b> <a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;() == <b>old</b>(<a href="treasury.md#0x1_treasury_spec_balance">spec_balance</a>&lt;TokenT&gt;()) + fa.amount;
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


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_capability">withdraw_with_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>




<pre><code><b>include</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt;;
</code></pre>



<a id="@Specification_1_withdraw"></a>

### Function `withdraw`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw">withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="treasury.md#0x1_treasury_WithdrawCapability">WithdrawCapability</a>&lt;TokenT&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>));
<b>include</b> <a href="treasury.md#0x1_treasury_WithdrawSchema">WithdrawSchema</a>&lt;TokenT&gt;;
</code></pre>



<a id="@Specification_1_issue_linear_withdraw_capability"></a>

### Function `issue_linear_withdraw_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_issue_linear_withdraw_capability">issue_linear_withdraw_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_WithdrawCapability">treasury::WithdrawCapability</a>&lt;TokenT&gt;, amount: u128, period: u64): <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;
</code></pre>




<pre><code><b>aborts_if</b> period == 0;
<b>aborts_if</b> amount == 0;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_withdraw_with_linear_capability"></a>

### Function `withdraw_with_linear_capability`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">withdraw_with_linear_capability</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
</code></pre>



<a id="@Specification_1_split_linear_withdraw_cap"></a>

### Function `split_linear_withdraw_cap`


<pre><code><b>public</b> <b>fun</b> <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">split_linear_withdraw_cap</a>&lt;TokenT&gt;(cap: &<b>mut</b> <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;, amount: u128): (<a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>, <a href="treasury.md#0x1_treasury_LinearWithdrawCapability">treasury::LinearWithdrawCapability</a>&lt;TokenT&gt;)
</code></pre>




<pre><code><b>pragma</b> aborts_if_is_partial;
<b>ensures</b> <b>old</b>(cap.total - cap.withdraw) ==
    result_1.amount + (result_2.total - result_2.withdraw) + (cap.total - cap.withdraw);
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
   <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(<b>global</b>&lt;<a href="treasury.md#0x1_treasury_Treasury">Treasury</a>&lt;TokenType&gt;&gt;(@0x2).fa_store)
}
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
