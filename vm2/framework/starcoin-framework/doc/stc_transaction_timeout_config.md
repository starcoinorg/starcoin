
<a id="0x1_stc_transaction_timeout_config"></a>

# Module `0x1::stc_transaction_timeout_config`

Onchain configuration for timeout setting of transaction.


-  [Struct `TransactionTimeoutConfig`](#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig)
-  [Function `initialize`](#0x1_stc_transaction_timeout_config_initialize)
-  [Function `new_transaction_timeout_config`](#0x1_stc_transaction_timeout_config_new_transaction_timeout_config)
-  [Function `get_transaction_timeout_config`](#0x1_stc_transaction_timeout_config_get_transaction_timeout_config)
-  [Function `duration_seconds`](#0x1_stc_transaction_timeout_config_duration_seconds)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `new_transaction_timeout_config`](#@Specification_0_new_transaction_timeout_config)
    -  [Function `get_transaction_timeout_config`](#@Specification_0_get_transaction_timeout_config)
    -  [Function `duration_seconds`](#@Specification_0_duration_seconds)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_stc_transaction_timeout_config_TransactionTimeoutConfig"></a>

## Struct `TransactionTimeoutConfig`

config structs.


<pre><code><b>struct</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>duration_seconds: u64</code>
</dt>
<dd>
 timeout in second.
</dd>
</dl>


</details>

<a id="0x1_stc_transaction_timeout_config_initialize"></a>

## Function `initialize`

Initialize function. Should only be called in genesis.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, duration_seconds: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, duration_seconds: u64) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">Self::TransactionTimeoutConfig</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds)
    );
}
</code></pre>



</details>

<a id="0x1_stc_transaction_timeout_config_new_transaction_timeout_config"></a>

## Function `new_transaction_timeout_config`

Create a new timeout config used in dao proposal.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a> {
    <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a> { duration_seconds: duration_seconds }
}
</code></pre>



</details>

<a id="0x1_stc_transaction_timeout_config_get_transaction_timeout_config"></a>

## Function `get_transaction_timeout_config`

Get current timeout config.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a> {
    <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>())
}
</code></pre>



</details>

<a id="0x1_stc_transaction_timeout_config_duration_seconds"></a>

## Function `duration_seconds`

Get current txn timeout in seconds.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">duration_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">duration_seconds</a>(): u64 {
    <b>let</b> config = <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_get_transaction_timeout_config">get_transaction_timeout_config</a>();
    config.duration_seconds
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, duration_seconds: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">on_chain_config::PublishNewConfigAbortsIf</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">on_chain_config::PublishNewConfigEnsures</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt;;
</code></pre>



<a id="@Specification_0_new_transaction_timeout_config"></a>

### Function `new_transaction_timeout_config`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_new_transaction_timeout_config">new_transaction_timeout_config</a>(duration_seconds: u64): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_0_get_transaction_timeout_config"></a>

### Function `get_transaction_timeout_config`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_get_transaction_timeout_config">get_transaction_timeout_config</a>(): <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt; {
    addr: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
};
</code></pre>



<a id="@Specification_0_duration_seconds"></a>

### Function `duration_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">duration_seconds</a>(): u64
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt; {
    addr: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
};
</code></pre>




<a id="0x1_stc_transaction_timeout_config_AbortsIfTxnTimeoutConfigNotExist"></a>


<pre><code><b>schema</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_AbortsIfTxnTimeoutConfigNotExist">AbortsIfTxnTimeoutConfigNotExist</a> {
    <b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">TransactionTimeoutConfig</a>&gt; {
        addr: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    };
}
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
