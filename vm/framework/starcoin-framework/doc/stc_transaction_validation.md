
<a id="0x1_stc_transaction_validation"></a>

# Module `0x1::stc_transaction_validation`

<code>starcoin transaction validation</code> manages:
1. prologue and epilogue of transactions.
2. prologue of blocks.


-  [Constants](#@Constants_0)
-  [Function `prologue`](#0x1_stc_transaction_validation_prologue)
-  [Function `epilogue`](#0x1_stc_transaction_validation_epilogue)
-  [Function `txn_prologue`](#0x1_stc_transaction_validation_txn_prologue)
-  [Function `txn_epilogue`](#0x1_stc_transaction_validation_txn_epilogue)
-  [Specification](#@Specification_1)
    -  [Function `prologue`](#@Specification_1_prologue)
    -  [Function `epilogue`](#@Specification_1_epilogue)
    -  [Function `txn_epilogue`](#@Specification_1_txn_epilogue)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="fungible_asset.md#0x1_fungible_asset">0x1::fungible_asset</a>;
<b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="object.md#0x1_object">0x1::object</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee">0x1::stc_transaction_fee</a>;
<b>use</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation">0x1::stc_transaction_package_validation</a>;
<b>use</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout">0x1::stc_transaction_timeout</a>;
<b>use</b> <a href="stc_util.md#0x1_stc_util">0x1::stc_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option">0x1::transaction_publish_option</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_transaction_validation_MAX_U64"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_MAX_U64">MAX_U64</a>: u128 = 18446744073709551615;
</code></pre>



<a id="0x1_stc_transaction_validation_EINSUFFICIENT_BALANCE"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>: u64 = 1010;
</code></pre>



<a id="0x1_stc_transaction_validation_EBAD_TRANSACTION_FEE_TOKEN"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>: u64 = 1018;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>: u64 = 1000;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_BAD_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>: u64 = 1006;
</code></pre>



<a id="0x1_stc_transaction_validation_ECOIN_DEPOSIT_IS_ZERO"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_ECOIN_DEPOSIT_IS_ZERO">ECOIN_DEPOSIT_IS_ZERO</a>: u64 = 1015;
</code></pre>



<a id="0x1_stc_transaction_validation_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 1019;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_CANT_PAY_GAS_DEPOSIT"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>: u64 = 1004;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>: u64 = 1001;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_MODULE_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_MODULE_NOT_ALLOWED">EPROLOGUE_MODULE_NOT_ALLOWED</a>: u64 = 1007;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_SCRIPT_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SCRIPT_NOT_ALLOWED">EPROLOGUE_SCRIPT_NOT_ALLOWED</a>: u64 = 1008;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG">EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG</a>: u64 = 1009;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW">EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>: u64 = 1003;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD">EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>: u64 = 1002;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_SIGNER_ALREADY_DELEGATED"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SIGNER_ALREADY_DELEGATED">EPROLOGUE_SIGNER_ALREADY_DELEGATED</a>: u64 = 1200;
</code></pre>



<a id="0x1_stc_transaction_validation_EPROLOGUE_TRANSACTION_EXPIRED"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_TRANSACTION_EXPIRED">EPROLOGUE_TRANSACTION_EXPIRED</a>: u64 = 1005;
</code></pre>



<a id="0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>: u8 = 1;
</code></pre>



<a id="0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>: u8 = 0;
</code></pre>



<a id="0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION"></a>



<pre><code><b>const</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION">TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION</a>: u8 = 2;
</code></pre>



<a id="0x1_stc_transaction_validation_prologue"></a>

## Function `prologue`

The prologue is invoked at the beginning of every transaction
It verifies:
- The account's auth key matches the transaction's public key
- That the account has enough balance to pay for all of the gas
- That the sequence number matches the transaction's sequence key


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_prologue">prologue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, txn_payload_type: u8, txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_package_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_prologue">prologue</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8,
    txn_payload_type: u8,
    txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_package_address: <b>address</b>,
) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"transaction_validation::prologue | Entered"));

    // Can only be invoked by genesis <a href="account.md#0x1_account">account</a>
    // <b>assert</b>!(
    //     <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&<a href="account.md#0x1_account">account</a>) == <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
    //     error::requires_address(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>),
    // );
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(&<a href="account.md#0x1_account">account</a>);

    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>!(<a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="chain_id.md#0x1_chain_id">chain_id</a>, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>));

    <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(
        &<a href="account.md#0x1_account">account</a>,
        txn_sender,
        txn_sequence_number,
        txn_authentication_key_preimage,
        txn_gas_price,
        txn_max_gas_units,
    );

    <b>assert</b>!(
        <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_is_valid_transaction_timestamp">stc_transaction_timeout::is_valid_transaction_timestamp</a>(txn_expiration_time),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_TRANSACTION_EXPIRED">EPROLOGUE_TRANSACTION_EXPIRED</a>),
    );

    <b>if</b> (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>) {
        // stdlib upgrade is not affected by PublishOption
        <b>if</b> (txn_package_address != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()) {
            <b>assert</b>!(
                <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_module_allowed">transaction_publish_option::is_module_allowed</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&<a href="account.md#0x1_account">account</a>)),
                <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_MODULE_NOT_ALLOWED">EPROLOGUE_MODULE_NOT_ALLOWED</a>),
            );
        };
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_prologue_v2">stc_transaction_package_validation::package_txn_prologue_v2</a>(
            &<a href="account.md#0x1_account">account</a>,
            txn_sender,
            txn_package_address,
            txn_script_or_package_hash,
        );
    } <b>else</b> <b>if</b> (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>) {
        <b>assert</b>!(
            <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_script_allowed">transaction_publish_option::is_script_allowed</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&<a href="account.md#0x1_account">account</a>), ),
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SCRIPT_NOT_ALLOWED">EPROLOGUE_SCRIPT_NOT_ALLOWED</a>),
        );
    };
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"transaction_validation::prologue | Exited"));
    // do nothing for <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION">TXN_PAYLOAD_TYPE_SCRIPT_FUNCTION</a>
}
</code></pre>



