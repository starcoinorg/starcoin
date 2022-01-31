
<a name="0x1_Timestamp"></a>

# Module `0x1::Timestamp`

The module implements onchain timestamp oracle.
Timestamp is updated on each block. It always steps forward, and never come backward.


-  [Resource `CurrentTimeMilliseconds`](#0x1_Timestamp_CurrentTimeMilliseconds)
-  [Resource `TimeHasStarted`](#0x1_Timestamp_TimeHasStarted)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_Timestamp_initialize)
-  [Function `update_global_time`](#0x1_Timestamp_update_global_time)
-  [Function `now_seconds`](#0x1_Timestamp_now_seconds)
-  [Function `now_milliseconds`](#0x1_Timestamp_now_milliseconds)
-  [Function `set_time_has_started`](#0x1_Timestamp_set_time_has_started)
-  [Function `is_genesis`](#0x1_Timestamp_is_genesis)
-  [Function `assert_genesis`](#0x1_Timestamp_assert_genesis)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `update_global_time`](#@Specification_1_update_global_time)
    -  [Function `now_seconds`](#@Specification_1_now_seconds)
    -  [Function `now_milliseconds`](#@Specification_1_now_milliseconds)
    -  [Function `set_time_has_started`](#@Specification_1_set_time_has_started)
    -  [Function `is_genesis`](#@Specification_1_is_genesis)
    -  [Function `assert_genesis`](#@Specification_1_assert_genesis)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
</code></pre>



<a name="0x1_Timestamp_CurrentTimeMilliseconds"></a>

## Resource `CurrentTimeMilliseconds`



<pre><code><b>struct</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>milliseconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Timestamp_TimeHasStarted"></a>

## Resource `TimeHasStarted`

A singleton resource used to determine whether time has started. This
is called at the end of genesis.


<pre><code><b>struct</b> <a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>dummy_field: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Timestamp_EINVALID_TIMESTAMP"></a>



<pre><code><b>const</b> <a href="Timestamp.md#0x1_Timestamp_EINVALID_TIMESTAMP">EINVALID_TIMESTAMP</a>: u64 = 14;
</code></pre>



<a name="0x1_Timestamp_ENOT_GENESIS"></a>



<pre><code><b>const</b> <a href="Timestamp.md#0x1_Timestamp_ENOT_GENESIS">ENOT_GENESIS</a>: u64 = 12;
</code></pre>



<a name="0x1_Timestamp_ENOT_INITIALIZED"></a>



<pre><code><b>const</b> <a href="Timestamp.md#0x1_Timestamp_ENOT_INITIALIZED">ENOT_INITIALIZED</a>: u64 = 101;
</code></pre>



<a name="0x1_Timestamp_MILLI_CONVERSION_FACTOR"></a>

Conversion factor between seconds and milliseconds


<pre><code><b>const</b> <a href="Timestamp.md#0x1_Timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>: u64 = 1000;
</code></pre>



<a name="0x1_Timestamp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64) {
    // Only callable by the <a href="Genesis.md#0x1_Genesis">Genesis</a> <b>address</b>
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>let</b> milli_timer = <a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a> {milliseconds: genesis_timestamp};
    <b>move_to</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(account, milli_timer);
}
</code></pre>



</details>

<a name="0x1_Timestamp_update_global_time"></a>

## Function `update_global_time`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64) <b>acquires</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a> {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    //Do not <b>update</b> time before time start.
    <b>let</b> global_milli_timer = <b>borrow_global_mut</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>assert</b>!(timestamp &gt; global_milli_timer.milliseconds, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Timestamp.md#0x1_Timestamp_EINVALID_TIMESTAMP">EINVALID_TIMESTAMP</a>));
    global_milli_timer.milliseconds = timestamp;
}
</code></pre>



</details>

<a name="0x1_Timestamp_now_seconds"></a>

## Function `now_seconds`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64 <b>acquires</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a> {
    <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">now_milliseconds</a>() / <a href="Timestamp.md#0x1_Timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>
}
</code></pre>



</details>

<a name="0x1_Timestamp_now_milliseconds"></a>

## Function `now_milliseconds`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">now_milliseconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">now_milliseconds</a>(): u64 <b>acquires</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a> {
    <b>borrow_global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).milliseconds
}
</code></pre>



</details>

<a name="0x1_Timestamp_set_time_has_started"></a>

## Function `set_time_has_started`

Marks that time has started and genesis has finished. This can only be called from genesis.


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">set_time_has_started</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">set_time_has_started</a>(account: &signer) {
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);

    // Current time must have been initialized.
    <b>assert</b>!(
        <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()),
        <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Timestamp.md#0x1_Timestamp_ENOT_INITIALIZED">ENOT_INITIALIZED</a>)
    );
    <b>move_to</b>(account, <a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>{});
}
</code></pre>



</details>

<a name="0x1_Timestamp_is_genesis"></a>

## Function `is_genesis`

Helper function to determine if the blockchain is in genesis state.


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>(): bool {
    !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>())
}
</code></pre>



</details>

<a name="0x1_Timestamp_assert_genesis"></a>

## Function `assert_genesis`

Helper function to assert genesis state.


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_assert_genesis">assert_genesis</a>()
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_assert_genesis">assert_genesis</a>() {
    <b>assert</b>!(<a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>(), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="Timestamp.md#0x1_Timestamp_ENOT_GENESIS">ENOT_GENESIS</a>));
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> timestamp &lt;= <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).milliseconds;
<b>ensures</b> <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).milliseconds == timestamp;
</code></pre>



<a name="@Specification_1_now_seconds"></a>

### Function `now_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> result == <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">now_milliseconds</a>() / <a href="Timestamp.md#0x1_Timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>;
</code></pre>




<a name="0x1_Timestamp_spec_now_seconds"></a>


<pre><code><b>fun</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_seconds">spec_now_seconds</a>(): u64 {
   <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).milliseconds / <a href="Timestamp.md#0x1_Timestamp_MILLI_CONVERSION_FACTOR">MILLI_CONVERSION_FACTOR</a>
}
</code></pre>



<a name="@Specification_1_now_milliseconds"></a>

### Function `now_milliseconds`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">now_milliseconds</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> result == <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).milliseconds;
</code></pre>




<a name="0x1_Timestamp_spec_now_millseconds"></a>


<pre><code><b>fun</b> <a href="Timestamp.md#0x1_Timestamp_spec_now_millseconds">spec_now_millseconds</a>(): u64 {
   <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).milliseconds
}
</code></pre>



<a name="@Specification_1_set_time_has_started"></a>

### Function `set_time_has_started`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">set_time_has_started</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_is_genesis"></a>

### Function `is_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>



<a name="@Specification_1_assert_genesis"></a>

### Function `assert_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_assert_genesis">assert_genesis</a>()
</code></pre>




<pre><code><b>pragma</b> opaque = <b>true</b>;
<b>include</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a>;
</code></pre>


Helper schema to specify that a function aborts if not in genesis.


<a name="0x1_Timestamp_AbortsIfNotGenesis"></a>


<pre><code><b>schema</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfNotGenesis">AbortsIfNotGenesis</a> {
    <b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>();
}
</code></pre>




<a name="0x1_Timestamp_AbortsIfTimestampNotExists"></a>


<pre><code><b>schema</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfTimestampNotExists">AbortsIfTimestampNotExists</a> {
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
}
</code></pre>
