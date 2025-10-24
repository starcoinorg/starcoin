
<a id="0x1_transaction_fee"></a>

# Module `0x1::transaction_fee`

<code>TransactionFee</code> collect gas fees used by transactions in blocks temporarily.
Then they are distributed in <code>TransactionManager</code>.


-  [Resource `TransactionFeePod`](#0x1_transaction_fee_TransactionFeePod)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_transaction_fee_initialize)
-  [Function `pay_fee`](#0x1_transaction_fee_pay_fee)
-  [Function `merge_fee_to_framework_account`](#0x1_transaction_fee_merge_fee_to_framework_account)
-  [Function `withdraw_account_transaction_fees`](#0x1_transaction_fee_withdraw_account_transaction_fees)
-  [Function `inner_create_fa_store`](#0x1_transaction_fee_inner_create_fa_store)
-  [Function `find_asset_store_with_metadata`](#0x1_transaction_fee_find_asset_store_with_metadata)
-  [Function `get_fa_store_seed`](#0x1_transaction_fee_get_fa_store_seed)
-  [Specification](#@Specification_1)
    -  [Function `pay_fee`](#@Specification_1_pay_fee)
    -  [Function `withdraw_account_transaction_fees`](#@Specification_1_withdraw_account_transaction_fees)


<pre><code><b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
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



<a id="0x1_transaction_fee_ETXN_FEE_POD_HAS_INITIALIZED"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_HAS_INITIALIZED">ETXN_FEE_POD_HAS_INITIALIZED</a>: u64 = 2;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>: u64 = 1;
</code></pre>



<a id="0x1_transaction_fee_ETXN_FEE_STORES_IS_EMPTY"></a>



<pre><code><b>const</b> <a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_STORES_IS_EMPTY">ETXN_FEE_STORES_IS_EMPTY</a>: u64 = 5;
</code></pre>



<a id="0x1_transaction_fee_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>assert</b>!(
        !<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(framework)),
        <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>)
    );

    <b>let</b> fee_stores = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(
        &<b>mut</b> fee_stores,
        <a href="transaction_fee.md#0x1_transaction_fee_inner_create_fa_store">Self::inner_create_fa_store</a>(framework, <a href="starcoin_coin.md#0x1_starcoin_coin_get_stc_fa_metadata">starcoin_coin::get_stc_fa_metadata</a>())
    );
    <b>move_to</b>(framework, <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
        fee_stores
    });
}
</code></pre>



</details>

<a id="0x1_transaction_fee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa: FungibleAsset) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_pay_fee">transaction_fee::pay_fee</a> | Entered"));

    <b>let</b> account_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> fa_store = <b>if</b> (<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(account_addr)) {
        <b>let</b> fee_pod = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(get_starcoin_framework());
        <b>let</b> store_opt = <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(
            &fee_pod.fee_stores,
            <a href="fungible_asset.md#0x1_fungible_asset_metadata_from_asset">fungible_asset::metadata_from_asset</a>(&fa)
        );
        <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&store_opt)) {
            <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(store_opt)
        } <b>else</b> {
            <b>let</b> fa_store = <a href="transaction_fee.md#0x1_transaction_fee_inner_create_fa_store">Self::inner_create_fa_store</a>(<a href="account.md#0x1_account">account</a>, <a href="fungible_asset.md#0x1_fungible_asset_metadata_from_asset">fungible_asset::metadata_from_asset</a>(&fa));
            <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> fee_pod.fee_stores, fa_store);
            fa_store
        }
    } <b>else</b> {
        <b>let</b> fa_store = <a href="transaction_fee.md#0x1_transaction_fee_inner_create_fa_store">Self::inner_create_fa_store</a>(<a href="account.md#0x1_account">account</a>, <a href="starcoin_coin.md#0x1_starcoin_coin_get_stc_fa_metadata">starcoin_coin::get_stc_fa_metadata</a>());
        <b>let</b> fee_stores = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> fee_stores, fa_store);
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
            fee_stores
        });
        fa_store
    };
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(fa_store, fa);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_pay_fee">transaction_fee::pay_fee</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_transaction_fee_merge_fee_to_framework_account"></a>

## Function `merge_fee_to_framework_account`