</details>

<a id="0x1_stc_transaction_validation_epilogue"></a>

## Function `epilogue`

Migration from old StarcoinFramework TransactionManager::epilogue
The epilogue is invoked at the end of transactions.
It collects gas and bumps the sequence number


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_epilogue">epilogue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, txn_payload_type: u8, _txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_package_address: <b>address</b>, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_epilogue">epilogue</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
    txn_payload_type: u8,
    _txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_package_address: <b>address</b>,
    // txn execute success or fail.
    success: bool,
) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_epilogue">stc_transaction_validation::epilogue</a> | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(&<a href="account.md#0x1_account">account</a>);
    <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(
        &<a href="account.md#0x1_account">account</a>,
        txn_sender,
        txn_sequence_number,
        txn_authentication_key_preimage,
        txn_gas_price,
        txn_max_gas_units,
        gas_units_remaining,
    );
    <b>if</b> (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>) {
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_epilogue">stc_transaction_package_validation::package_txn_epilogue</a>(
            &<a href="account.md#0x1_account">account</a>,
            txn_sender,
            txn_package_address,
            success,
        );
    };

    <b>let</b> metadata = <a href="coin.md#0x1_coin_paired_metadata">coin::paired_metadata</a>&lt;STC&gt;();
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&metadata), 10000);
    <b>let</b> metdata_obj = <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(metadata);
    <b>assert</b>!(<a href="object.md#0x1_object_is_object">object::is_object</a>(<a href="object.md#0x1_object_object_address">object::object_address</a>(&metdata_obj)), 10001);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_epilogue">stc_transaction_validation::epilogue</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_stc_transaction_validation_txn_prologue"></a>

## Function `txn_prologue`

