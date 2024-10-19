
<a id="0x1_stc_transaction_timeout"></a>

# Module `0x1::stc_transaction_timeout`

A module used to check expiration time of transactions.


-  [Function `is_valid_transaction_timestamp`](#0x1_stc_transaction_timeout_is_valid_transaction_timestamp)
-  [Specification](#@Specification_0)
    -  [Function `is_valid_transaction_timestamp`](#@Specification_0_is_valid_transaction_timestamp)


<pre><code><b>use</b> <a href="stc_block.md#0x1_stc_block">0x1::stc_block</a>;
<b>use</b> <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config">0x1::stc_transaction_timeout_config</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_stc_transaction_timeout_is_valid_transaction_timestamp"></a>

## Function `is_valid_transaction_timestamp`

Check whether the given timestamp is valid for transactions.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool {
    <b>let</b> current_block_time = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <b>let</b> block_number = <a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>();
    // before first <a href="block.md#0x1_block">block</a>, just require txn_timestamp &gt; <a href="genesis.md#0x1_genesis">genesis</a> <a href="timestamp.md#0x1_timestamp">timestamp</a>.
    <b>if</b> (block_number == 0) {
        <b>return</b> txn_timestamp &gt; current_block_time
    };
    <b>let</b> timeout = <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">stc_transaction_timeout_config::duration_seconds</a>();
    <b>let</b> max_txn_time = current_block_time + timeout;
    current_block_time &lt; txn_timestamp && txn_timestamp &lt; max_txn_time
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>




<a id="0x1_stc_transaction_timeout_spec_is_valid_transaction_timestamp"></a>


<pre><code><b>fun</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_spec_is_valid_transaction_timestamp">spec_is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool {
   <b>if</b> (<a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>() == 0) {
       txn_timestamp &gt; <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>()
   } <b>else</b> {
       <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() &lt; txn_timestamp && txn_timestamp &lt;
           (<a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() + <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">stc_transaction_timeout_config::duration_seconds</a>())
   }
}
</code></pre>



<a id="@Specification_0_is_valid_transaction_timestamp"></a>

### Function `is_valid_transaction_timestamp`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">stc_block::BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>() != 0
    && <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() + <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">stc_transaction_timeout_config::duration_seconds</a>() &gt; max_u64();
<b>aborts_if</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>() != 0
    && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>&gt;&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
);
</code></pre>




<a id="0x1_stc_transaction_timeout_AbortsIfTimestampNotValid"></a>


<pre><code><b>schema</b> <a href="stc_transaction_timeout.md#0x1_stc_transaction_timeout_AbortsIfTimestampNotValid">AbortsIfTimestampNotValid</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">stc_block::BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>aborts_if</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>() != 0 && <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>(
    ) + <a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_duration_seconds">stc_transaction_timeout_config::duration_seconds</a>() &gt; max_u64();
    <b>aborts_if</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">stc_block::get_current_block_number</a>(
    ) != 0 && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_transaction_timeout_config.md#0x1_stc_transaction_timeout_config_TransactionTimeoutConfig">stc_transaction_timeout_config::TransactionTimeoutConfig</a>&gt;&gt;(
        <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    );
}
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
