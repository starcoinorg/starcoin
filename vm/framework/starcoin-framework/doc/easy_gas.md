
<a id="0x1_easy_gas"></a>

# Module `0x1::easy_gas`



-  [Struct `STCToken`](#0x1_easy_gas_STCToken)
-  [Resource `GasTokenEntry`](#0x1_easy_gas_GasTokenEntry)
-  [Resource `GasFeeAddress`](#0x1_easy_gas_GasFeeAddress)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_easy_gas_initialize)
-  [Function `register_oracle`](#0x1_easy_gas_register_oracle)
-  [Function `init_oracle_source`](#0x1_easy_gas_init_oracle_source)
-  [Function `update_oracle`](#0x1_easy_gas_update_oracle)
-  [Function `get_scaling_factor`](#0x1_easy_gas_get_scaling_factor)
-  [Function `gas_oracle_read`](#0x1_easy_gas_gas_oracle_read)
-  [Function `register_gas_token`](#0x1_easy_gas_register_gas_token)
-  [Function `get_data_source_address`](#0x1_easy_gas_get_data_source_address)
-  [Function `create_gas_fee_address`](#0x1_easy_gas_create_gas_fee_address)
-  [Function `get_gas_fee_address`](#0x1_easy_gas_get_gas_fee_address)
-  [Function `withdraw_gas_fee`](#0x1_easy_gas_withdraw_gas_fee)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../move-stdlib/doc/bcs.md#0x1_bcs">0x1::bcs</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="oracle_price.md#0x1_oracle_price">0x1::oracle_price</a>;
<b>use</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer">0x1::reserved_accounts_signer</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info">0x1::type_info</a>;
</code></pre>



<a id="0x1_easy_gas_STCToken"></a>

## Struct `STCToken`



<pre><code><b>struct</b> <a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType: store&gt; <b>has</b> <b>copy</b>, drop, store
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

<a id="0x1_easy_gas_GasTokenEntry"></a>

## Resource `GasTokenEntry`



<pre><code><b>struct</b> <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> <b>has</b> drop, store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>module_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>struct_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>data_source: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_easy_gas_GasFeeAddress"></a>

## Resource `GasFeeAddress`



<pre><code><b>struct</b> <a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>gas_fee_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>cap: <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_easy_gas_EBAD_TRANSACTION_FEE_TOKEN"></a>



<pre><code><b>const</b> <a href="easy_gas.md#0x1_easy_gas_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>: u64 = 18;
</code></pre>



<a id="0x1_easy_gas_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_initialize">initialize</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, token_account_address: <b>address</b>, token_module_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, token_struct_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data_source: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_initialize">initialize</a>(
    sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    token_account_address: <b>address</b>,
    token_module_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    token_struct_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    data_source: <b>address</b>,
) <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> {
    <a href="easy_gas.md#0x1_easy_gas_register_gas_token">register_gas_token</a>(sender, token_account_address, token_module_name, token_struct_name, data_source);
    <a href="easy_gas.md#0x1_easy_gas_create_gas_fee_address">create_gas_fee_address</a>(sender);
}
</code></pre>



</details>

<a id="0x1_easy_gas_register_oracle"></a>

## Function `register_oracle`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_register_oracle">register_oracle</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_register_oracle">register_oracle</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8) {
    <a href="oracle_price.md#0x1_oracle_price_register_oracle">oracle_price::register_oracle</a>&lt;<a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType&gt;&gt;(sender, precision);
    // <b>let</b> genesis_account =
    //     <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">reserved_accounts_signer::get_stored_signer</a>(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    // // todo:check gas token entry
    // <a href="coin.md#0x1_coin_register">coin::register</a>&lt;TokenType&gt;(&genesis_account);
}
</code></pre>



</details>

<a id="0x1_easy_gas_init_oracle_source"></a>

## Function `init_oracle_source`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_init_oracle_source">init_oracle_source</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_init_oracle_source">init_oracle_source</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128) {
    <a href="oracle_price.md#0x1_oracle_price_init_data_source">oracle_price::init_data_source</a>&lt;<a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType&gt;&gt;(sender, init_value);
}
</code></pre>



</details>

<a id="0x1_easy_gas_update_oracle"></a>

## Function `update_oracle`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_update_oracle">update_oracle</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_update_oracle">update_oracle</a>&lt;TokenType: store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128) {
    <a href="oracle_price.md#0x1_oracle_price_update">oracle_price::update</a>&lt;<a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType&gt;&gt;(sender, value);
}
</code></pre>



</details>

<a id="0x1_easy_gas_get_scaling_factor"></a>

## Function `get_scaling_factor`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_scaling_factor">get_scaling_factor</a>&lt;TokenType: store&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_scaling_factor">get_scaling_factor</a>&lt;TokenType: store&gt;(): u128 {
    <a href="oracle_price.md#0x1_oracle_price_get_scaling_factor">oracle_price::get_scaling_factor</a>&lt;<a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType&gt;&gt;()
}
</code></pre>



</details>

<a id="0x1_easy_gas_gas_oracle_read"></a>

## Function `gas_oracle_read`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_gas_oracle_read">gas_oracle_read</a>&lt;TokenType: store&gt;(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_gas_oracle_read">gas_oracle_read</a>&lt;TokenType: store&gt;(): u128 <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> {
    <b>let</b> data_source = <a href="easy_gas.md#0x1_easy_gas_get_data_source_address">get_data_source_address</a>&lt;TokenType&gt;();
    <a href="oracle_price.md#0x1_oracle_price_read">oracle_price::read</a>&lt;<a href="easy_gas.md#0x1_easy_gas_STCToken">STCToken</a>&lt;TokenType&gt;&gt;(data_source)
}
</code></pre>



</details>

<a id="0x1_easy_gas_register_gas_token"></a>

## Function `register_gas_token`



<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_register_gas_token">register_gas_token</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, account_address: <b>address</b>, module_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, struct_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, data_source: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_register_gas_token">register_gas_token</a>(
    sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    account_address: <b>address</b>,
    module_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    struct_name: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    data_source: <b>address</b>,
) <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(sender);

    <b>let</b> genesis_account =
        <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">reserved_accounts_signer::get_stored_signer</a>(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> gas_token_entry = <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> { account_address, module_name, struct_name, data_source };
    <b>if</b> (<b>exists</b>&lt;<a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&genesis_account))) {
        <b>move_from</b>&lt;<a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&genesis_account));
    };
    <b>move_to</b>(&genesis_account, gas_token_entry);
}
</code></pre>



</details>

<a id="0x1_easy_gas_get_data_source_address"></a>

## Function `get_data_source_address`



<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_data_source_address">get_data_source_address</a>&lt;TokenType: store&gt;(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_data_source_address">get_data_source_address</a>&lt;TokenType: store&gt;(): <b>address</b> <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a> {
    <b>let</b> token_type_info = <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_type_of">type_info::type_of</a>&lt;TokenType&gt;();
    <b>let</b> <a href="genesis.md#0x1_genesis">genesis</a> = <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
    <b>let</b> gas_token_entry = <b>borrow_global</b>&lt;<a href="easy_gas.md#0x1_easy_gas_GasTokenEntry">GasTokenEntry</a>&gt;(<a href="genesis.md#0x1_genesis">genesis</a>);
    <b>assert</b>!(<a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_module_name">type_info::module_name</a>(&token_type_info) == *&gas_token_entry.module_name
        && <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_account_address">type_info::account_address</a>(&token_type_info) == *&gas_token_entry.account_address
        && <a href="../../starcoin-stdlib/doc/type_info.md#0x1_type_info_struct_name">type_info::struct_name</a>(&token_type_info) == *&gas_token_entry.struct_name,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="easy_gas.md#0x1_easy_gas_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>)
    );
    gas_token_entry.data_source
}
</code></pre>



</details>

<a id="0x1_easy_gas_create_gas_fee_address"></a>

## Function `create_gas_fee_address`



<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_create_gas_fee_address">create_gas_fee_address</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="easy_gas.md#0x1_easy_gas_create_gas_fee_address">create_gas_fee_address</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(sender);
    <b>let</b> genesis_account =
        <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">reserved_accounts_signer::get_stored_signer</a>(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> (gas_fee_signer, cap) = <a href="account.md#0x1_account_create_resource_account">account::create_resource_account</a>(
        &genesis_account,
        <a href="../../move-stdlib/doc/bcs.md#0x1_bcs_to_bytes">bcs::to_bytes</a>(&<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender))
    );
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;STC&gt;(&gas_fee_signer);
    //<b>let</b> gas_fee_signer = account::create_signer_with_cap(&cap);
    // account::set_auto_accept_token(&gas_fee_signer, <b>true</b>);
    <b>move_to</b>(&genesis_account, <a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a> {
        gas_fee_address: <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&gas_fee_signer),
        cap
    });
}
</code></pre>