Migration from old StarcoinFramework Account::txn_prologue


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_prologue">txn_prologue</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    txn_sender: <b>address</b>,
    txn_sequence_number: u64,
    txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"transaction_validation::txn_prologue | Entered"));
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    // Verify that the transaction sender's <a href="account.md#0x1_account">account</a> <b>exists</b>
    <b>assert</b>!(<a href="account.md#0x1_account_exists_at">account::exists_at</a>(txn_sender), <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>));
    // Verify the <a href="account.md#0x1_account">account</a> <b>has</b> not delegate its <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a> cap.
    <b>assert</b>!(
        !<a href="account.md#0x1_account_is_signer_capability_offered">account::is_signer_capability_offered</a>(txn_sender),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SIGNER_ALREADY_DELEGATED">EPROLOGUE_SIGNER_ALREADY_DELEGATED</a>)
    );

    // txn_authentication_key_preimage <b>to</b> be check
    // Load the transaction sender's <a href="account.md#0x1_account">account</a>
    <b>if</b> (<a href="account.md#0x1_account_is_account_zero_auth_key">account::is_account_zero_auth_key</a>(txn_sender)) {
        // <b>if</b> sender's auth key is empty, <b>use</b> <b>address</b> <b>as</b> auth key for check transaction.
        <b>assert</b>!(
            <a href="account.md#0x1_account_auth_key_to_address">account::auth_key_to_address</a>(<a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(txn_authentication_key_preimage)) == txn_sender,
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>)
        );
    } <b>else</b> {
        // Check that the <a href="../../move-stdlib/doc/hash.md#0x1_hash">hash</a> of the transaction's <b>public</b> key matches the <a href="account.md#0x1_account">account</a>'s auth key
        <b>assert</b>!(
            //<a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(txn_authentication_key_preimage) == *&sender_account.authentication_key,
            <a href="account.md#0x1_account_get_authentication_key">account::get_authentication_key</a>(txn_sender) == <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(txn_authentication_key_preimage),
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY">EPROLOGUE_INVALID_ACCOUNT_AUTH_KEY</a>)
        );
    };

    <b>assert</b>!(
        (txn_gas_price <b>as</b> u128) * (txn_max_gas_units <b>as</b> u128) &lt;= <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_MAX_U64">MAX_U64</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>),
    );

    <b>let</b> max_transaction_fee = txn_gas_price * txn_max_gas_units;
    <b>if</b> (max_transaction_fee &gt; 0) {
        <b>assert</b>!(
            <a href="stc_util.md#0x1_stc_util_is_stc">stc_util::is_stc</a>&lt;TokenType&gt;(),
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EBAD_TRANSACTION_FEE_TOKEN">EBAD_TRANSACTION_FEE_TOKEN</a>)
        );

        <b>let</b> balance_amount = <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;TokenType&gt;(txn_sender);
        <b>assert</b>!(balance_amount &gt;= max_transaction_fee, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_CANT_PAY_GAS_DEPOSIT">EPROLOGUE_CANT_PAY_GAS_DEPOSIT</a>));

        <b>assert</b>!(
            (txn_sequence_number <b>as</b> u128) &lt; <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_MAX_U64">MAX_U64</a>,
            <a href="../../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG">EPROLOGUE_SEQUENCE_NUMBER_TOO_BIG</a>)
        );
    };
    <b>let</b> account_sequence_number = <a href="account.md#0x1_account_get_sequence_number">account::get_sequence_number</a>(txn_sender);
    // Check that the transaction sequence number matches the sequence number of the <a href="account.md#0x1_account">account</a>
    <b>assert</b>!(
        txn_sequence_number &gt;= account_sequence_number,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD">EPROLOGUE_SEQUENCE_NUMBER_TOO_OLD</a>)
    );
    <b>assert</b>!(
        txn_sequence_number == account_sequence_number,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW">EPROLOGUE_SEQUENCE_NUMBER_TOO_NEW</a>)
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"transaction_validation::txn_prologue | Exited"));
}
</code></pre>



</details>

<a id="0x1_stc_transaction_validation_txn_epilogue"></a>

## Function `txn_epilogue`

