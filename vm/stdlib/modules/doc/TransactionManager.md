
<a name="0x1_TransactionManager"></a>

# Module `0x1::TransactionManager`



-  [Constants](#@Constants_0)
-  [Function `prologue`](#0x1_TransactionManager_prologue)
-  [Function `epilogue`](#0x1_TransactionManager_epilogue)
-  [Function `block_prologue`](#0x1_TransactionManager_block_prologue)
-  [Function `distribute`](#0x1_TransactionManager_distribute)
-  [Specification](#@Specification_1)
    -  [Function `prologue`](#@Specification_1_prologue)
    -  [Function `epilogue`](#@Specification_1_epilogue)
    -  [Function `block_prologue`](#@Specification_1_block_prologue)
    -  [Function `distribute`](#@Specification_1_distribute)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="BlockReward.md#0x1_BlockReward">0x1::BlockReward</a>;
<b>use</b> <a href="ChainId.md#0x1_ChainId">0x1::ChainId</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Epoch.md#0x1_Epoch">0x1::Epoch</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager">0x1::PackageTxnManager</a>;
<b>use</b> <a href="STC.md#0x1_STC">0x1::STC</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="TransactionFee.md#0x1_TransactionFee">0x1::TransactionFee</a>;
<b>use</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">0x1::TransactionPublishOption</a>;
<b>use</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout">0x1::TransactionTimeout</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_TransactionManager_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>: u64 = 0;
</code></pre>



<a name="0x1_TransactionManager_EPROLOGUE_BAD_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>: u64 = 6;
</code></pre>



<a name="0x1_TransactionManager_EPROLOGUE_MODULE_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_MODULE_NOT_ALLOWED">EPROLOGUE_MODULE_NOT_ALLOWED</a>: u64 = 7;
</code></pre>



<a name="0x1_TransactionManager_EPROLOGUE_SCRIPT_NOT_ALLOWED"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_SCRIPT_NOT_ALLOWED">EPROLOGUE_SCRIPT_NOT_ALLOWED</a>: u64 = 8;
</code></pre>



<a name="0x1_TransactionManager_EPROLOGUE_TRANSACTION_EXPIRED"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_TRANSACTION_EXPIRED">EPROLOGUE_TRANSACTION_EXPIRED</a>: u64 = 5;
</code></pre>



<a name="0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>: u8 = 1;
</code></pre>



<a name="0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT"></a>



<pre><code><b>const</b> <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>: u8 = 0;
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
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>),
    );
    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>));
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
        <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_TRANSACTION_EXPIRED">EPROLOGUE_TRANSACTION_EXPIRED</a>),
    );
    <b>if</b> (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>) {
        <b>assert</b>(
            <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_module_allowed">TransactionPublishOption::is_module_allowed</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_MODULE_NOT_ALLOWED">EPROLOGUE_MODULE_NOT_ALLOWED</a>),
        );
        <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">PackageTxnManager::package_txn_prologue</a>(
            account,
            txn_package_address,
            txn_script_or_package_hash,
        );
    } <b>else</b> <b>if</b> (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>) {
        <b>assert</b>(
            <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_script_allowed">TransactionPublishOption::is_script_allowed</a>(
                <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account),
                &txn_script_or_package_hash,
            ),
            <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_SCRIPT_NOT_ALLOWED">EPROLOGUE_SCRIPT_NOT_ALLOWED</a>),
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
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
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



<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_block_prologue">block_prologue</a>(account: &signer, parent_hash: vector&lt;u8&gt;, timestamp: u64, author: address, auth_key_vec: vector&lt;u8&gt;, uncles: u64, number: u64, chain_id: u8, parent_gas_used: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_block_prologue">block_prologue</a>(
    account: &signer,
    parent_hash: vector&lt;u8&gt;,
    timestamp: u64,
    author: address,
    auth_key_vec: vector&lt;u8&gt;,
    uncles: u64,
    number: u64,
    chain_id: u8,
    parent_gas_used: u64,
) {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <a href="Timestamp.md#0x1_Timestamp_update_global_time">Timestamp::update_global_time</a>(account, timestamp);
    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>(<a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() == chain_id, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="TransactionManager.md#0x1_TransactionManager_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>));
    //get previous author for distribute txn_fee
    <b>let</b> previous_author = <a href="Block.md#0x1_Block_get_current_author">Block::get_current_author</a>();
    <b>let</b> txn_fee = <a href="TransactionFee.md#0x1_TransactionFee_distribute_transaction_fees">TransactionFee::distribute_transaction_fees</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
    <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>(txn_fee, previous_author);
    <a href="Block.md#0x1_Block_process_block_metadata">Block::process_block_metadata</a>(
        account,
        parent_hash,
        author,
        timestamp,
        uncles,
        number,
    );
    <b>let</b> reward = <a href="Epoch.md#0x1_Epoch_adjust_epoch">Epoch::adjust_epoch</a>(account, number, timestamp, uncles, parent_gas_used);
    <a href="BlockReward.md#0x1_BlockReward_process_block_reward">BlockReward::process_block_reward</a>(account, number, reward, author, auth_key_vec);
}
</code></pre>



</details>

<a name="0x1_TransactionManager_distribute"></a>

## Function `distribute`



<pre><code><b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>&lt;TokenType&gt;(txn_fee: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, author: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>&lt;TokenType&gt;(txn_fee: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;, author: address) {
    <b>let</b> value = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&txn_fee);
    <b>if</b> (value &gt; 0) {
        <a href="Account.md#0x1_Account_deposit">Account::deposit</a>&lt;TokenType&gt;(author, txn_fee);
    } <b>else</b> {
        <a href="Token.md#0x1_Token_destroy_zero">Token::destroy_zero</a>&lt;TokenType&gt;(txn_fee);
    }
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_prologue"></a>

### Function `prologue`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_prologue">prologue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_public_key: vector&lt;u8&gt;, txn_gas_price: u64, txn_max_gas_units: u64, txn_expiration_time: u64, chain_id: u8, txn_payload_type: u8, txn_script_or_package_hash: vector&lt;u8&gt;, txn_package_address: address)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="ChainId.md#0x1_ChainId_ChainId">ChainId::ChainId</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
<b>aborts_if</b> <a href="ChainId.md#0x1_ChainId_get">ChainId::get</a>() != chain_id;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(txn_sender);
<b>aborts_if</b> <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(txn_public_key) != <b>global</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(txn_sender).authentication_key;
<b>aborts_if</b> txn_gas_price * txn_max_gas_units &gt; max_u64();
<b>include</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfTimestampNotExists">Timestamp::AbortsIfTimestampNotExists</a>;
<b>include</b> <a href="Block.md#0x1_Block_AbortsIfBlockMetadataNotExist">Block::AbortsIfBlockMetadataNotExist</a>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> <b>global</b>&lt;<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;&gt;(txn_sender).token.value &lt; txn_gas_price * txn_max_gas_units;
<b>aborts_if</b> txn_sequence_number &lt; <b>global</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(txn_sender).sequence_number;
<b>aborts_if</b> txn_sequence_number != <b>global</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(txn_sender).sequence_number;
<b>include</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_AbortsIfTimestampNotValid">TransactionTimeout::AbortsIfTimestampNotValid</a>;
<b>aborts_if</b> !<a href="TransactionTimeout.md#0x1_TransactionTimeout_spec_is_valid_transaction_timestamp">TransactionTimeout::spec_is_valid_transaction_timestamp</a>(txn_expiration_time);
<b>include</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_AbortsIfTxnPublishOptionNotExistWithBool">TransactionPublishOption::AbortsIfTxnPublishOptionNotExistWithBool</a> {
    is_script_or_package: (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a> || txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a>),
};
<b>aborts_if</b> txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a> && !<a href="TransactionPublishOption.md#0x1_TransactionPublishOption_spec_is_module_allowed">TransactionPublishOption::spec_is_module_allowed</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_SCRIPT">TXN_PAYLOAD_TYPE_SCRIPT</a> && !<a href="TransactionPublishOption.md#0x1_TransactionPublishOption_spec_is_script_allowed">TransactionPublishOption::spec_is_script_allowed</a>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account), txn_script_or_package_hash);
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIfWithType">PackageTxnManager::CheckPackageTxnAbortsIfWithType</a>{is_package: (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>), sender:txn_sender, package_address: txn_package_address, package_hash: txn_script_or_package_hash};
</code></pre>



<a name="@Specification_1_epilogue"></a>

### Function `epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_epilogue">epilogue</a>&lt;TokenType&gt;(account: &signer, txn_sender: address, txn_sequence_number: u64, txn_gas_price: u64, txn_max_gas_units: u64, gas_units_remaining: u64, txn_payload_type: u8, _txn_script_or_package_hash: vector&lt;u8&gt;, txn_package_address: address, success: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>include</b> <a href="CoreAddresses.md#0x1_CoreAddresses_AbortsIfNotGenesisAddress">CoreAddresses::AbortsIfNotGenesisAddress</a>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Account">Account::Account</a>&gt;(txn_sender);
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Account.md#0x1_Account_Balance">Account::Balance</a>&lt;TokenType&gt;&gt;(txn_sender);
<b>aborts_if</b> txn_max_gas_units &lt; gas_units_remaining;
<b>aborts_if</b> txn_sequence_number + 1 &gt; max_u64();
<b>aborts_if</b> txn_gas_price * (txn_max_gas_units - gas_units_remaining) &gt; max_u64();
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_AbortsIfPackageTxnEpilogue">PackageTxnManager::AbortsIfPackageTxnEpilogue</a> {
    is_package: (txn_payload_type == <a href="TransactionManager.md#0x1_TransactionManager_TXN_PAYLOAD_TYPE_PACKAGE">TXN_PAYLOAD_TYPE_PACKAGE</a>),
    package_address: txn_package_address,
    success: success,
};
</code></pre>



<a name="@Specification_1_block_prologue"></a>

### Function `block_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_block_prologue">block_prologue</a>(account: &signer, parent_hash: vector&lt;u8&gt;, timestamp: u64, author: address, auth_key_vec: vector&lt;u8&gt;, uncles: u64, number: u64, chain_id: u8, parent_gas_used: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_distribute"></a>

### Function `distribute`


<pre><code><b>fun</b> <a href="TransactionManager.md#0x1_TransactionManager_distribute">distribute</a>&lt;TokenType&gt;(txn_fee: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;, author: address)
</code></pre>




<pre><code><b>include</b> <a href="Account.md#0x1_Account_AbortsIfDepositWithMetadata">Account::AbortsIfDepositWithMetadata</a>&lt;TokenType&gt;{
    value_is_not_zero: (<a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(txn_fee) &gt; 0),
    receiver: author,
    to_deposit: txn_fee,
};
</code></pre>
