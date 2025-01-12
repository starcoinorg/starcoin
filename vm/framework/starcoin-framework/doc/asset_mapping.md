
<a id="0x1_asset_mapping"></a>

# Module `0x1::asset_mapping`

Asset Mapping Module
This module implements functionality for managing fungible asset mappings in the Starcoin framework.
It provides capabilities for creating stores, managing balances, and assigning assets to accounts
with proof verification.


-  [Resource `AssetMappingStore`](#0x1_asset_mapping_AssetMappingStore)
-  [Resource `AssetMappingStoreT`](#0x1_asset_mapping_AssetMappingStoreT)
-  [Resource `AssetMappingPool`](#0x1_asset_mapping_AssetMappingPool)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_asset_mapping_initialize)
-  [Function `create_store_from_coin`](#0x1_asset_mapping_create_store_from_coin)
-  [Function `create_store_for_coin_type`](#0x1_asset_mapping_create_store_for_coin_type)
-  [Function `fungible_store_balance`](#0x1_asset_mapping_fungible_store_balance)
-  [Function `assign_to_account_with_proof`](#0x1_asset_mapping_assign_to_account_with_proof)
-  [Function `assign_to_account`](#0x1_asset_mapping_assign_to_account)
-  [Function `calculation_proof`](#0x1_asset_mapping_calculation_proof)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="primary_fungible_store.md#0x1_primary_fungible_store">0x1::primary_fungible_store</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map">0x1::simple_map</a>;
<b>use</b> <a href="starcoin_proof.md#0x1_starcoin_proof_verifier">0x1::starcoin_proof_verifier</a>;
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
- fungible_metadata: The type of fungible assets


<pre><code>#[resource_group_member(#[group = <a href="object.md#0x1_object_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> <b>has</b> store, key
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
<dt>
<code>metadata: <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_asset_mapping_AssetMappingStoreT"></a>

## Resource `AssetMappingStoreT`



<pre><code><b>struct</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStoreT">AssetMappingStoreT</a>&lt;T&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;T&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>old_path_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_asset_mapping_AssetMappingPool"></a>

## Resource `AssetMappingPool`

AssetMappingCoinType represents a mapping that from old version token types to now version asset stores
eg. 0x1::STC::STC -> 0x1::starcoin_coin::STC


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
<code>token_mapping: <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_SimpleMap">simple_map::SimpleMap</a>&lt;<a href="../../move-stdlib/doc/string.md#0x1_string_String">string::String</a>, <b>address</b>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_asset_mapping_ASSET_MAPPING_OBJECT_SEED"></a>



<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_ASSET_MAPPING_OBJECT_SEED">ASSET_MAPPING_OBJECT_SEED</a>: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; = [97, 115, 115, 101, 116, 45, 109, 97, 112, 112, 105, 110, 103];
</code></pre>



<a id="0x1_asset_mapping_EINVALID_ASSET_MAPPING_POOL"></a>



<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_ASSET_MAPPING_POOL">EINVALID_ASSET_MAPPING_POOL</a>: u64 = 104;
</code></pre>



<a id="0x1_asset_mapping_EINVALID_NOT_PROOF"></a>



<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_NOT_PROOF">EINVALID_NOT_PROOF</a>: u64 = 103;
</code></pre>



<a id="0x1_asset_mapping_EINVALID_PROOF_ROOT"></a>



<pre><code><b>const</b> <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_PROOF_ROOT">EINVALID_PROOF_ROOT</a>: u64 = 102;
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
Verifies the framework signer and creates a new AssetMappingPool


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proof_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_initialize">initialize</a>(framework: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proof_root: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;){
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(framework) == <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>)
    );
    <b>move_to</b>(framework, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
        token_mapping: <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_new">simple_map::new</a>(),
        proof_root,
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


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">create_store_from_coin</a>&lt;T: key&gt;(token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;T&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">create_store_from_coin</a>&lt;T: key&gt;(
    token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="coin.md#0x1_coin">coin</a>: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;T&gt;
) <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">asset_mapping::create_store_from_coin</a> | entered"));

    <b>let</b> token_issuer_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(token_issuer);
    <b>assert</b>!(
        token_issuer_addr == <a href="stc_util.md#0x1_stc_util_token_issuer">stc_util::token_issuer</a>&lt;T&gt;(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>)
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">asset_mapping::create_store_from_coin</a> | coin_to_fungible_asset"));

    <b>let</b> <a href="fungible_asset.md#0x1_fungible_asset">fungible_asset</a> = <a href="coin.md#0x1_coin_coin_to_fungible_asset">coin::coin_to_fungible_asset</a>(<a href="coin.md#0x1_coin">coin</a>);

    <b>let</b> (
        metadata,
        fungible_store,
        extend_ref
    ) = <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_coin_type">create_store_for_coin_type</a>&lt;T&gt;(token_issuer);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">asset_mapping::create_store_from_coin</a> | created token store"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&fungible_store);

    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(fungible_store, <a href="fungible_asset.md#0x1_fungible_asset">fungible_asset</a>);

    // Add token mapping <a href="coin.md#0x1_coin">coin</a> type
    <b>let</b> asset_coin_type =
        <b>borrow_global_mut</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());

    <b>let</b> store_constructor_ref = &<a href="object.md#0x1_object_create_object">object::create_object</a>(<a href="system_addresses.md#0x1_system_addresses_get_core_resource_address">system_addresses::get_core_resource_address</a>());
    <b>let</b> store_signer = &<a href="object.md#0x1_object_generate_signer">object::generate_signer</a>(store_constructor_ref);
    <b>move_to</b>(store_signer, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> {
        extend_ref,
        fungible_store,
        metadata,
    });

    <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_add">simple_map::add</a>(
        &<b>mut</b> asset_coin_type.token_mapping,
        <a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(old_token_str),
        <a href="object.md#0x1_object_address_from_constructor_ref">object::address_from_constructor_ref</a>(store_constructor_ref),
    );

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_create_store_from_coin">asset_mapping::create_store_from_coin</a> | exited"));
}
</code></pre>



