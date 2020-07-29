
<a name="0x1_Timestamp"></a>

# Module `0x1::Timestamp`

### Table of Contents

-  [Resource `CurrentTimeSeconds`](#0x1_Timestamp_CurrentTimeSeconds)
-  [Function `initialize`](#0x1_Timestamp_initialize)
-  [Function `update_global_time`](#0x1_Timestamp_update_global_time)
-  [Function `now_seconds`](#0x1_Timestamp_now_seconds)
-  [Function `is_genesis`](#0x1_Timestamp_is_genesis)
-  [Specification](#0x1_Timestamp_Specification)
    -  [Function `initialize`](#0x1_Timestamp_Specification_initialize)
    -  [Function `update_global_time`](#0x1_Timestamp_Specification_update_global_time)
    -  [Function `now_seconds`](#0x1_Timestamp_Specification_now_seconds)
    -  [Function `is_genesis`](#0x1_Timestamp_Specification_is_genesis)



<a name="0x1_Timestamp_CurrentTimeSeconds"></a>

## Resource `CurrentTimeSeconds`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>
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

<a name="0x1_Timestamp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_initialize">initialize</a>(account: &signer) {
    // Only callable by the <a href="Genesis.md#0x1_Genesis">Genesis</a> address
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    // TODO: Pass the initialized value be passed in <b>to</b> genesis?
    <b>let</b> timer = <a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {seconds: 0};
    move_to&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(account, timer);
}
</code></pre>



</details>

<a name="0x1_Timestamp_update_global_time"></a>

## Function `update_global_time`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64) <b>acquires</b> <a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> global_timer = borrow_global_mut&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>());
    //TODO should support '=' ?
    <b>assert</b>(global_timer.seconds &lt;= timestamp, 5001);
    global_timer.seconds = timestamp;
}
</code></pre>



</details>

<a name="0x1_Timestamp_now_seconds"></a>

## Function `now_seconds`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_now_seconds">now_seconds</a>(): u64 <b>acquires</b> <a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a> {
    borrow_global&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>()).seconds
}
</code></pre>



</details>

<a name="0x1_Timestamp_is_genesis"></a>

## Function `is_genesis`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_is_genesis">is_genesis</a>(): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_is_genesis">is_genesis</a>(): bool {
    !exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>())
}
</code></pre>



</details>

<a name="0x1_Timestamp_Specification"></a>

## Specification


<a name="0x1_Timestamp_Specification_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>();
<b>aborts_if</b> exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account)).seconds == 0;
</code></pre>



<a name="0x1_Timestamp_Specification_update_global_time"></a>

### Function `update_global_time`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_update_global_time">update_global_time</a>(account: &signer, timestamp: u64)
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>());
<b>ensures</b> <b>global</b>&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>()).seconds == timestamp;
</code></pre>



<a name="0x1_Timestamp_Specification_now_seconds"></a>

### Function `now_seconds`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_now_seconds">now_seconds</a>(): u64
</code></pre>




<pre><code><b>aborts_if</b> !exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>());
<b>ensures</b> result == <b>global</b>&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>()).seconds;
</code></pre>



<a name="0x1_Timestamp_Specification_is_genesis"></a>

### Function `is_genesis`


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_Timestamp_is_genesis">is_genesis</a>(): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
<b>ensures</b> result == !exists&lt;<a href="#0x1_Timestamp_CurrentTimeSeconds">CurrentTimeSeconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ACCOUNT">CoreAddresses::SPEC_GENESIS_ACCOUNT</a>());
</code></pre>