<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_merge_fee_to_framework_account">merge_fee_to_framework_account</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payer_addresses: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_merge_fee_to_framework_account">merge_fee_to_framework_account</a>(
    framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payer_addresses: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;
) <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(framework);

    <b>let</b> framework_address = <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&payer_addresses);
    for (i in 0 .. len) {
        <b>let</b> addr = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&payer_addresses, i);
        <b>if</b> (addr != framework_address && <b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(addr)) {
            <b>let</b> transaction_fee_pod = <b>borrow_global</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(addr);
            <b>let</b> stc_metadata = <a href="starcoin_coin.md#0x1_starcoin_coin_get_stc_fa_metadata">starcoin_coin::get_stc_fa_metadata</a>();
            <b>let</b> fa_store = <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">Self::find_asset_store_with_metadata</a>(
                &transaction_fee_pod.fee_stores,
                stc_metadata
            );
            <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&fa_store), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_STORE_NOT_FOUND">ETXN_FEE_FA_STORE_NOT_FOUND</a>));
            <b>let</b> fa = <a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">Self::withdraw_account_transaction_fees</a>(&<a href="create_signer.md#0x1_create_signer">create_signer</a>(addr), stc_metadata);
            <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(framework, fa);
        }
    }
}
</code></pre>



</details>

<a id="0x1_transaction_fee_withdraw_account_transaction_fees"></a>

## Function `withdraw_account_transaction_fees`

Distribute the transaction fees collected in the <code>TokenType</code> token.
If the <code>TokenType</code> is STC, it unpacks the token and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">withdraw_account_transaction_fees</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">withdraw_account_transaction_fees</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    metadata: Object&lt;Metadata&gt;
): FungibleAsset <b>acquires</b> <a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">transaction_fee::withdraw_account_transaction_fees</a> | Entered"));

    <b>let</b> account_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(<b>exists</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(account_addr), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_POD_NOT_INITIALIZED">ETXN_FEE_POD_NOT_INITIALIZED</a>));

    <b>let</b> fee_pod = <b>borrow_global_mut</b>&lt;<a href="transaction_fee.md#0x1_transaction_fee_TransactionFeePod">TransactionFeePod</a>&gt;(account_addr);
    <b>let</b> fa_store_opt = <a href="transaction_fee.md#0x1_transaction_fee_find_asset_store_with_metadata">find_asset_store_with_metadata</a>(&fee_pod.fee_stores, metadata);
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&fa_store_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="transaction_fee.md#0x1_transaction_fee_ETXN_FEE_FA_STORE_NOT_FOUND">ETXN_FEE_FA_STORE_NOT_FOUND</a>));

    <b>let</b> fa_store = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(fa_store_opt);
    <b>let</b> all_asset_balance = <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(fa_store);

    <b>let</b> ret = <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(<a href="account.md#0x1_account">account</a>, fa_store, all_asset_balance);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">transaction_fee::withdraw_account_transaction_fees</a> | Exited"));

    ret
}
</code></pre>



</details>

<a id="0x1_transaction_fee_inner_create_fa_store"></a>

## Function `inner_create_fa_store`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_inner_create_fa_store">inner_create_fa_store</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_inner_create_fa_store">inner_create_fa_store</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: Object&lt;Metadata&gt;): Object&lt;FungibleStore&gt; {
    <b>let</b> fa_store_seed = <a href="transaction_fee.md#0x1_transaction_fee_get_fa_store_seed">Self::get_fa_store_seed</a>(metadata);
    <b>let</b> construct_addr = <a href="object.md#0x1_object_create_object_address">object::create_object_address</a>(&<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>), fa_store_seed);
    <b>let</b> fa_store = <b>if</b> (<a href="object.md#0x1_object_object_exists">object::object_exists</a>&lt;FungibleStore&gt;(construct_addr)) {
        <a href="object.md#0x1_object_address_to_object">object::address_to_object</a>&lt;FungibleStore&gt;(construct_addr)
    } <b>else</b> {
        <b>let</b> construct_ref = <a href="object.md#0x1_object_create_named_object">object::create_named_object</a>(<a href="account.md#0x1_account">account</a>, fa_store_seed);
        <a href="fungible_asset.md#0x1_fungible_asset_create_store">fungible_asset::create_store</a>(&construct_ref, metadata)
    };
    fa_store
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

<a id="0x1_transaction_fee_get_fa_store_seed"></a>

## Function `get_fa_store_seed`



<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_fa_store_seed">get_fa_store_seed</a>(metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_get_fa_store_seed">get_fa_store_seed</a>(metadata: Object&lt;Metadata&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; {
    <b>let</b> seed = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> seed, b"<a href="transaction_fee.md#0x1_transaction_fee">transaction_fee</a>");
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> seed, 0xFE);
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> seed, *<a href="../../move-stdlib/doc/string.md#0x1_string_bytes">string::bytes</a>(&<a href="fungible_asset.md#0x1_fungible_asset_name">fungible_asset::name</a>(metadata)));
    seed
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_1_pay_fee"></a>

### Function `pay_fee`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_pay_fee">pay_fee</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, fa: <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_withdraw_account_transaction_fees"></a>

### Function `withdraw_account_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="transaction_fee.md#0x1_transaction_fee_withdraw_account_transaction_fees">withdraw_account_transaction_fees</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;): <a href="fungible_asset.md#0x1_fungible_asset_FungibleAsset">fungible_asset::FungibleAsset</a>
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
