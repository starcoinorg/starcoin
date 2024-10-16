
<a id="0x1_stc_block"></a>

# Module `0x1::stc_block`

Block module provide metadata for generated blocks.


-  [Resource `BlockMetadata`](#0x1_stc_block_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_stc_block_NewBlockEvent)
-  [Struct `Checkpoint`](#0x1_stc_block_Checkpoint)
-  [Resource `Checkpoints`](#0x1_stc_block_Checkpoints)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_stc_block_initialize)
-  [Function `get_current_block_number`](#0x1_stc_block_get_current_block_number)
-  [Function `get_parent_hash`](#0x1_stc_block_get_parent_hash)
-  [Function `get_current_author`](#0x1_stc_block_get_current_author)
-  [Function `process_block_metadata`](#0x1_stc_block_process_block_metadata)
-  [Function `checkpoints_init`](#0x1_stc_block_checkpoints_init)
-  [Function `checkpoint_entry`](#0x1_stc_block_checkpoint_entry)
-  [Function `checkpoint`](#0x1_stc_block_checkpoint)
-  [Function `base_checkpoint`](#0x1_stc_block_base_checkpoint)
-  [Function `latest_state_root`](#0x1_stc_block_latest_state_root)
-  [Function `base_latest_state_root`](#0x1_stc_block_base_latest_state_root)
-  [Function `update_state_root_entry`](#0x1_stc_block_update_state_root_entry)
-  [Function `update_state_root`](#0x1_stc_block_update_state_root)
-  [Function `base_update_state_root`](#0x1_stc_block_base_update_state_root)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `get_current_block_number`](#@Specification_1_get_current_block_number)
    -  [Function `get_parent_hash`](#@Specification_1_get_parent_hash)
    -  [Function `get_current_author`](#@Specification_1_get_current_author)
    -  [Function `process_block_metadata`](#@Specification_1_process_block_metadata)
    -  [Function `checkpoints_init`](#@Specification_1_checkpoints_init)
    -  [Function `checkpoint_entry`](#@Specification_1_checkpoint_entry)
    -  [Function `checkpoint`](#@Specification_1_checkpoint)
    -  [Function `base_checkpoint`](#@Specification_1_base_checkpoint)
    -  [Function `latest_state_root`](#@Specification_1_latest_state_root)
    -  [Function `base_latest_state_root`](#@Specification_1_base_latest_state_root)
    -  [Function `update_state_root_entry`](#@Specification_1_update_state_root_entry)
    -  [Function `update_state_root`](#@Specification_1_update_state_root)
    -  [Function `base_update_state_root`](#@Specification_1_base_update_state_root)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="bcs_util.md#0x1_bcs_util">0x1::bcs_util</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="../../move-stdlib/doc/hash.md#0x1_hash">0x1::hash</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="ring.md#0x1_ring">0x1::ring</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="../../move-stdlib/doc/vector.md#0x1_vector">0x1::vector</a>;
</code></pre>



<a id="0x1_stc_block_BlockMetadata"></a>

## Resource `BlockMetadata`

Block metadata struct.


<pre><code><b>struct</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> <b>has</b> key
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
<code>parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
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
<code>new_block_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">stc_block::NewBlockEvent</a>&gt;</code>
</dt>
<dd>
 Handle of events when new blocks are emitted
</dd>
</dl>


</details>

<a id="0x1_stc_block_NewBlockEvent"></a>

## Struct `NewBlockEvent`

Events emitted when new block generated.


<pre><code><b>struct</b> <a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a> <b>has</b> drop, store
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
<code><a href="timestamp.md#0x1_timestamp">timestamp</a>: u64</code>
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

<a id="0x1_stc_block_Checkpoint"></a>

## Struct `Checkpoint`



<pre><code><b>struct</b> <a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>block_number: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>block_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>state_root: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_block_Checkpoints"></a>

## Resource `Checkpoints`



<pre><code><b>struct</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>checkpoints: <a href="ring.md#0x1_ring_Ring">ring::Ring</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">stc_block::Checkpoint</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>index: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>last_number: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_block_BLOCK_HEADER_LENGTH"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_BLOCK_HEADER_LENGTH">BLOCK_HEADER_LENGTH</a>: u64 = 247;
</code></pre>



<a id="0x1_stc_block_BLOCK_INTERVAL_NUMBER"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_BLOCK_INTERVAL_NUMBER">BLOCK_INTERVAL_NUMBER</a>: u64 = 5;
</code></pre>



<a id="0x1_stc_block_CHECKPOINT_LENGTH"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_CHECKPOINT_LENGTH">CHECKPOINT_LENGTH</a>: u64 = 60;
</code></pre>



<a id="0x1_stc_block_EBLOCK_NUMBER_MISMATCH"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>: u64 = 17;
</code></pre>



<a id="0x1_stc_block_ERROR_INTERVAL_TOO_LITTLE"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_ERROR_INTERVAL_TOO_LITTLE">ERROR_INTERVAL_TOO_LITTLE</a>: u64 = 20;
</code></pre>



<a id="0x1_stc_block_ERROR_NOT_BLOCK_HEADER"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_ERROR_NOT_BLOCK_HEADER">ERROR_NOT_BLOCK_HEADER</a>: u64 = 19;
</code></pre>



<a id="0x1_stc_block_ERROR_NO_HAVE_CHECKPOINT"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_ERROR_NO_HAVE_CHECKPOINT">ERROR_NO_HAVE_CHECKPOINT</a>: u64 = 18;
</code></pre>



<a id="0x1_stc_block_initialize"></a>

## Function `initialize`

This can only be invoked by the GENESIS_ACCOUNT at genesis


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>move_to</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
            number: 0,
            parent_hash,
            author: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
            uncles: 0,
            new_block_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">Self::NewBlockEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
        });
}
</code></pre>



</details>

<a id="0x1_stc_block_get_current_block_number"></a>

## Function `get_current_block_number`

Get the current block number


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">get_current_block_number</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">get_current_block_number</a>(): u64 <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    <b>borrow_global</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).number
}
</code></pre>



</details>

<a id="0x1_stc_block_get_parent_hash"></a>

## Function `get_parent_hash`

Get the hash of the parent block.


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_parent_hash">get_parent_hash</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_parent_hash">get_parent_hash</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    *&<b>borrow_global</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).parent_hash
}
</code></pre>



</details>

<a id="0x1_stc_block_get_current_author"></a>

## Function `get_current_author`

Gets the address of the author of the current block


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_author">get_current_author</a>(): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_author">get_current_author</a>(): <b>address</b> <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    <b>borrow_global</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).author
}
</code></pre>



</details>

<a id="0x1_stc_block_process_block_metadata"></a>

## Function `process_block_metadata`

Call at block prologue


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, author: <b>address</b>, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64, uncles: u64, number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    author: <b>address</b>,
    <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64,
    uncles: u64,
    number: u64
) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> block_metadata_ref = <b>borrow_global_mut</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>assert</b>!(number == (block_metadata_ref.number + 1), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_block.md#0x1_stc_block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>));
    block_metadata_ref.number = number;
    block_metadata_ref.author = author;
    block_metadata_ref.parent_hash = parent_hash;
    block_metadata_ref.uncles = uncles;

    <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a>&gt;(
        &<b>mut</b> block_metadata_ref.new_block_events,
        <a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a> {
            number: number,
            author: author,
            <a href="timestamp.md#0x1_timestamp">timestamp</a>: <a href="timestamp.md#0x1_timestamp">timestamp</a>,
            uncles: uncles,
        }
    );
}
</code></pre>



</details>

<a id="0x1_stc_block_checkpoints_init"></a>

## Function `checkpoints_init`



<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoints_init">checkpoints_init</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoints_init">checkpoints_init</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> checkpoints = <a href="ring.md#0x1_ring_create_with_capacity">ring::create_with_capacity</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(<a href="stc_block.md#0x1_stc_block_CHECKPOINT_LENGTH">CHECKPOINT_LENGTH</a>);
    <b>move_to</b>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>&gt;(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
            checkpoints,
            index: 0,
            last_number: 0,
        });
}
</code></pre>



</details>

<a id="0x1_stc_block_checkpoint_entry"></a>

## Function `checkpoint_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint_entry">checkpoint_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint_entry">checkpoint_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>, <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
    <a href="stc_block.md#0x1_stc_block_checkpoint">checkpoint</a>();
}
</code></pre>



</details>

<a id="0x1_stc_block_checkpoint"></a>

## Function `checkpoint`



<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint">checkpoint</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint">checkpoint</a>() <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>, <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
    <b>let</b> parent_block_number = <a href="stc_block.md#0x1_stc_block_get_current_block_number">get_current_block_number</a>() - 1;
    <b>let</b> parent_block_hash = <a href="stc_block.md#0x1_stc_block_get_parent_hash">get_parent_hash</a>();

    <b>let</b> checkpoints = <b>borrow_global_mut</b>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="stc_block.md#0x1_stc_block_base_checkpoint">base_checkpoint</a>(checkpoints, parent_block_number, parent_block_hash);
}
</code></pre>



</details>

<a id="0x1_stc_block_base_checkpoint"></a>

## Function `base_checkpoint`



<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_checkpoint">base_checkpoint</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>, parent_block_number: u64, parent_block_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_checkpoint">base_checkpoint</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>, parent_block_number: u64, parent_block_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>assert</b>!(
        checkpoints.last_number + <a href="stc_block.md#0x1_stc_block_BLOCK_INTERVAL_NUMBER">BLOCK_INTERVAL_NUMBER</a> &lt;= parent_block_number || checkpoints.last_number == 0,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_block.md#0x1_stc_block_ERROR_INTERVAL_TOO_LITTLE">ERROR_INTERVAL_TOO_LITTLE</a>)
    );

    checkpoints.index = checkpoints.index + 1;
    checkpoints.last_number = parent_block_number;
    <b>let</b> op_checkpoint = <a href="ring.md#0x1_ring_push">ring::push</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(&<b>mut</b> checkpoints.checkpoints, <a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a> {
        block_number: parent_block_number,
        block_hash: parent_block_hash,
        state_root: <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;(),
    });
    <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&op_checkpoint)) {
        <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_some">option::destroy_some</a>(op_checkpoint);
    }<b>else</b> {
        <a href="../../move-stdlib/doc/option.md#0x1_option_destroy_none">option::destroy_none</a>(op_checkpoint);
    }
}
</code></pre>



</details>

<a id="0x1_stc_block_latest_state_root"></a>

## Function `latest_state_root`



<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_latest_state_root">latest_state_root</a>(): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_latest_state_root">latest_state_root</a>(): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
    <b>let</b> checkpoints = <b>borrow_global</b>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="stc_block.md#0x1_stc_block_base_latest_state_root">base_latest_state_root</a>(checkpoints)
}
</code></pre>



</details>

<a id="0x1_stc_block_base_latest_state_root"></a>

## Function `base_latest_state_root`



<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_latest_state_root">base_latest_state_root</a>(checkpoints: &<a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_latest_state_root">base_latest_state_root</a>(checkpoints: &<a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> len = <a href="ring.md#0x1_ring_capacity">ring::capacity</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(&checkpoints.checkpoints);
    <b>let</b> j = <b>if</b> (checkpoints.index &lt; len - 1) {
        checkpoints.index
    }<b>else</b> {
        len
    };
    <b>let</b> i = checkpoints.index;
    <b>while</b> (j &gt; 0) {
        <b>let</b> op_checkpoint = <a href="ring.md#0x1_ring_borrow">ring::borrow</a>(&checkpoints.checkpoints, i - 1);
        <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(op_checkpoint) && <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(op_checkpoint).state_root)) {
            <b>let</b> state_root = <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(&<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(op_checkpoint).state_root);
            <b>return</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(op_checkpoint).block_number, *state_root)
        };
        j = j - 1;
        i = i - 1;
    };

    <b>abort</b> <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="stc_block.md#0x1_stc_block_ERROR_NO_HAVE_CHECKPOINT">ERROR_NO_HAVE_CHECKPOINT</a>)
}
</code></pre>



</details>

<a id="0x1_stc_block_update_state_root_entry"></a>

## Function `update_state_root_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root_entry">update_state_root_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root_entry">update_state_root_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
<b>acquires</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
    <a href="stc_block.md#0x1_stc_block_update_state_root">update_state_root</a>(header);
}
</code></pre>



</details>

<a id="0x1_stc_block_update_state_root"></a>

## Function `update_state_root`



<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root">update_state_root</a>(header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root">update_state_root</a>(header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a> {
    <b>let</b> checkpoints = <b>borrow_global_mut</b>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <a href="stc_block.md#0x1_stc_block_base_update_state_root">base_update_state_root</a>(checkpoints, header);
}
</code></pre>



</details>

<a id="0x1_stc_block_base_update_state_root"></a>

## Function `base_update_state_root`



<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_update_state_root">base_update_state_root</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_update_state_root">base_update_state_root</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">Checkpoints</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <b>let</b> prefix = <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(b"STARCOIN::BlockHeader");

    //parent_hash
    <b>let</b> new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_bytes">bcs_util::skip_bytes</a>(&header, 0);
    //<a href="timestamp.md#0x1_timestamp">timestamp</a>
    <b>let</b> new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_u64">bcs_util::skip_u64</a>(&header, new_offset);
    //number
    <b>let</b> (number, new_offset) = <a href="bcs_util.md#0x1_bcs_util_deserialize_u64">bcs_util::deserialize_u64</a>(&header, new_offset);
    //author
    new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_address">bcs_util::skip_address</a>(&header, new_offset);
    //author_auth_key
    new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_option_bytes">bcs_util::skip_option_bytes</a>(&header, new_offset);
    //txn_accumulator_root
    new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_bytes">bcs_util::skip_bytes</a>(&header, new_offset);
    //block_accumulator_root
    new_offset = <a href="bcs_util.md#0x1_bcs_util_skip_bytes">bcs_util::skip_bytes</a>(&header, new_offset);
    //state_root
    <b>let</b> (state_root, _new_offset) = <a href="bcs_util.md#0x1_bcs_util_deserialize_bytes">bcs_util::deserialize_bytes</a>(&header, new_offset);

    <a href="../../move-stdlib/doc/vector.md#0x1_vector_append">vector::append</a>(&<b>mut</b> prefix, header);
    <b>let</b> block_hash = <a href="../../move-stdlib/doc/hash.md#0x1_hash_sha3_256">hash::sha3_256</a>(prefix);

    <b>let</b> len = <a href="ring.md#0x1_ring_capacity">ring::capacity</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(&checkpoints.checkpoints);
    <b>let</b> j = <b>if</b> (checkpoints.index &lt; len - 1) {
        checkpoints.index
    }<b>else</b> {
        len
    };
    <b>let</b> i = checkpoints.index;
    <b>while</b> (j &gt; 0) {
        <b>let</b> op_checkpoint = <a href="ring.md#0x1_ring_borrow_mut">ring::borrow_mut</a>(&<b>mut</b> checkpoints.checkpoints, i - 1);

        <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(op_checkpoint) && &<a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(
            op_checkpoint
        ).block_hash == &block_hash && <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(op_checkpoint).block_number == number) {
            <b>let</b> op_state_root = &<b>mut</b> <a href="../../move-stdlib/doc/option.md#0x1_option_borrow_mut">option::borrow_mut</a>&lt;<a href="stc_block.md#0x1_stc_block_Checkpoint">Checkpoint</a>&gt;(op_checkpoint).state_root;
            <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(op_state_root)) {
                <a href="../../move-stdlib/doc/option.md#0x1_option_swap">option::swap</a>(op_state_root, state_root);
            }<b>else</b> {
                <a href="../../move-stdlib/doc/option.md#0x1_option_fill">option::fill</a>(op_state_root, state_root);
            };
            <b>return</b>
        };
        j = j - 1;
        i = i - 1;
    };

    <b>abort</b> <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="stc_block.md#0x1_stc_block_ERROR_NO_HAVE_CHECKPOINT">ERROR_NO_HAVE_CHECKPOINT</a>)
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification


<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_1_get_current_block_number"></a>

### Function `get_current_block_number`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_block_number">get_current_block_number</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_get_parent_hash"></a>

### Function `get_parent_hash`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_parent_hash">get_parent_hash</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_get_current_author"></a>

### Function `get_current_author`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_current_author">get_current_author</a>(): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_1_process_block_metadata"></a>

### Function `process_block_metadata`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, author: <b>address</b>, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64, uncles: u64, number: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> number != <b>global</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).number + 1;
</code></pre>




<a id="0x1_stc_block_AbortsIfBlockMetadataNotExist"></a>


<pre><code><b>schema</b> <a href="stc_block.md#0x1_stc_block_AbortsIfBlockMetadataNotExist">AbortsIfBlockMetadataNotExist</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
}
</code></pre>



<a id="@Specification_1_checkpoints_init"></a>

### Function `checkpoints_init`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoints_init">checkpoints_init</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_checkpoint_entry"></a>

### Function `checkpoint_entry`


<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint_entry">checkpoint_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_checkpoint"></a>

### Function `checkpoint`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_checkpoint">checkpoint</a>()
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_base_checkpoint"></a>

### Function `base_checkpoint`


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_checkpoint">base_checkpoint</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>, parent_block_number: u64, parent_block_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_latest_state_root"></a>

### Function `latest_state_root`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_latest_state_root">latest_state_root</a>(): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_base_latest_state_root"></a>

### Function `base_latest_state_root`


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_latest_state_root">base_latest_state_root</a>(checkpoints: &<a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>): (u64, <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_update_state_root_entry"></a>

### Function `update_state_root_entry`


<pre><code><b>public</b> entry <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root_entry">update_state_root_entry</a>(_account: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_update_state_root"></a>

### Function `update_state_root`


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_update_state_root">update_state_root</a>(header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_1_base_update_state_root"></a>

### Function `base_update_state_root`


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_base_update_state_root">base_update_state_root</a>(checkpoints: &<b>mut</b> <a href="stc_block.md#0x1_stc_block_Checkpoints">stc_block::Checkpoints</a>, header: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