</details>

<a id="0x1_asset_mapping_create_store_for_coin_type"></a>

## Function `create_store_for_coin_type`

Creates a store for a specific token type
@param framework - The framework signer
@returns (metadata, store, extend_ref):
- metadata: Token metadata object
- store: Created fungible store
- extend_ref: Extension reference for the store


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_coin_type">create_store_for_coin_type</a>&lt;T&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (<a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_Metadata">fungible_asset::Metadata</a>&gt;, <a href="object.md#0x1_object_Object">object::Object</a>&lt;<a href="fungible_asset.md#0x1_fungible_asset_FungibleStore">fungible_asset::FungibleStore</a>&gt;, <a href="object.md#0x1_object_ExtendRef">object::ExtendRef</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_create_store_for_coin_type">create_store_for_coin_type</a>&lt;T&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): (Object&lt;Metadata&gt;, Object&lt;FungibleStore&gt;, ExtendRef) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"asset_mapping::create_store_for_type | entered"));

    <b>let</b> metadata = <a href="coin.md#0x1_coin_ensure_paired_metadata">coin::ensure_paired_metadata</a>&lt;T&gt;();
    <b>let</b> construct_ref = <a href="object.md#0x1_object_create_object_from_account">object::create_object_from_account</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> store = <a href="fungible_asset.md#0x1_fungible_asset_create_store">fungible_asset::create_store</a>(&construct_ref, metadata);

    // Generate extend reference
    <b>let</b> extend_ref = <a href="object.md#0x1_object_generate_extend_ref">object::generate_extend_ref</a>(&construct_ref);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"asset_mapping::create_store_for_type | exited"));

    (metadata, store, extend_ref)
}
</code></pre>



</details>

<a id="0x1_asset_mapping_fungible_store_balance"></a>

## Function `fungible_store_balance`

Retrieves the balance for a specific token type
@returns Current balance of the token in the mapping pool


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_fungible_store_balance">fungible_store_balance</a>(old_asset_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_fungible_store_balance">fungible_store_balance</a>(old_asset_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;): u64 <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> {
    <b>let</b> pool = <b>borrow_global</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> store_object_addr = <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_borrow">simple_map::borrow</a>(&pool.token_mapping, &<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(old_asset_str));
    <b>let</b> mapping_store = <b>borrow_global</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a>&gt;(*store_object_addr);
    <a href="fungible_asset.md#0x1_fungible_asset_balance">fungible_asset::balance</a>(mapping_store.fungible_store)
}
</code></pre>



</details>

<a id="0x1_asset_mapping_assign_to_account_with_proof"></a>

## Function `assign_to_account_with_proof`



