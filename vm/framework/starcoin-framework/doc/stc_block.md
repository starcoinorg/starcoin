
<a id="0x1_stc_block"></a>

# Module `0x1::stc_block`

Block module provide metadata for generated blocks.


-  [Resource `BlockMetadata`](#0x1_stc_block_BlockMetadata)
-  [Struct `NewBlockEvent`](#0x1_stc_block_NewBlockEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_stc_block_initialize)
-  [Function `get_current_block_number`](#0x1_stc_block_get_current_block_number)
-  [Function `get_parent_hash`](#0x1_stc_block_get_parent_hash)
-  [Function `get_parents_hash`](#0x1_stc_block_get_parents_hash)
-  [Function `get_current_author`](#0x1_stc_block_get_current_author)
-  [Function `block_prologue`](#0x1_stc_block_block_prologue)
-  [Function `process_block_metadata`](#0x1_stc_block_process_block_metadata)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `get_current_block_number`](#@Specification_1_get_current_block_number)
    -  [Function `get_parent_hash`](#@Specification_1_get_parent_hash)
    -  [Function `get_current_author`](#@Specification_1_get_current_author)
    -  [Function `process_block_metadata`](#@Specification_1_process_block_metadata)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="block_reward.md#0x1_block_reward">0x1::block_reward</a>;
<b>use</b> <a href="chain_id.md#0x1_chain_id">0x1::chain_id</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="epoch.md#0x1_epoch">0x1::epoch</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee">0x1::stc_transaction_fee</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
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
<code>parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>
 An Array of the parents hash for a Dag block.
</dd>
<dt>
<code>new_block_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">stc_block::NewBlockEvent</a>&gt;</code>
</dt>
<dd>

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
<dt>
<code>parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
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



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>: u64 = 1017;
</code></pre>



<a id="0x1_stc_block_EPROLOGUE_BAD_CHAIN_ID"></a>



<pre><code><b>const</b> <a href="stc_block.md#0x1_stc_block_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>: u64 = 1006;
</code></pre>



<a id="0x1_stc_block_initialize"></a>

## Function `initialize`

This can only be invoked by the GENESIS_ACCOUNT at genesis


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_initialize">stc_block::initialize</a> | entered "));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> block_metadata = <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
        number: 0,
        parent_hash,
        author: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
        uncles: 0,
        parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;(),
        new_block_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a>&gt;(<a href="account.md#0x1_account">account</a>),
    };

    <b>move_to</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="account.md#0x1_account">account</a>, block_metadata);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_initialize">stc_block::initialize</a> | exited "));
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

<a id="0x1_stc_block_get_parents_hash"></a>

## Function `get_parents_hash`

Get the hash of the parents block, used for DAG


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_parents_hash">get_parents_hash</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_get_parents_hash">get_parents_hash</a>(): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt; <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    *&<b>borrow_global</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()).parents_hash
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

<a id="0x1_stc_block_block_prologue"></a>

## Function `block_prologue`

Set the metadata for the current block and distribute transaction fees and block rewards.
The runtime always runs this before executing the transactions in a block.


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_block_prologue">block_prologue</a>(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64, author: <b>address</b>, auth_key_vec: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, uncles: u64, number: u64, <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8, parent_gas_used: u64, parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_block.md#0x1_stc_block_block_prologue">block_prologue</a>(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64,
    author: <b>address</b>,
    auth_key_vec: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    uncles: u64,
    number: u64,
    <a href="chain_id.md#0x1_chain_id">chain_id</a>: u8,
    parent_gas_used: u64,
    parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_block_prologue">stc_block::block_prologue</a> | Entered"));

    // Can only be invoked by genesis <a href="account.md#0x1_account">account</a>
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(&<a href="account.md#0x1_account">account</a>);

    // Check that the chain ID stored on-chain matches the chain ID
    // specified by the transaction
    <b>assert</b>!(
        <a href="chain_id.md#0x1_chain_id_get">chain_id::get</a>() == <a href="chain_id.md#0x1_chain_id">chain_id</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_block.md#0x1_stc_block_EPROLOGUE_BAD_CHAIN_ID">EPROLOGUE_BAD_CHAIN_ID</a>)
    );

    // deal <b>with</b> previous block first.
    <b>let</b> txn_fee = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">stc_transaction_fee::distribute_transaction_fees</a>&lt;STC&gt;(&<a href="account.md#0x1_account">account</a>);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_block_prologue">stc_block::block_prologue</a> | txn_fee"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="coin.md#0x1_coin_value">coin::value</a>(&txn_fee));

    // then deal <b>with</b> current block.
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_block_prologue">stc_block::block_prologue</a> | <a href="timestamp.md#0x1_timestamp_update_global_time">timestamp::update_global_time</a>"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="timestamp.md#0x1_timestamp">timestamp</a>);
    <a href="timestamp.md#0x1_timestamp_update_global_time">timestamp::update_global_time</a>(&<a href="account.md#0x1_account">account</a>, <a href="timestamp.md#0x1_timestamp">timestamp</a> * 1000);

    <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(
        &<a href="account.md#0x1_account">account</a>,
        parent_hash,
        author,
        <a href="timestamp.md#0x1_timestamp">timestamp</a>,
        uncles,
        number,
        parents_hash,
    );

    <b>let</b> reward = <a href="epoch.md#0x1_epoch_adjust_epoch">epoch::adjust_epoch</a>(&<a href="account.md#0x1_account">account</a>, number, <a href="timestamp.md#0x1_timestamp">timestamp</a>, uncles, parent_gas_used);

    // pass in previous block gas fees.
    <a href="block_reward.md#0x1_block_reward_process_block_reward">block_reward::process_block_reward</a>(&<a href="account.md#0x1_account">account</a>, number, reward, author, auth_key_vec, txn_fee);

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_block_prologue">stc_block::block_prologue</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_stc_block_process_block_metadata"></a>

## Function `process_block_metadata`

Call at block prologue


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, author: <b>address</b>, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64, uncles: u64, number: u64, parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    author: <b>address</b>,
    <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64,
    uncles: u64,
    number: u64,
    parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
) <b>acquires</b> <a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_process_block_metadata">stc_block::process_block_metadata</a> | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    <b>let</b> block_metadata_ref = <b>borrow_global_mut</b>&lt;<a href="stc_block.md#0x1_stc_block_BlockMetadata">BlockMetadata</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_process_block_metadata">stc_block::process_block_metadata</a> | <b>to</b> check block number"));

    <b>assert</b>!(number == (block_metadata_ref.number + 1), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_block.md#0x1_stc_block_EBLOCK_NUMBER_MISMATCH">EBLOCK_NUMBER_MISMATCH</a>));
    block_metadata_ref.number = number;
    block_metadata_ref.author = author;
    block_metadata_ref.parent_hash = parent_hash;
    block_metadata_ref.uncles = uncles;
    block_metadata_ref.parents_hash = parents_hash;

    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_process_block_metadata">stc_block::process_block_metadata</a> | <b>to</b> emit <a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a>  "));

    <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a>&gt;(
        &<b>mut</b> block_metadata_ref.new_block_events,
        <a href="stc_block.md#0x1_stc_block_NewBlockEvent">NewBlockEvent</a> {
            number,
            author,
            <a href="timestamp.md#0x1_timestamp">timestamp</a>,
            parents_hash,
            uncles,
        }
    );
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="stc_block.md#0x1_stc_block_process_block_metadata">stc_block::process_block_metadata</a> | Exited"));
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


<pre><code><b>fun</b> <a href="stc_block.md#0x1_stc_block_process_block_metadata">process_block_metadata</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, parent_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, author: <b>address</b>, <a href="timestamp.md#0x1_timestamp">timestamp</a>: u64, uncles: u64, number: u64, parents_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
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


[move-book]: https://starcoin.dev/move/book/SUMMARY
