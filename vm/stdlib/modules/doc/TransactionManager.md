
<a name="0x1_TransactionManager"></a>

# Module `0x1::TransactionManager`



-  [Const <code><a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a></code>](#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT)
-  [Const <code><a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a></code>](#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE)
-  [Function <code>prologue</code>](#0x1_TransactionManager_prologue)
-  [Function <code>epilogue</code>](#0x1_TransactionManager_epilogue)
-  [Function <code>block_prologue</code>](#0x1_TransactionManager_block_prologue)
-  [Function <code>distribute</code>](#0x1_TransactionManager_distribute)


<a name="0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT"></a>

## Const `TXN_PAYLOAD_TYPE_SCRIPT`



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>: u8 = 0;
</code></pre>



<a name="0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE"></a>

## Const `TXN_PAYLOAD_TYPE_PACKAGE`



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>: u8 = 1;
</code></pre>



<a name="0x1_TransactionManager_prologue"></a>

## Function `prologue`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_prologue">prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8, txn_payload_type: u8, txn_script_or_package_hash: vector&lt;u8&gt;, txn_package_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_prologue">prologue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_public_key: vector&lt;u8&gt;,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    txn_expiration_time: u64,
    chain_id: u8,
    txn_payload_type: u8,
    txn_script_or_package_hash: vector&lt;u8&gt;,
    txn_package_address: address,
) {
    // Can only be invoked by genesis account
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">Errors::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>()),
    );
    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_BAD_CHAIN_ID">Errors::PROLOGUE_BAD_CHAIN_ID</a>()));
    <a href="Account.md#0x1_Account_txn_prologue">Account::txn_prologue</a>&lt;TokenType&gt;(
        account,
        txn_sender,
        txn_sequence_number,
        txn_public_key,
        txn_gas_price,
        txn_max_gas_units,
    );
    <b>assert</b>(
        <a href="TransactionTimeout.md#0x1_TransactionTimeout_is_valid_transaction_timestamp">TransactionTimeout::is_valid_transaction_timestamp</a>(txn_expiration_time),
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_TRANSACTION_EXPIRED">Errors::PROLOGUE_TRANSACTION_EXPIRED</a>()),
    );
    <b>if</b> (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>) {
        <b>assert</b>(
            <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_module_allowed">TransactionPublishOption::is_module_allowed</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_MODULE_NOT_ALLOWED">Errors::PROLOGUE_MODULE_NOT_ALLOWED</a>()),
        );
        <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">PackageTxnManager::package_txn_prologue</a>(
            account,
            txn_sender,
            txn_package_address,
            txn_script_or_package_hash,
        );
    } <b>else</b> <b>if</b> (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>) {
        <b>assert</b>(
            <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_script_allowed">TransactionPublishOption::is_script_allowed</a>(
                <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
                &txn_script_or_package_hash,
            ),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_SCRIPT_NOT_ALLOWED">Errors::PROLOGUE_SCRIPT_NOT_ALLOWED</a>()),
        );
    };
}
</code></pre>



</details>

<a name="0x1_TransactionManager_epilogue"></a>

## Function `epilogue`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_epilogue">epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, txn_payload_type: u8, _txn_script_or_package_hash: vector&lt;u8&gt;, txn_package_address: address, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_epilogue">epilogue</a>&lt;TokenType&gt;(
    account: &signer,
    txn_sender: address,
    txn_sequence_number: u64,
    txn_gas_price: u64,
    txn_max_gas_units: u64,
    gas_units_remaining: u64,
    txn_payload_type: u8,
    _txn_script_or_package_hash: vector&lt;u8&gt;,
    txn_package_address: address,
    // txn execute success or fail.
    success: bool,
) {
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()),
    );
    <a href="Account.md#0x1_Account_txn_epilogue">Account::txn_epilogue</a>&lt;TokenType&gt;(
        account,
        txn_sender,
        txn_sequence_number,
        txn_gas_price,
        txn_max_gas_units,
        gas_units_remaining,
    );
    <b>if</b> (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>) {
        <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">PackageTxnManager::package_txn_epilogue</a>(
            account,
            txn_sender,
            txn_package_address,
            success,
        );
    }
}
</code></pre>



</details>

<a name="0x1_TransactionManager_block_prologue"></a>

## Function `block_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_block_prologue">block_prologue</a>(account: &signer, parent_hash: vector&lt;u8&gt;, timestamp: u64, author: address, public_key_vec: vector&lt;u8&gt;, uncles: u64, number: u64, chain_id: u8, parent_gas_used: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_block_prologue">block_prologue</a>(
    account: &signer,
    parent_hash: vector&lt;u8&gt;,
    timestamp: u64,
    author: address,
    public_key_vec: vector&lt;u8&gt;,
    uncles: u64,
    number: u64,
    chain_id: u8,
    parent_gas_used: u64,
) {
    // Can only be invoked by genesis account
    <b>assert</b>(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()),
    );
    <a href="Timestamp.md#0x1_Timestamp_update_global_time">Timestamp::update_global_time</a>(account, timestamp);
    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Errors.md#0x1_Errors_PROLOGUE_BAD_CHAIN_ID">Errors::PROLOGUE_BAD_CHAIN_ID</a>()));
    //get previous author for distribute txn_fee
    <b>let</b> previous_author = <a href="Block.md#0x1_Block_get_current_author">Block::get_current_author</a>();
    <b>let</b> txn_fee = <a href="TransactionFee.md#0x1_TransactionFee_distribute_transaction_fees">TransactionFee::distribute_transaction_fees</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>(account, txn_fee, previous_author);
    <b>let</b> reward = <a href="Block.md#0x1_Block_process_block_metadata">Block::process_block_metadata</a>(
        account,
        parent_hash,
        author,
        timestamp,
        uncles,
        number,
        parent_gas_used,
    );
    <a href="BlockReward.md#0x1_BlockReward_process_block_reward">BlockReward::process_block_reward</a>(account, number, reward, author, public_key_vec);
}
</code></pre>



</details>

<a name="0x1_TransactionManager_distribute"></a>

## Function `distribute`



<pre><code><b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>&lt;TokenType&gt;(account: &signer, txn_fee: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, author: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>&lt;TokenType&gt;(account: &signer, txn_fee: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, author: address) {
    <b>let</b> value = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&txn_fee);
    <b>if</b> (value &gt; 0) {
        <a href="Account.md#0x1_Account_deposit_to">Account::deposit_to</a>&lt;TokenType&gt;(account, author, txn_fee);
    } <b>else</b> {
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>&lt;TokenType&gt;(txn_fee);
    }
}
</code></pre>



</details>
