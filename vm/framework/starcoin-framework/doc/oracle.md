
<a id="0x1_oracle"></a>

# Module `0x1::oracle`



-  [Resource `OracleInfo`](#0x1_oracle_OracleInfo)
-  [Struct `DataRecord`](#0x1_oracle_DataRecord)
-  [Resource `OracleFeed`](#0x1_oracle_OracleFeed)
-  [Struct `OracleUpdateEvent`](#0x1_oracle_OracleUpdateEvent)
-  [Resource `DataSource`](#0x1_oracle_DataSource)
-  [Resource `UpdateCapability`](#0x1_oracle_UpdateCapability)
-  [Resource `GenesisSignerCapability`](#0x1_oracle_GenesisSignerCapability)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_oracle_initialize)
-  [Function `extract_signer_cap`](#0x1_oracle_extract_signer_cap)
-  [Function `register_oracle`](#0x1_oracle_register_oracle)
-  [Function `get_oracle_counter`](#0x1_oracle_get_oracle_counter)
-  [Function `get_oracle_info`](#0x1_oracle_get_oracle_info)
-  [Function `init_data_source`](#0x1_oracle_init_data_source)
-  [Function `is_data_source_initialized`](#0x1_oracle_is_data_source_initialized)
-  [Function `update`](#0x1_oracle_update)
-  [Function `update_with_cap`](#0x1_oracle_update_with_cap)
-  [Function `read`](#0x1_oracle_read)
-  [Function `read_record`](#0x1_oracle_read_record)
-  [Function `read_records`](#0x1_oracle_read_records)
-  [Function `remove_update_capability`](#0x1_oracle_remove_update_capability)
-  [Function `add_update_capability`](#0x1_oracle_add_update_capability)
-  [Function `unpack_record`](#0x1_oracle_unpack_record)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer">0x1::reserved_accounts_signer</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_oracle_OracleInfo"></a>

## Resource `OracleInfo`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>counter: u64</code>
</dt>
<dd>
The datasource counter
</dd>
<dt>
<code>info: Info</code>
</dt>
<dd>
Ext info
</dd>
</dl>


</details>

<a id="0x1_oracle_DataRecord"></a>

## Struct `DataRecord`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_DataRecord">DataRecord</a>&lt;ValueT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="version.md#0x1_version">version</a>: u64</code>
</dt>
<dd>
The data version
</dd>
<dt>
<code>value: ValueT</code>
</dt>
<dd>
The record value
</dd>
<dt>
<code>updated_at: u64</code>
</dt>
<dd>
Update timestamp millisecond
</dd>
</dl>


</details>

<a id="0x1_oracle_OracleFeed"></a>

## Resource `OracleFeed`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>record: <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;ValueT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_OracleUpdateEvent"></a>

## Struct `OracleUpdateEvent`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_OracleUpdateEvent">OracleUpdateEvent</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt; <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>source_id: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>record: <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;ValueT&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_DataSource"></a>

## Resource `DataSource`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_DataSource">DataSource</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>id: u64</code>
</dt>
<dd>
 the id of data source of ValueT
</dd>
<dt>
<code>counter: u64</code>
</dt>
<dd>
 the data version counter.
</dd>
<dt>
<code>update_events: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="oracle.md#0x1_oracle_OracleUpdateEvent">oracle::OracleUpdateEvent</a>&lt;OracleT, ValueT&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_UpdateCapability"></a>

## Resource `UpdateCapability`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT: <b>copy</b>, drop, store&gt; <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code><a href="account.md#0x1_account">account</a>: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_oracle_GenesisSignerCapability"></a>

## Resource `GenesisSignerCapability`



<pre><code><b>struct</b> <a href="oracle.md#0x1_oracle_GenesisSignerCapability">GenesisSignerCapability</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_oracle_ERR_CAPABILITY_ACCOUNT_MISS_MATCH"></a>



<pre><code><b>const</b> <a href="oracle.md#0x1_oracle_ERR_CAPABILITY_ACCOUNT_MISS_MATCH">ERR_CAPABILITY_ACCOUNT_MISS_MATCH</a>: u64 = 104;
</code></pre>



<a id="0x1_oracle_ERR_NO_DATA_SOURCE"></a>



<pre><code><b>const</b> <a href="oracle.md#0x1_oracle_ERR_NO_DATA_SOURCE">ERR_NO_DATA_SOURCE</a>: u64 = 103;
</code></pre>



<a id="0x1_oracle_ERR_NO_UPDATE_CAPABILITY"></a>

No capability to update the oracle value.


<pre><code><b>const</b> <a href="oracle.md#0x1_oracle_ERR_NO_UPDATE_CAPABILITY">ERR_NO_UPDATE_CAPABILITY</a>: u64 = 102;
</code></pre>



<a id="0x1_oracle_ERR_ORACLE_TYPE_NOT_REGISTER"></a>

The oracle type not register.


<pre><code><b>const</b> <a href="oracle.md#0x1_oracle_ERR_ORACLE_TYPE_NOT_REGISTER">ERR_ORACLE_TYPE_NOT_REGISTER</a>: u64 = 101;
</code></pre>



<a id="0x1_oracle_initialize"></a>

## Function `initialize`

deprecated.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_initialize">initialize</a>(_sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_initialize">initialize</a>(_sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {}
</code></pre>



</details>

<a id="0x1_oracle_extract_signer_cap"></a>

## Function `extract_signer_cap`

Used in v7->v8 upgrade. struct <code><a href="oracle.md#0x1_oracle_GenesisSignerCapability">GenesisSignerCapability</a></code> is deprecated, in favor of module <code>StarcoinFramework::GenesisSignerCapability</code>.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_extract_signer_cap">extract_signer_cap</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_extract_signer_cap">extract_signer_cap</a>(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="account.md#0x1_account_SignerCapability">account::SignerCapability</a> <b>acquires</b> <a href="oracle.md#0x1_oracle_GenesisSignerCapability">GenesisSignerCapability</a> {
    // CoreAddresses::assert_genesis_address(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>);
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(sender);
    <b>let</b> cap = <b>move_from</b>&lt;<a href="oracle.md#0x1_oracle_GenesisSignerCapability">GenesisSignerCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender));
    <b>let</b> <a href="oracle.md#0x1_oracle_GenesisSignerCapability">GenesisSignerCapability</a> { cap } = cap;
    cap
}
</code></pre>



</details>

<a id="0x1_oracle_register_oracle"></a>

## Function `register_oracle`

Register <code>OracleT</code> as an oracle type.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, info: Info)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>+store+drop, Info: <b>copy</b>+store+drop&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, info: Info) {
    <b>let</b> genesis_account =
        <a href="reserved_accounts_signer.md#0x1_reserved_accounts_signer_get_stored_signer">reserved_accounts_signer::get_stored_signer</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender));
    <b>move_to</b>(&genesis_account, <a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT, Info&gt; {
        counter: 0,
        info,
    });
}
</code></pre>



</details>

<a id="0x1_oracle_get_oracle_counter"></a>

## Function `get_oracle_counter`

Get the <code>OracleT</code> oracle's counter, the counter represent how many <code>OracleT</code> datasources


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_get_oracle_counter">get_oracle_counter</a>&lt;OracleT: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store&gt;(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_get_oracle_counter">get_oracle_counter</a>&lt;OracleT: <b>copy</b> + store + drop, Info: <b>copy</b> + store + drop&gt;(): u64 <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a> {
    <b>let</b> oracle_info = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT, Info&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    oracle_info.counter
}
</code></pre>



</details>

<a id="0x1_oracle_get_oracle_info"></a>

## Function `get_oracle_info`



<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_get_oracle_info">get_oracle_info</a>&lt;OracleT: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store&gt;(): Info
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_get_oracle_info">get_oracle_info</a>&lt;OracleT: <b>copy</b> + store + drop, Info: <b>copy</b> + store + drop&gt;(): Info <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a> {
    <b>let</b> oracle_info = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT, Info&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    *&oracle_info.info
}
</code></pre>



</details>

<a id="0x1_oracle_init_data_source"></a>

## Function `init_data_source`

Init a data source for type <code>OracleT</code>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: ValueT)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>+store+drop, Info: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(
    sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    init_value: ValueT
) <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a> {
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT, Info&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()),
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="oracle.md#0x1_oracle_ERR_ORACLE_TYPE_NOT_REGISTER">ERR_ORACLE_TYPE_NOT_REGISTER</a>)
    );
    <b>let</b> oracle_info = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_OracleInfo">OracleInfo</a>&lt;OracleT, Info&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>let</b> now = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>();
    <b>move_to</b>(sender, <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a>&lt;OracleT, ValueT&gt; {
        record: <a href="oracle.md#0x1_oracle_DataRecord">DataRecord</a>&lt;ValueT&gt; {
            <a href="version.md#0x1_version">version</a>: 0,
            value: init_value,
            updated_at: now,
        }
    });
    <b>let</b> sender_addr = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender);
    <b>move_to</b>(sender, <a href="oracle.md#0x1_oracle_DataSource">DataSource</a>&lt;OracleT, ValueT&gt; {
        id: oracle_info.counter,
        counter: 1,
        update_events: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="oracle.md#0x1_oracle_OracleUpdateEvent">OracleUpdateEvent</a>&lt;OracleT, ValueT&gt;&gt;(sender),
    });
    <b>move_to</b>(sender, <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt; { <a href="account.md#0x1_account">account</a>: sender_addr });
    oracle_info.counter = oracle_info.counter + 1;
}
</code></pre>



</details>

<a id="0x1_oracle_is_data_source_initialized"></a>

## Function `is_data_source_initialized`

Check the DataSource<OracleT,ValueT> is initiailzed at ds_addr


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_is_data_source_initialized">is_data_source_initialized</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(ds_addr: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_is_data_source_initialized">is_data_source_initialized</a>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(ds_addr: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="oracle.md#0x1_oracle_DataSource">DataSource</a>&lt;OracleT, ValueT&gt;&gt;(ds_addr)
}
</code></pre>



</details>

<a id="0x1_oracle_update"></a>

## Function `update`

Update Oracle's record with new value, the <code>sender</code> must have UpdateCapability<OracleT>


<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: ValueT)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(
    sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    value: ValueT
) <b>acquires</b> <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>, <a href="oracle.md#0x1_oracle_DataSource">DataSource</a>, <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a> {
    <b>let</b> <a href="account.md#0x1_account">account</a> = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(<b>exists</b>&lt;<a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;&gt;(<a href="account.md#0x1_account">account</a>), <a href="../../move-stdlib/doc/error.md#0x1_error_resource_exhausted">error::resource_exhausted</a>(<a href="oracle.md#0x1_oracle_ERR_NO_UPDATE_CAPABILITY">ERR_NO_UPDATE_CAPABILITY</a>));
    <b>let</b> cap = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;&gt;(<a href="account.md#0x1_account">account</a>);
    <a href="oracle.md#0x1_oracle_update_with_cap">update_with_cap</a>(cap, value);
}
</code></pre>



</details>

<a id="0x1_oracle_update_with_cap"></a>

## Function `update_with_cap`

Update Oracle's record with new value and UpdateCapability<OracleT>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_update_with_cap">update_with_cap</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(cap: &<b>mut</b> <a href="oracle.md#0x1_oracle_UpdateCapability">oracle::UpdateCapability</a>&lt;OracleT&gt;, value: ValueT)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_update_with_cap">update_with_cap</a>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(
    cap: &<b>mut</b> <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;,
    value: ValueT
) <b>acquires</b> <a href="oracle.md#0x1_oracle_DataSource">DataSource</a>, <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a> {
    <b>let</b> <a href="account.md#0x1_account">account</a> = cap.<a href="account.md#0x1_account">account</a>;
    <b>assert</b>!(<b>exists</b>&lt;<a href="oracle.md#0x1_oracle_DataSource">DataSource</a>&lt;OracleT, ValueT&gt;&gt;(<a href="account.md#0x1_account">account</a>), <a href="../../move-stdlib/doc/error.md#0x1_error_resource_exhausted">error::resource_exhausted</a>(<a href="oracle.md#0x1_oracle_ERR_NO_DATA_SOURCE">ERR_NO_DATA_SOURCE</a>));
    <b>let</b> source = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_DataSource">DataSource</a>&lt;OracleT, ValueT&gt;&gt;(<a href="account.md#0x1_account">account</a>);
    <b>let</b> now = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>();
    <b>let</b> oracle_feed = <b>borrow_global_mut</b>&lt;<a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a>&lt;OracleT, ValueT&gt;&gt;(<a href="account.md#0x1_account">account</a>);
    oracle_feed.record.<a href="version.md#0x1_version">version</a> = source.counter;
    oracle_feed.record.value = value;
    oracle_feed.record.updated_at = now;
    source.counter = source.counter + 1;
    <a href="event.md#0x1_event_emit_event">event::emit_event</a>(&<b>mut</b> source.update_events, <a href="oracle.md#0x1_oracle_OracleUpdateEvent">OracleUpdateEvent</a>&lt;OracleT, ValueT&gt; {
        source_id: source.id,
        record: *&oracle_feed.record
    });
}
</code></pre>



</details>

<a id="0x1_oracle_read"></a>

## Function `read`

Read the Oracle's value from <code>ds_addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read">read</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(ds_addr: <b>address</b>): ValueT
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read">read</a>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(ds_addr: <b>address</b>): ValueT <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a> {
    <b>let</b> oracle_feed = <b>borrow_global</b>&lt;<a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a>&lt;OracleT, ValueT&gt;&gt;(ds_addr);
    *&oracle_feed.record.value
}
</code></pre>



</details>

<a id="0x1_oracle_read_record"></a>

## Function `read_record`

Read the Oracle's DataRecord from <code>ds_addr</code>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read_record">read_record</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(ds_addr: <b>address</b>): <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;ValueT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read_record">read_record</a>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(
    ds_addr: <b>address</b>
): <a href="oracle.md#0x1_oracle_DataRecord">DataRecord</a>&lt;ValueT&gt; <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a> {
    <b>let</b> oracle_feed = <b>borrow_global</b>&lt;<a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a>&lt;OracleT, ValueT&gt;&gt;(ds_addr);
    *&oracle_feed.record
}
</code></pre>



</details>

<a id="0x1_oracle_read_records"></a>

## Function `read_records`

Batch read Oracle's DataRecord from <code>ds_addrs</code>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read_records">read_records</a>&lt;OracleT: <b>copy</b>, drop, store, ValueT: <b>copy</b>, drop, store&gt;(ds_addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;ValueT&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_read_records">read_records</a>&lt;OracleT: <b>copy</b>+store+drop, ValueT: <b>copy</b>+store+drop&gt;(
    ds_addrs: &<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;
): <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="oracle.md#0x1_oracle_DataRecord">DataRecord</a>&lt;ValueT&gt;&gt; <b>acquires</b> <a href="oracle.md#0x1_oracle_OracleFeed">OracleFeed</a> {
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(ds_addrs);
    <b>let</b> results = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>();
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> addr = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(ds_addrs, i);
        <b>let</b> record = <a href="oracle.md#0x1_oracle_read_record">Self::read_record</a>&lt;OracleT, ValueT&gt;(addr);
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> results, record);
        i = i + 1;
    };
    results
}
</code></pre>



</details>

<a id="0x1_oracle_remove_update_capability"></a>

## Function `remove_update_capability`

Remove UpdateCapability from current sender.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_remove_update_capability">remove_update_capability</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="oracle.md#0x1_oracle_UpdateCapability">oracle::UpdateCapability</a>&lt;OracleT&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_remove_update_capability">remove_update_capability</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(
    sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
): <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt; <b>acquires</b> <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a> {
    <b>let</b> <a href="account.md#0x1_account">account</a> = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender);
    <b>assert</b>!(<b>exists</b>&lt;<a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;&gt;(<a href="account.md#0x1_account">account</a>), <a href="../../move-stdlib/doc/error.md#0x1_error_resource_exhausted">error::resource_exhausted</a>(<a href="oracle.md#0x1_oracle_ERR_NO_UPDATE_CAPABILITY">ERR_NO_UPDATE_CAPABILITY</a>));
    <b>move_from</b>&lt;<a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;&gt;(<a href="account.md#0x1_account">account</a>)
}
</code></pre>



</details>

<a id="0x1_oracle_add_update_capability"></a>

## Function `add_update_capability`

Add UpdateCapability to current sender


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_add_update_capability">add_update_capability</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, update_cap: <a href="oracle.md#0x1_oracle_UpdateCapability">oracle::UpdateCapability</a>&lt;OracleT&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_add_update_capability">add_update_capability</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, update_cap: <a href="oracle.md#0x1_oracle_UpdateCapability">UpdateCapability</a>&lt;OracleT&gt;) {
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(sender) == update_cap.<a href="account.md#0x1_account">account</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="oracle.md#0x1_oracle_ERR_CAPABILITY_ACCOUNT_MISS_MATCH">ERR_CAPABILITY_ACCOUNT_MISS_MATCH</a>)
    );
    <b>move_to</b>(sender, update_cap);
}
</code></pre>



</details>

<a id="0x1_oracle_unpack_record"></a>

## Function `unpack_record`

Unpack Record to fields: version, oracle, updated_at.


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_unpack_record">unpack_record</a>&lt;ValueT: <b>copy</b>, drop, store&gt;(record: <a href="oracle.md#0x1_oracle_DataRecord">oracle::DataRecord</a>&lt;ValueT&gt;): (u64, ValueT, u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="oracle.md#0x1_oracle_unpack_record">unpack_record</a>&lt;ValueT: <b>copy</b>+store+drop&gt;(record: <a href="oracle.md#0x1_oracle_DataRecord">DataRecord</a>&lt;ValueT&gt;): (u64, ValueT, u64) {
    (record.<a href="version.md#0x1_version">version</a>, *&record.value, record.updated_at)
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
