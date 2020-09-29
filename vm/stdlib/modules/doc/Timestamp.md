
<a name="0x1_Timestamp"></a>

# Module `0x1::Timestamp`



-  [Resource <code><a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a></code>](#0x1_Timestamp_CurrentTimeSeconds)
-  [Resource <code><a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a></code>](#0x1_Timestamp_TimeHasStarted)
-  [Function <code>initialize</code>](#0x1_Timestamp_initialize)
-  [Function <code>update_global_time</code>](#0x1_Timestamp_update_global_time)
-  [Function <code>now_seconds</code>](#0x1_Timestamp_now_seconds)
-  [Function <code>set_time_has_started</code>](#0x1_Timestamp_set_time_has_started)
-  [Function <code>is_genesis</code>](#0x1_Timestamp_is_genesis)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>update_global_time</code>](#@Specification_0_update_global_time)
    -  [Function <code>now_seconds</code>](#@Specification_0_now_seconds)
    -  [Function <code>set_time_has_started</code>](#@Specification_0_set_time_has_started)
    -  [Function <code>is_genesis</code>](#@Specification_0_is_genesis)


<a name="0x1_Timestamp_CurrentTimeSeconds"></a>

## Resource `CurrentTimeSeconds`



<pre><code><b>resource</b> <b>struct</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>seconds: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_Timestamp_TimeHasStarted"></a>

## Resource `TimeHasStarted`

A singleton resource used to determine whether time has started. This
is called at the end of genesis.


<pre><code><b>resource</b> <b>struct</b> <a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>
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

<a name="0x1_Timestamp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64) {
    // Only callable by the Genesis address
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> timer = <a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {seconds: genesis_timestamp};
    move_to&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(account, timer);
}
</code></pre>



</details>

<a name="0x1_Timestamp_update_global_time"></a>

## Function `update_global_time`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64) <b>acquires</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    //Do not <b>update</b> time before time start.
    <b>let</b> global_timer = borrow_global_mut&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    //TODO remove = after <b>use</b> milli seconds
    <b>assert</b>(timestamp &gt;= global_timer.seconds, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_TIMESTAMP">ErrorCode::EINVALID_TIMESTAMP</a>());
    global_timer.seconds = timestamp;
}
</code></pre>



</details>

<a name="0x1_Timestamp_now_seconds"></a>

## Function `now_seconds`



<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64 <b>acquires</b> <a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {
    borrow_global&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).seconds
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
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    // Current time must have been initialized.
    <b>assert</b>(
        <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()),
        1
    );
    move_to(account, <a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>{});
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

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_initialize">initialize</a>(account: &signer, genesis_timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> timestamp &lt; <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).seconds;
<b>ensures</b> <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).seconds == timestamp;
</code></pre>



<a name="@Specification_0_now_seconds"></a>

### Function `now_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>ensures</b> result == <b>global</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).seconds;
</code></pre>



<a name="@Specification_0_set_time_has_started"></a>

### Function `set_time_has_started`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_set_time_has_started">set_time_has_started</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_is_genesis"></a>

### Function `is_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="Timestamp.md#0x1_Timestamp_is_genesis">is_genesis</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_TimeHasStarted">TimeHasStarted</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
</code></pre>
