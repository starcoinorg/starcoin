
<a name="0x1_Block"></a>

# Module `0x1::Block`

Block module provide metadata for generated blocks.


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


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_Block_BlockMetadata"></a>

## Resource `BlockMetadata`

Block metadata struct.


<pre><code><b>struct</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>number: u64</code>
</dt>
<dd>
 number of the current block
</dd>
<dt>
<code>parent_hash: vector&lt;u8&gt;</code>
</dt>
<dd>
 Hash of the parent block.
</dd>
<dt>
<code>author: <b>address</b></code>
</dt>
<dd>
 Author of the current block.
</dd>
<dt>
<code>uncles: u64</code>
</dt>
<dd>
 number of uncles.
</dd>
<dt>
<code>new_block_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">Block::NewBlockEvent</a>&gt;</code>
</dt>
<dd>
 Handle of events when new blocks are emitted
</dd>
</dl>


</details>

<a name="0x1_Block_NewBlockEvent"></a>

## Struct `NewBlockEvent`

Events emitted when new block generated.


<pre><code><b>struct</b> <a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a> <b>has</b> drop, store
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
<code>author: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>timestamp: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>uncles: u64</code>
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

This can only be invoked by the GENESIS_ACCOUNT at genesis


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_initialize">initialize</a>(account: &signer, parent_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_initialize">initialize</a>(account: &signer, parent_hash: vector&lt;u8&gt;) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>move_to</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(
        account,
        <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
            number: 0,
            parent_hash: parent_hash,
            author: <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
            uncles: 0,
            new_block_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">Self::NewBlockEvent</a>&gt;(account),
        });
}
</code></pre>



</details>

<a name="0x1_Block_get_current_block_number"></a>

## Function `get_current_block_number`

Get the current block number


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_block_number">get_current_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_block_number">get_current_block_number</a>(): u64 <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  <b>borrow_global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).number
}
</code></pre>



</details>

<a name="0x1_Block_get_parent_hash"></a>

## Function `get_parent_hash`

Get the hash of the parent block.


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_parent_hash">get_parent_hash</a>(): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_parent_hash">get_parent_hash</a>(): vector&lt;u8&gt; <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  *&<b>borrow_global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).parent_hash
}
</code></pre>



</details>

<a name="0x1_Block_get_current_author"></a>

## Function `get_current_author`

Gets the address of the author of the current block


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): <b>address</b> <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a> {
  <b>borrow_global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).author
}
</code></pre>



</details>

<a name="0x1_Block_process_block_metadata"></a>

## Function `process_block_metadata`

Call at block prologue


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;, author: <b>address</b>, timestamp: u64, uncles: u64, number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;,author: <b>address</b>, timestamp: u64, uncles:u64, number:u64) <b>acquires</b> <a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>{
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    <b>let</b> block_metadata_ref = <b>borrow_global_mut</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>assert</b>!(number == (block_metadata_ref.number + 1), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Block.md#0x1_Block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>));
    block_metadata_ref.number = number;
    block_metadata_ref.author= author;
    block_metadata_ref.parent_hash = parent_hash;
    block_metadata_ref.uncles = uncles;

    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a>&gt;(
      &<b>mut</b> block_metadata_ref.new_block_events,
      <a href="Block.md#0x1_Block_NewBlockEvent">NewBlockEvent</a> {
          number: number,
          author: author,
          timestamp: timestamp,
          uncles: uncles,
      }
    );
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
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
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


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_get_current_author">get_current_author</a>(): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_process_block_metadata"></a>

### Function `process_block_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="Block.md#0x1_Block_process_block_metadata">process_block_metadata</a>(account: &signer, parent_hash: vector&lt;u8&gt;, author: <b>address</b>, timestamp: u64, uncles: u64, number: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> number != <b>global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).number + 1;
</code></pre>




<a name="0x1_Block_AbortsIfBlockMetadataNotExist"></a>


<pre><code><b>schema</b> <a href="Block.md#0x1_Block_AbortsIfBlockMetadataNotExist">AbortsIfBlockMetadataNotExist</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
}
</code></pre>
