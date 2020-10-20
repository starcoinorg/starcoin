
<a name="0x1_Block"></a>

# Module `0x1::Block`



-  [Resource `BlockMetadata`](#0x1_Block_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_Block_NewBlockEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_Block_initialize)
-  [Function `get_current_block_number`](#0x1_Block_get_current_block_number)
-  [Function `get_parent_hash`](#0x1_Block_get_parent_hash)
-  [Function `get_current_author`](#0x1_Block_get_current_author)
-  [Function `process_block_metadata`](#0x1_Block_process_block_metadata)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `get_current_block_number`](#@Specification_1_get_current_block_number)
    -  [Function `get_parent_hash`](#@Specification_1_get_parent_hash)
    -  [Function `get_current_author`](#@Specification_1_get_current_author)
    -  [Function `process_block_metadata`](#@Specification_1_process_block_metadata)


<pre><code><b>use</b> <a href="ConsensusConfig.md#0x1_ConsensusConfig">0x1::ConsensusConfig</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_Block_BlockMetadata"></a>

## Resource `BlockMetadata`



<pre><code><b>resource</b> <b>struct</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>parent_hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>author: address</code>
</dt>
<dd>

</dd>
<dt>
<code>new_block_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">Block::NewBlockEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Block_NewBlockEvent"></a>

## Struct `NewBlockEvent`



<pre><code><b>struct</b> <a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>author: address</code>
</dt>
<dd>

</dd>
<dt>
<code>timestamp: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Block_EBLOCK_NUMBER_MISMATCH"></a>



<pre><code><b>const</b> <a href="Block.md#0x1_Block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>: u64 = 17;
</code></pre>



<a name="0x1_Block_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_initialize">initialize</a>(account: &signer, parent_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_initialize">initialize</a>(account: &signer, parent_hash: vector&lt;u8&gt;) {
  <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
  <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

  move_to&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(
      account,
  <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
    number: 0,
    parent_hash: parent_hash,
    author: <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
    new_block_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">Self::NewBlockEvent</a>&gt;(account),
  });
}
</code></pre>



</details>

<a name="0x1_Block_get_current_block_number"></a>

## Function `get_current_block_number`



<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_block_number">get_current_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_block_number">get_current_block_number</a>(): u64 <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  borrow_global&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).number
}
</code></pre>



</details>

<a name="0x1_Block_get_parent_hash"></a>

## Function `get_parent_hash`



<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_parent_hash">get_parent_hash</a>(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_parent_hash">get_parent_hash</a>(): vector&lt;u8&gt; <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  *&borrow_global&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).parent_hash
}
</code></pre>



</details>

<a name="0x1_Block_get_current_author"></a>

## Function `get_current_author`



<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): address <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  borrow_global&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).author
}
</code></pre>



</details>

<a name="0x1_Block_process_block_metadata"></a>

## Function `process_block_metadata`



<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;, author: address, timestamp: u64, uncles: u64, number: u64, parent_gas_used: u64): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;,author: address, timestamp: u64, uncles:u64, number:u64, parent_gas_used:u64): u128 <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>{
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>let</b> block_metadata_ref = borrow_global_mut&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>assert</b>(number == (block_metadata_ref.number + 1), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Block.md#0x1_Block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>));
    block_metadata_ref.number = number;
    block_metadata_ref.author= author;
    block_metadata_ref.parent_hash = parent_hash;

    <b>let</b> reward = <a href="ConsensusConfig.md#0x1_ConsensusConfig_adjust_epoch">ConsensusConfig::adjust_epoch</a>(account, number, timestamp, uncles, parent_gas_used);

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a>&gt;(
      &<b>mut</b> block_metadata_ref.new_block_events,
      <a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a> {
        number: number,
        author: author,
        timestamp: timestamp,
      }
    );
    reward
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_initialize">initialize</a>(account: &signer, parent_hash: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_1_get_current_block_number"></a>

### Function `get_current_block_number`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_block_number">get_current_block_number</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_get_parent_hash"></a>

### Function `get_parent_hash`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_parent_hash">get_parent_hash</a>(): vector&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_get_current_author"></a>

### Function `get_current_author`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): address
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_process_block_metadata"></a>

### Function `process_block_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;, author: address, timestamp: u64, uncles: u64, number: u64, parent_gas_used: u64): u128
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> number != <b>global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).number + 1;
</code></pre>
