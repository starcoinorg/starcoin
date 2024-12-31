
<a id="0x1_asset_mapping"></a>

# Module `0x1::asset_mapping`

Asset Mapping Module
This module implements functionality for managing fungible asset mappings in the Starcoin framework.
It provides capabilities for creating stores, managing balances, and assigning assets to accounts
with proof verification.


-  [Resource `AssetMappingStore`](#0x1_asset_mapping_AssetMappingStore)
-  [Resource `AssetMappingPool`](#0x1_asset_mapping_AssetMappingPool)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_asset_mapping_initialize)
-  [Function `create_store_from_coin`](#0x1_asset_mapping_create_store_from_coin)
-  [Function `create_store_for_type`](#0x1_asset_mapping_create_store_for_type)
-  [Function `balance`](#0x1_asset_mapping_balance)
-  [Function `assign_to_account`](#0x1_asset_mapping_assign_to_account)
-  [Function `computer_poove`](#0x1_asset_mapping_computer_poove)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table">0x1::smart_table</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_asset_mapping_AssetMappingStore"></a>

## Resource `AssetMappingStore`

AssetMappingStore represents a store for mapped assets
Contains:
- extend_ref: Reference for extending object capabilities
- fungible_store: The actual store holding fungible assets


<pre><code><b>struct</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>extend_ref: <a href="object.md#0x1_object_ExtendRef">object::ExtendRef</a></code>
</dt>
<dd>

</dd>
<dt>
<code>fungible_store: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_asset_mapping_AssetMappingPool"></a>

## Resource `AssetMappingPool`

AssetMappingPool manages a collection of asset mapping stores
Contains:
- proof_root: Root hash for proof verification
- anchor_height: Block height anchor for the mapping
- token_stores: Smart table mapping metadata to stores


<pre><code><b>struct</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>proof_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>anchor_height: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>token_stores: <a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table_SmartTable">smart_table::SmartTable</a>&lt;<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">asset_mapping::AssetMappingStore</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_asset_mapping_EINVALID_NOT_PROOF"></a>



<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_NOT_PROOF">EINVALID_NOT_PROOF</a>: u64 = 102;
</code></pre>



<a id="0x1_asset_mapping_EINVALID_SIGNER"></a>

Error code for invalid signer


<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>: u64 = 101;
</code></pre>



<a id="0x1_asset_mapping_initialize"></a>

## Function `initialize`

Initializes the asset mapping pool
@param framework - The framework signer
@param proof_root - Initial proof root for verification
@param anchor_height - Initial anchor height
Verifies the framework signer and creates a new AssetMappingPool


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proof_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, anchor_height: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proof_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, anchor_height: u64) {
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(framework) == <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>)
    );
    <b>move_to</b>(framework, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
        proof_root,
        anchor_height,
        token_stores: <a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table_new">smart_table::new</a>&lt;Object&lt;Metadata&gt;, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a>&gt;(),
    });
}
</code></pre>



</details>

<a id="0x1_asset_mapping_create_store_from_coin"></a>

## Function `create_store_from_coin`

Creates a new store from a coin
@param token_issuer - The token issuer signer
@param coin - The coin to be stored
Requirements:
- Token issuer must be authorized for the given token type
- Converts coin to fungible asset and stores it


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">create_store_from_coin</a>&lt;T: key&gt;(token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">create_store_from_coin</a>&lt;T: key&gt;(token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;T&gt;) <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
    <b>let</b> token_issuer_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(token_issuer);
    <b>assert</b>!(
        token_issuer_addr == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;T&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>)
    );

    <b>let</b> <a href="fungible_asset.md#0x1_fungible_asset">fungible_asset</a> = <a href="coin.md#0x1_coin_coin_to_fungible_asset">coin::coin_to_fungible_asset</a>(<a href="coin.md#0x1_coin">coin</a>);
    <b>let</b> token_stores =
        &<b>mut</b> <b>borrow_global_mut</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).token_stores;


    <b>let</b> (metadata, fungible_store, extend_ref) = <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_type">create_store_for_type</a>&lt;T&gt;(token_issuer);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(fungible_store, <a href="fungible_asset.md#0x1_fungible_asset">fungible_asset</a>);
    <a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table_add">smart_table::add</a>(token_stores, metadata, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> {
        extend_ref,
        fungible_store,
    });
}
</code></pre>