Migration from old StarcoinFramework Account::txn_eiplogue
The epilogue is invoked at the end of transactions.
It collects gas and bumps the sequence number


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, _txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    txn_sender: <b>address</b>,
    _txn_sequence_number: u64,
    txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
) {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    // Charge for gas
    <b>let</b> transaction_fee_amount = (txn_gas_price * (txn_max_gas_units - gas_units_remaining) <b>as</b> u128);
    <b>assert</b>!(
        <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;STC&gt;(txn_sender) &gt;= (transaction_fee_amount <b>as</b> u64),
        <a href="../../move-stdlib/doc/error.md#0x1_error_out_of_range">error::out_of_range</a>(<a href="stc_transaction_validation.md#0x1_stc_transaction_validation_EINSUFFICIENT_BALANCE">EINSUFFICIENT_BALANCE</a>)
    );

    // Bump the sequence number
    <a href="account.md#0x1_account_increment_sequence_number">account::increment_sequence_number</a>(txn_sender);

    // Set auth key when user send transaction first.
    <b>if</b> (<a href="account.md#0x1_account_is_account_zero_auth_key">account::is_account_zero_auth_key</a>(txn_sender) &&
        !<a href="../../move-stdlib/doc/vector.md#0x1_vector_is_empty">vector::is_empty</a>(&txn_authentication_key_preimage)) {
        <a href="account.md#0x1_account_rotate_authentication_key_internal">account::rotate_authentication_key_internal</a>(
            &<a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(txn_sender),
            <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(txn_authentication_key_preimage)
        )
    };

    <b>if</b> (transaction_fee_amount &gt; 0) {
        <b>let</b> transaction_fee = <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>&lt;STC&gt;(
            &<a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(txn_sender),
            (transaction_fee_amount <b>as</b> u64)
        );
        <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">stc_transaction_fee::pay_fee</a>(transaction_fee);
    };
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_1_prologue"></a>

### Function `prologue`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_prologue">prologue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, txn_payload_type: u8, txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_package_address: <b>address</b>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="chain_id.md#0x1_chain_id_ChainId">chain_id::ChainId</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() != <a href="chain_id.md#0x1_chain_id">chain_id</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(txn_sender);
<b>aborts_if</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(txn_authentication_key_preimage) != <b>global</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(
    txn_sender
).authentication_key;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; max_u64();
<b>include</b> <a href="stc_block.md#0x1_stc_block_AbortsIfBlockMetadataNotExist">stc_block::AbortsIfBlockMetadataNotExist</a>;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && !<b>exists</b>&lt;<a href="coin.md#0x1_coin_CoinStore">coin::CoinStore</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; 0 && txn_sequence_number &gt;= max_u64();
<b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(txn_sender).sequence_number;
<b>aborts_if</b> txn_sequence_number != <b>global</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(txn_sender).sequence_number;
<b>include</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_AbortsIfTimestampNotValid">stc_transaction_timeout::AbortsIfTimestampNotValid</a>;
<b>aborts_if</b> !<a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_spec_is_valid_transaction_timestamp">stc_transaction_timeout::spec_is_valid_transaction_timestamp</a>(txn_expiration_time);
<b>include</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_AbortsIfTxnPublishOptionNotExistWithBool">transaction_publish_option::AbortsIfTxnPublishOptionNotExistWithBool</a> {
    is_script_or_package: (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a> || txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>),
};
<b>aborts_if</b> txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>
    && txn_package_address != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    && !<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_spec_is_module_allowed">transaction_publish_option::spec_is_module_allowed</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>
    && !<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_spec_is_script_allowed">transaction_publish_option::spec_is_script_allowed</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIfWithType">stc_transaction_package_validation::CheckPackageTxnAbortsIfWithType</a> {
    is_package: (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>),
    sender: txn_sender,
    package_address: txn_package_address,
    package_hash: txn_script_or_package_hash
};
</code></pre>



<a id="@Specification_1_epilogue"></a>

### Function `epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_epilogue">epilogue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, txn_payload_type: u8, _txn_script_or_package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_package_address: <b>address</b>, success: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(txn_sender);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="coin.md#0x1_coin_CoinStore">coin::CoinStore</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
<b>aborts_if</b> txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u64();
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_AbortsIfPackageTxnEpilogue">stc_transaction_package_validation::AbortsIfPackageTxnEpilogue</a> {
    is_package: (txn_payload_type == <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>),
    package_address: txn_package_address,
    success,
};
</code></pre>



<a id="@Specification_1_txn_epilogue"></a>

### Function `txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_validation.md#0x1_stc_transaction_validation_txn_epilogue">txn_epilogue</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, _txn_sequence_number: u64, txn_authentication_key_preimage: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="account.md#0x1_account_Account">account::Account</a>&gt;(txn_sender);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="coin.md#0x1_coin_CoinStore">coin::CoinStore</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> _txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
