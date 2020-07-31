
<a name="0x1_TransactionTimeout"></a>

# Module `0x1::TransactionTimeout`

### Table of Contents

-  [Resource `TTL`](#0x1_TransactionTimeout_TTL)
-  [Function `initialize`](#0x1_TransactionTimeout_initialize)
-  [Function `set_timeout`](#0x1_TransactionTimeout_set_timeout)
-  [Function `is_valid_transaction_timestamp`](#0x1_TransactionTimeout_is_valid_transaction_timestamp)
-  [Specification](#0x1_TransactionTimeout_Specification)
    -  [Function `initialize`](#0x1_TransactionTimeout_Specification_initialize)
    -  [Function `set_timeout`](#0x1_TransactionTimeout_Specification_set_timeout)



<a name="0x1_TransactionTimeout_TTL"></a>

## Resource `TTL`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_TransactionTimeout_TTL">TTL</a>
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

<a name="0x1_TransactionTimeout_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer) {
  // Only callable by the Genesis address
  <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
  // Currently set <b>to</b> 1day.
  //TODO set by onchain config.
  move_to(account, <a href="#0x1_TransactionTimeout_TTL">TTL</a> {duration_seconds: ONE_DAY});
}
</code></pre>



</details>

<a name="0x1_TransactionTimeout_set_timeout"></a>

## Function `set_timeout`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64) <b>acquires</b> <a href="#0x1_TransactionTimeout_TTL">TTL</a> {
  // Only callable by the Genesis address
  <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

  <b>let</b> timeout = borrow_global_mut&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
  timeout.duration_seconds = new_duration;
}
</code></pre>



</details>

<a name="0x1_TransactionTimeout_is_valid_transaction_timestamp"></a>

## Function `is_valid_transaction_timestamp`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool <b>acquires</b> <a href="#0x1_TransactionTimeout_TTL">TTL</a> {
  <b>let</b> current_block_time = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
  <b>let</b> block_number = <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>();
  // before first block, just require txn_timestamp &gt; genesis timestamp.
  <b>if</b> (block_number == 0) {
    <b>return</b> txn_timestamp &gt; current_block_time
  };
  <b>let</b> timeout = borrow_global&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>()).duration_seconds;
  <b>let</b> max_txn_time = current_block_time + timeout;
  current_block_time &lt; txn_timestamp && txn_timestamp &lt; max_txn_time
}
</code></pre>



</details>

<a name="0x1_TransactionTimeout_Specification"></a>

## Specification



<pre><code>pragma verify = <b>false</b>;
</code></pre>



<a name="0x1_TransactionTimeout_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>();
<b>aborts_if</b> exists&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).duration_seconds == ONE_DAY;
</code></pre>



<a name="0x1_TransactionTimeout_Specification_set_timeout"></a>

### Function `set_timeout`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>();
<b>aborts_if</b> !exists&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>());
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).duration_seconds == new_duration;
</code></pre>