</details>

<a id="0x1_asset_mapping_create_store_for_type"></a>

## Function `create_store_for_type`

Creates a store for a specific token type
@param framework - The framework signer
@returns (metadata, store, extend_ref):
- metadata: Token metadata object
- store: Created fungible store
- extend_ref: Extension reference for the store


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_type">create_store_for_type</a>&lt;T&gt;(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;, <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;, <a href="object.md#0x1_object_ExtendRef">object::ExtendRef</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_type">create_store_for_type</a>&lt;T&gt;(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (Object&lt;Metadata&gt;, Object&lt;FungibleStore&gt;, ExtendRef) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_for_type">asset_mapping::create_store_for_type</a> | entered"));

    <b>let</b> metadata = <a href="coin.md#0x1_coin_ensure_paired_metadata">coin::ensure_paired_metadata</a>&lt;T&gt;();
    <b>let</b> construct_ref = <a href="object.md#0x1_object_create_object_from_account">object::create_object_from_account</a>(framework);

    <b>let</b> store = <a href="fungible_asset.md#0x1_fungible_asset_create_store">fungible_asset::create_store</a>(&construct_ref, metadata);

    // Generate extend reference
    <b>let</b> extend_ref = <a href="object.md#0x1_object_generate_extend_ref">object::generate_extend_ref</a>(&construct_ref);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_for_type">asset_mapping::create_store_for_type</a> | exited"));

    (metadata, store, extend_ref)
}
</code></pre>



</details>

<a id="0x1_asset_mapping_balance"></a>

## Function `balance`

Retrieves the balance for a specific token type
@returns Current balance of the token in the mapping pool


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_balance">balance</a>&lt;T&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_balance">balance</a>&lt;T&gt;(): u64 <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
    <b>let</b> metadata = <a href="coin.md#0x1_coin_ensure_paired_metadata">coin::ensure_paired_metadata</a>&lt;T&gt;();
    <b>let</b> pool = <b>borrow_global</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(<a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table_borrow">smart_table::borrow</a>(&pool.token_stores, metadata).fungible_store)
}
</code></pre>



</details>

<a id="0x1_asset_mapping_assign_to_account"></a>

## Function `assign_to_account`

Assigns tokens to a recipient account with proof verification
@param token_issuer - The token issuer signer
@param receiper - Recipient address
@param proove - Proof data for verification
@param amount - Amount of tokens to assign
Requirements:
- Valid proof must be provided
- Sufficient balance must exist


<pre><code><b>public</b> entry <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">assign_to_account</a>&lt;T&gt;(token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiper: <b>address</b>, proove: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">assign_to_account</a>&lt;T&gt;(
    token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    receiper: <b>address</b>,
    proove: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64
) <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
    <b>let</b> metadata = <a href="coin.md#0x1_coin_ensure_paired_metadata">coin::ensure_paired_metadata</a>&lt;T&gt;();
    <b>let</b> mapping_pool = <b>borrow_global_mut</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(token_issuer));
    <b>let</b> mapping_store = <a href="../../starcoin-stdlib/doc/smart_table.md#0x1_smart_table_borrow_mut">smart_table::borrow_mut</a>(&<b>mut</b> mapping_pool.token_stores, metadata);

    <b>assert</b>!(<a href="asset_mapping.md#0x1_asset_mapping_computer_poove">computer_poove</a>(proove), <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_NOT_PROOF">EINVALID_NOT_PROOF</a>));
    // <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(&store.transfer_ref, store.fungible_store, to_account_primary_store, amount);
    <b>let</b> store_signer = <a href="object.md#0x1_object_generate_signer_for_extending">object::generate_signer_for_extending</a>(&mapping_store.extend_ref);
    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(
        <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(receiper, metadata),
        <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(&store_signer, mapping_store.fungible_store, amount)
    )
}
</code></pre>



</details>

<a id="0x1_asset_mapping_computer_poove"></a>

## Function `computer_poove`

Computes and verifies the provided proof
@param proove - The proof data to verify
@returns Boolean indicating proof validity
Note: Current implementation returns true (TODO)


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_computer_poove">computer_poove</a>(proove: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_computer_poove">computer_poove</a>(proove: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) : bool {
    // TODO(BobOng): implement this function
    <b>true</b>
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