<pre><code><b>public</b> entry <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account_with_proof">assign_to_account_with_proof</a>(token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiper: <b>address</b>, old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof_path_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof_value_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof_siblings: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account_with_proof">assign_to_account_with_proof</a>(
    token_issuer: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    receiper: <b>address</b>,
    old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof_path_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof_value_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof_siblings: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64
) <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> {
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_PROOF_ROOT">EINVALID_PROOF_ROOT</a>)
    );

    // Verify that the token type of the request mapping is the passed-in verification type
    <b>assert</b>!(
        <a href="asset_mapping.md#0x1_asset_mapping_calculation_proof">calculation_proof</a>(proof_path_hash, proof_value_hash, <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_split">starcoin_proof_verifier::split</a>(proof_siblings)),
        <a href="../../move-stdlib/doc/error.md#0x1_error_unauthenticated">error::unauthenticated</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_NOT_PROOF">EINVALID_NOT_PROOF</a>)
    );

    <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">assign_to_account</a>(token_issuer, receiper, old_token_str, amount);
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


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">assign_to_account</a>(system_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver: <b>address</b>, old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">assign_to_account</a>(
    system_account: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    receiver: <b>address</b>,
    old_token_str: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u64
) <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>, <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | entered"));

    <b>let</b> account_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(system_account);
    <b>assert</b>!(
        <a href="system_addresses.md#0x1_system_addresses_is_starcoin_framework_address">system_addresses::is_starcoin_framework_address</a>(account_addr) ||
            <a href="system_addresses.md#0x1_system_addresses_is_core_resource_address">system_addresses::is_core_resource_address</a>(account_addr),
        <a href="asset_mapping.md#0x1_asset_mapping_EINVALID_SIGNER">EINVALID_SIGNER</a>
    );

    <b>assert</b>!(
        <b>exists</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="asset_mapping.md#0x1_asset_mapping_EINVALID_ASSET_MAPPING_POOL">EINVALID_ASSET_MAPPING_POOL</a>)
    );

    <b>let</b> coin_type_mapping =
        <b>borrow_global_mut</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | coin_type_mapping"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&coin_type_mapping.token_mapping);

    <b>let</b> mapping_store_addr = <a href="../../starcoin-stdlib/doc/simple_map.md#0x1_simple_map_borrow">simple_map::borrow</a>(&coin_type_mapping.token_mapping, &<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(old_token_str));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(mapping_store_addr);
    <b>let</b> mapping_store = <b>borrow_global</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingStore">AssetMappingStore</a>&gt;(*mapping_store_addr);

    // <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | metadata"));
    // <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="fungible_asset.md#0x1_fungible_asset_is_frozen">fungible_asset::is_frozen</a>(mapping_store.fungible_store));

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>"));
    <b>let</b> mapping_fa = <a href="fungible_asset.md#0x1_fungible_asset_withdraw">fungible_asset::withdraw</a>(
        &<a href="object.md#0x1_object_generate_signer_for_extending">object::generate_signer_for_extending</a>(&mapping_store.extend_ref),
        mapping_store.fungible_store,
        amount
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | Getting receiver fungible store: "));

    <b>let</b> target_store =
        <a href="primary_fungible_store.md#0x1_primary_fungible_store_ensure_primary_store_exists">primary_fungible_store::ensure_primary_store_exists</a>(receiver, mapping_store.metadata);

    <a href="fungible_asset.md#0x1_fungible_asset_deposit">fungible_asset::deposit</a>(target_store, mapping_fa);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="../../move-stdlib/doc/string.md#0x1_string_utf8">string::utf8</a>(b"<a href="asset_mapping.md#0x1_asset_mapping_assign_to_account">asset_mapping::assign_to_account</a> | exited"));
}
</code></pre>



</details>

<a id="0x1_asset_mapping_calculation_proof"></a>

## Function `calculation_proof`

Computes and verifies the provided proof


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_calculation_proof">calculation_proof</a>(proof_path_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, blob_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, proof_siblings: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="asset_mapping.md#0x1_asset_mapping_calculation_proof">calculation_proof</a>(
    proof_path_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    blob_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    proof_siblings: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;
): bool <b>acquires</b> <a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a> {
    <b>let</b> expect_proof_root =
        <b>borrow_global_mut</b>&lt;<a href="asset_mapping.md#0x1_asset_mapping_AssetMappingPool">AssetMappingPool</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).proof_root;
    <b>let</b> actual_root = <a href="starcoin_proof.md#0x1_starcoin_proof_verifier_computer_root_hash">starcoin_proof_verifier::computer_root_hash</a>(
        proof_path_hash,
        blob_hash,
        proof_siblings
    );
    expect_proof_root == actual_root
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
