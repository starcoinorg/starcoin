
<a id="0x1_transaction_fee"></a>

# Module `0x1::transaction_fee`

<code>TransactionFee</code> collect gas fees used by transactions in blocks temporarily.
Then they are distributed in <code>TransactionManager</code>.


-  [Resource `TransactionFeePod`](#0x1_transaction_fee_TransactionFeePod)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_transaction_fee_initialize)
-  [Function `pay_fee`](#0x1_transaction_fee_pay_fee)
-  [Function `distribute_transaction_fees`](#0x1_transaction_fee_distribute_transaction_fees)
-  [Function `find_asset_store_with_metadata`](#0x1_transaction_fee_find_asset_store_with_metadata)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `pay_fee`](#@Specification_1_pay_fee)
    -  [Function `distribute_transaction_fees`](#@Specification_1_distribute_transaction_fees)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_transaction_fee_TransactionFeePod"></a>

## Resource `TransactionFeePod`

The <code>TransactionFee</code> resource holds a preburn resource for each
fiat <code>TokenType</code> that can be collected as a transaction fee.


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fee_stores: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>owner_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_transaction_fee_ETXN_FEE_FA_METADATA_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_METADATA_NOT_FOUND">ETXN_FEE_FA_METADATA_NOT_FOUND</a>: u64 = 3;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_FA_STORE_NOT_FOUND"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_STORE_NOT_FOUND">ETXN_FEE_FA_STORE_NOT_FOUND</a>: u64 = 4;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>: u64 = 2;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_STC_METADATA_NOT_INITIALIZED"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STC_METADATA_NOT_INITIALIZED">ETXN_FEE_STC_METADATA_NOT_INITIALIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_STORES_IS_EMPTY"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STORES_IS_EMPTY">ETXN_FEE_STORES_IS_EMPTY</a>: u64 = 5;
</code></pre>



<a id="0x1_transaction_fee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code>TransactionFee</code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_initialize">transaction_fee::initialize</a> | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(framework);

    <b>let</b> constructor_ref = <a href="object.md#0x1_object_create_named_object">object::create_named_object</a>(framework, b"txn_fee");
    <b>let</b> stc_metadata_opt = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;STC&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&stc_metadata_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STC_METADATA_NOT_INITIALIZED">ETXN_FEE_STC_METADATA_NOT_INITIALIZED</a>));

    <b>let</b> stc_metadata = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(stc_metadata_opt);
    <b>let</b> fee_fa_store = create_store(&constructor_ref, stc_metadata);
    <b>assert</b>!(
        <a href="fungible_asset.md#0x1_fungible_asset_store_exists">fungible_asset::store_exists</a>(<a href="object.md#0x1_object_address_from_constructor_ref">object::address_from_constructor_ref</a>(&constructor_ref)),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STC_METADATA_NOT_INITIALIZED">ETXN_FEE_STC_METADATA_NOT_INITIALIZED</a>)
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&constructor_ref);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&fee_fa_store);

    <b>let</b> fee_stores = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;Object&lt;FungibleStore&gt;&gt;();
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> fee_stores, fee_fa_store);

    <b>let</b> owner_address = <a href="object.md#0x1_object_address_from_constructor_ref">object::address_from_constructor_ref</a>(&constructor_ref);
    <b>move_to</b>(
        framework,
        <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> { fee_stores, owner_address }
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_initialize">transaction_fee::initialize</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_transaction_fee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(fa: FungibleAsset) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
    <b>assert</b>!(<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(get_starcoin_framework()), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(
        <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>
    ));

    <b>let</b> fee_pod = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(get_starcoin_framework());
    <b>let</b> store_opt = <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(
        &fee_pod.fee_stores,
        <a href="fungible_asset.md#0x1_fungible_asset_metadata_from_asset">fungible_asset::metadata_from_asset</a>(&fa)
    );
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&store_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>));

    <b>let</b> store = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(store_opt);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(store, fa);
}
</code></pre>



</details>

<a id="0x1_transaction_fee_distribute_transaction_fees"></a>

## Function `distribute_transaction_fees`

Distribute the transaction fees collected in the <code>TokenType</code> token.
If the <code>TokenType</code> is STC, it unpacks the token and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(
    framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
): FungibleAsset <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_distribute_transaction_fees">transaction_fee::distribute_transaction_fees</a> | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(framework);
    <b>let</b> framework_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(framework);
    <b>assert</b>!(<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(framework_addr), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>));

    <b>let</b> fee_pod = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(framework_addr);
    <b>let</b> coin_metadata_opt = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;TokenType&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&coin_metadata_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_METADATA_NOT_FOUND">ETXN_FEE_FA_METADATA_NOT_FOUND</a>));

    <b>let</b> coin_metatdata = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(coin_metadata_opt);
    <b>let</b> fa_store_opt = <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(&fee_pod.fee_stores, coin_metatdata);
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&fa_store_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_STORE_NOT_FOUND">ETXN_FEE_FA_STORE_NOT_FOUND</a>));

    <b>let</b> fa_store = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(fa_store_opt);
    <b>let</b> all_asset_balance = <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(fa_store);

    <b>let</b> txn_fee_signer = <a href="create_signer.md#0x1_create_signer">create_signer</a>(fee_pod.owner_address);
    <b>let</b> ret = <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(&txn_fee_signer, fa_store, all_asset_balance);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_distribute_transaction_fees">transaction_fee::distribute_transaction_fees</a> | Exited"));

    ret
}
</code></pre>



</details>

<a id="0x1_transaction_fee_find_asset_store_with_metadata"></a>

## Function `find_asset_store_with_metadata`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(fee_stores: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;&gt;, target_metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(
    fee_stores: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;Object&lt;FungibleStore&gt;&gt;,
    target_metadata: Object&lt;Metadata&gt;
): Option&lt;Object&lt;FungibleStore&gt;&gt; {
    <b>let</b> fee_len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(fee_stores);
    <b>assert</b>!(fee_len &gt; 0, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STORES_IS_EMPTY">ETXN_FEE_STORES_IS_EMPTY</a>));

    <b>let</b> idx: u64 = 0;
    <b>while</b> (idx &lt; fee_len) {
        <b>let</b> store = <a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(fee_stores, idx);
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(store);
        <b>let</b> store_metadata = <a href="fungible_asset.md#0x1_fungible_asset_store_metadata">fungible_asset::store_metadata</a>(*store);

        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&store_metadata);
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&target_metadata);

        <b>if</b> (store_metadata == target_metadata) {
            <b>return</b> <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(*store)
        };
        idx = idx + 1;
    };
    <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>()
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_pay_fee"></a>

### Function `pay_fee`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_distribute_transaction_fees"></a>

### Function `distribute_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
