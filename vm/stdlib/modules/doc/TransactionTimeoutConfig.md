
<a name="0x1_TransactionTimeoutConfig"></a>

# Module `0x1::TransactionTimeoutConfig`



-  [Struct `TransactionTimeoutConfig`](#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig)
-  [Function `initialize`](#0x1_TransactionTimeoutConfig_initialize)
-  [Function `new_transaction_timeout_config`](#0x1_TransactionTimeoutConfig_new_transaction_timeout_config)
-  [Function `get_transaction_timeout_config`](#0x1_TransactionTimeoutConfig_get_transaction_timeout_config)
-  [Function `duration_seconds`](#0x1_TransactionTimeoutConfig_duration_seconds)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `new_transaction_timeout_config`](#@Specification_0_new_transaction_timeout_config)
    -  [Function `get_transaction_timeout_config`](#@Specification_0_get_transaction_timeout_config)
    -  [Function `duration_seconds`](#@Specification_0_duration_seconds)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



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
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

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

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_initialize">initialize</a>(account: &signer, duration_seconds: u64)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">Config::PublishNewConfigAbortsIf</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;;
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">Config::PublishNewConfigEnsures</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;;
</code></pre>



<a name="@Specification_0_new_transaction_timeout_config"></a>

### Function `new_transaction_timeout_config`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64): <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_0_get_transaction_timeout_config"></a>

### Function `get_transaction_timeout_config`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_TransactionTimeoutConfig">TransactionTimeoutConfig::TransactionTimeoutConfig</a>
</code></pre>




<pre><code><b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;{
    addr: <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()
};
</code></pre>



<a name="@Specification_0_duration_seconds"></a>

### Function `duration_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_duration_seconds">duration_seconds</a>(): u64
</code></pre>




<pre><code><b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;{
    addr: <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()
};
</code></pre>




<a name="0x1_TransactionTimeoutConfig_AbortsIfTxnTimeoutConfigNotExist"></a>


<pre><code><b>schema</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_AbortsIfTxnTimeoutConfigNotExist">AbortsIfTxnTimeoutConfigNotExist</a> {
    <b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;{
        addr: <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()
    };
}
</code></pre>