</details>

<a id="0x1_easy_gas_get_gas_fee_address"></a>

## Function `get_gas_fee_address`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_gas_fee_address">get_gas_fee_address</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_get_gas_fee_address">get_gas_fee_address</a>(): <b>address</b> <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a> {
    <b>borrow_global</b>&lt;<a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).gas_fee_address
}
</code></pre>



</details>

<a id="0x1_easy_gas_withdraw_gas_fee"></a>

## Function `withdraw_gas_fee`



<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_withdraw_gas_fee">withdraw_gas_fee</a>&lt;TokenType: store&gt;(_sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="easy_gas.md#0x1_easy_gas_withdraw_gas_fee">withdraw_gas_fee</a>&lt;TokenType: store&gt;(_sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128) <b>acquires</b> <a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a> {
    <b>let</b> gas_fee_address_entry =
        <b>borrow_global</b>&lt;<a href="easy_gas.md#0x1_easy_gas_GasFeeAddress">GasFeeAddress</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> gas_fee_signer = <a href="account.md#0x1_account_create_signer_with_capability">account::create_signer_with_capability</a>(&gas_fee_address_entry.cap);
    // <b>let</b> withdraw_cap = extract_withdraw_capability(&gas_fee_signer);
    // <b>let</b> token = withdraw_with_capability&lt;TokenType&gt;(&withdraw_cap, amount);
    // restore_withdraw_capability(withdraw_cap);
    // deposit(CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), token);

    <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(
        <a href="system_addresses.md#0x1_system_addresses_get_core_resource_address">system_addresses::get_core_resource_address</a>(),
        <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>&lt;TokenType&gt;(&gas_fee_signer, (amount <b>as</b> u64))
    );
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
