
<a id="0x1_stc_offer"></a>

# Module `0x1::stc_offer`



-  [Resource `Offer`](#0x1_stc_offer_Offer)
-  [Constants](#@Constants_0)
-  [Function `create`](#0x1_stc_offer_create)
-  [Function `redeem`](#0x1_stc_offer_redeem)
-  [Function `exists_at`](#0x1_stc_offer_exists_at)
-  [Function `address_of`](#0x1_stc_offer_address_of)
-  [Specification](#@Specification_1)
    -  [Function `create`](#@Specification_1_create)
    -  [Function `redeem`](#@Specification_1_redeem)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `address_of`](#@Specification_1_address_of)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_stc_offer_Offer"></a>

## Resource `Offer`

A wrapper around value <code>offered</code> that can be claimed by the address stored in <code>for</code> when after lock time.


<pre><code><b>struct</b> <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>offered: Offered</code>
</dt>
<dd>

</dd>
<dt>
<code>for_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>time_lock: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_offer_EOFFER_DNE_FOR_ACCOUNT"></a>

An offer of the specified type for the account does not match


<pre><code><b>const</b> <a href="stc_offer.md#0x1_stc_offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>: u64 = 101;
</code></pre>



<a id="0x1_stc_offer_EOFFER_NOT_UNLOCKED"></a>

Offer is not unlocked yet.


<pre><code><b>const</b> <a href="stc_offer.md#0x1_stc_offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>: u64 = 102;
</code></pre>



<a id="0x1_stc_offer_create"></a>

## Function `create`

Publish a value of type <code>Offered</code> under the sender's account. The value can be claimed by
either the <code>for</code> address or the transaction sender.


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_create">create</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offered: Offered, for_address: <b>address</b>, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_create">create</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offered: Offered, for_address: <b>address</b>, lock_period: u64) {
    <b>let</b> time_lock = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() + lock_period;
    //TODO should support multi <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>?
    <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt; {
        offered,
        for_address,
        time_lock
    });
}
</code></pre>



</details>

<a id="0x1_stc_offer_redeem"></a>

## Function `redeem`

Claim the value of type <code>Offered</code> published at <code>offer_address</code>.
Only succeeds if the sender is the intended recipient stored in <code>for</code> or the original
publisher <code>offer_address</code>, and now >= time_lock
Also fails if no such value exists.


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_redeem">redeem</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offer_address: <b>address</b>): Offered
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_redeem">redeem</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offer_address: <b>address</b>): Offered <b>acquires</b> <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a> {
    <b>let</b> <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt; { offered, for_address, time_lock } = <b>move_from</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
    <b>let</b> sender = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> now = <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>();
    <b>assert</b>!(sender == for_address || sender == offer_address, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_offer.md#0x1_stc_offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>));
    <b>assert</b>!(now &gt;= time_lock, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="stc_offer.md#0x1_stc_offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>));
    offered
}
</code></pre>



</details>

<a id="0x1_stc_offer_exists_at"></a>

## Function `exists_at`

Returns true if an offer of type <code>Offered</code> exists at <code>offer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)
}
</code></pre>



</details>

<a id="0x1_stc_offer_address_of"></a>

## Function `address_of`

Returns the address of the <code>Offered</code> type stored at <code>offer_address</code>.
Fails if no such <code><a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a></code> exists.


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b> <b>acquires</b> <a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a> {
    <b>borrow_global</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for_address
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_1_create"></a>

### Function `create`


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_create">create</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offered: Offered, for_address: <b>address</b>, lock_period: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() + lock_period &gt; max_u64();
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_1_redeem"></a>

### Function `redeem`


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_redeem">redeem</a>&lt;Offered: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, offer_address: <b>address</b>): Offered
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
<b>aborts_if</b>
    <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <b>global</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for_address
        && <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != offer_address;
<b>aborts_if</b> <a href="timestamp.md#0x1_timestamp_now_seconds">timestamp::now_seconds</a>() &lt; <b>global</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).time_lock;
</code></pre>



<a id="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_address_of"></a>

### Function `address_of`


<pre><code><b>public</b> <b>fun</b> <a href="stc_offer.md#0x1_stc_offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_offer.md#0x1_stc_offer_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
