
<a name="0x1_TransactionTimeout"></a>

# Module `0x1::TransactionTimeout`



-  [Function `is_valid_transaction_timestamp`](#0x1_TransactionTimeout_is_valid_transaction_timestamp)
-  [Specification](#@Specification_0)
    -  [Function `is_valid_transaction_timestamp`](#@Specification_0_is_valid_transaction_timestamp)


<pre><code><b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig">0x1::TransactionTimeoutConfig</a>;
</code></pre>



<a name="0x1_TransactionTimeout_is_valid_transaction_timestamp"></a>

## Function `is_valid_transaction_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool {
  <b>let</b> current_block_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
  <b>let</b> block_number = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>();
  // before first block, just require txn_timestamp &gt; genesis timestamp.
  <b>if</b> (block_number == 0) {
    <b>return</b> txn_timestamp &gt; current_block_time
  };
  <b>let</b> timeout = <a href="TransactionTimeoutConfig.md#0x1_TransactionTimeoutConfig_duration_seconds">TransactionTimeoutConfig::duration_seconds</a>();
  <b>let</b> max_txn_time = current_block_time + timeout;
  current_block_time &lt; txn_timestamp && txn_timestamp &lt; max_txn_time
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_is_valid_transaction_timestamp"></a>

### Function `is_valid_transaction_timestamp`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">Block::BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
