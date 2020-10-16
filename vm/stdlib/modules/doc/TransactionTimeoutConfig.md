
<a name="0x1_TransactionTimeoutConfig"></a>

# Module `0x1::TransactionTimeoutConfig`



-  [Struct <code><a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a></code>](#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig)
-  [Function <code>initialize</code>](#0x1_TransactionTimeoutConfig_initialize)
-  [Function <code>new_transaction_timeout_config</code>](#0x1_TransactionTimeoutConfig_new_transaction_timeout_config)
-  [Function <code>get_transaction_timeout_config</code>](#0x1_TransactionTimeoutConfig_get_transaction_timeout_config)
-  [Function <code>duration_seconds</code>](#0x1_TransactionTimeoutConfig_duration_seconds)


<a name="0x1_TransactionTimeoutConfig_TransactionTimeoutConfig"></a>

## Struct `TransactionTimeoutConfig`



<pre><code><b>struct</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>duration_seconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TransactionTimeoutConfig_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_initialize">initialize</a>(account: &signer, duration_seconds: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_initialize">initialize</a>(account: &signer, duration_seconds: u64) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS">Errors::ENOT_GENESIS</a>()));
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()));

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">Self::TransactionTimeoutConfig</a>&gt;(
        account,
        <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds)
    );
}
</code></pre>



</details>

<a name="0x1_TransactionTimeoutConfig_new_transaction_timeout_config"></a>

## Function `new_transaction_timeout_config`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64): <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64) : <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a> {
    <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a> {duration_seconds: duration_seconds}
}
</code></pre>



</details>

<a name="0x1_TransactionTimeoutConfig_get_transaction_timeout_config"></a>

## Function `get_transaction_timeout_config`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a> {
    <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_TransactionTimeoutConfig_duration_seconds"></a>

## Function `duration_seconds`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_duration_seconds">duration_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_duration_seconds">duration_seconds</a>() :u64 {
    <b>let</b> config = <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_get_transaction_timeout_config">get_transaction_timeout_config</a>();
    config.duration_seconds
}
</code></pre>



</details>
