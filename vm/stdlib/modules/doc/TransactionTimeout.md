
<a name="0x1_TransactionTimeout"></a>

# Module `0x1::TransactionTimeout`



-  [Resource <code><a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a></code>](#0x1_TransactionTimeout_TTL)
-  [Const <code><a href="TransactionTimeout.md#0x1_TransactionTimeout_ONE_DAY">ONE_DAY</a></code>](#0x1_TransactionTimeout_ONE_DAY)
-  [Function <code>initialize</code>](#0x1_TransactionTimeout_initialize)
-  [Function <code>set_timeout</code>](#0x1_TransactionTimeout_set_timeout)
-  [Function <code>is_valid_transaction_timestamp</code>](#0x1_TransactionTimeout_is_valid_transaction_timestamp)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>set_timeout</code>](#@Specification_0_set_timeout)
    -  [Function <code>is_valid_transaction_timestamp</code>](#@Specification_0_is_valid_transaction_timestamp)


<a name="0x1_TransactionTimeout_TTL"></a>

## Resource `TTL`



<pre><code><b>resource</b> <b>struct</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>
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

<a name="0x1_TransactionTimeout_ONE_DAY"></a>

## Const `ONE_DAY`



<pre><code><b>const</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_ONE_DAY">ONE_DAY</a>: u64 = 86400;
</code></pre>



<a name="0x1_TransactionTimeout_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer) {
  // Only callable by the Genesis address
  <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()));
  // Currently set <b>to</b> 1day.
  //TODO set by onchain config.
  move_to(account, <a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a> {duration_seconds: <a href="TransactionTimeout.md#0x1_TransactionTimeout_ONE_DAY">ONE_DAY</a>});
}
</code></pre>



</details>

<a name="0x1_TransactionTimeout_set_timeout"></a>

## Function `set_timeout`



<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64) <b>acquires</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a> {
  // Only callable by the Genesis address
  <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="Errors.md#0x1_Errors_ENOT_GENESIS_ACCOUNT">Errors::ENOT_GENESIS_ACCOUNT</a>()));

  <b>let</b> timeout = borrow_global_mut&lt;<a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
  timeout.duration_seconds = new_duration;
}
</code></pre>



</details>

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



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>global</b>&lt;<a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).duration_seconds == <a href="TransactionTimeout.md#0x1_TransactionTimeout_ONE_DAY">ONE_DAY</a>;
</code></pre>



<a name="@Specification_0_set_timeout"></a>

### Function `set_timeout`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_set_timeout">set_timeout</a>(account: &signer, new_duration: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> <b>global</b>&lt;<a href="TransactionTimeout.md#0x1_TransactionTimeout_TTL">TTL</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).duration_seconds == new_duration;
</code></pre>



<a name="@Specification_0_is_valid_transaction_timestamp"></a>

### Function `is_valid_transaction_timestamp`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionTimeout.md#0x1_TransactionTimeout_is_valid_transaction_timestamp">is_valid_transaction_timestamp</a>(txn_timestamp: u64): bool
</code></pre>




<pre><code>pragma verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">Block::BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
