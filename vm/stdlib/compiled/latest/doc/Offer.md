
<a name="0x1_Offer"></a>

# Module `0x1::Offer`



-  [Resource `Offer`](#0x1_Offer_Offer)
-  [Constants](#@Constants_0)
-  [Function `create`](#0x1_Offer_create)
-  [Function `redeem`](#0x1_Offer_redeem)
-  [Function `exists_at`](#0x1_Offer_exists_at)
-  [Function `address_of`](#0x1_Offer_address_of)
-  [Function `take_offer`](#0x1_Offer_take_offer)
-  [Specification](#@Specification_1)
    -  [Function `create`](#@Specification_1_create)
    -  [Function `redeem`](#@Specification_1_redeem)
    -  [Function `exists_at`](#@Specification_1_exists_at)
    -  [Function `address_of`](#@Specification_1_address_of)
    -  [Function `take_offer`](#@Specification_1_take_offer)


<pre><code><b>use</b> <a href="Collection2.md#0x1_Collection2">0x1::Collection2</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_Offer_Offer"></a>

## Resource `Offer`

A wrapper around value <code>offered</code> that can be claimed by the address stored in <code>for</code> when after lock time.


<pre><code><b>struct</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; <b>has</b> key
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
<code>for: <b>address</b></code>
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_Offer_EOFFER_DNE_FOR_ACCOUNT"></a>

An offer of the specified type for the account does not match


<pre><code><b>const</b> <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>: u64 = 101;
</code></pre>



<a name="0x1_Offer_EOFFER_NOT_UNLOCKED"></a>

Offer is not unlocked yet.


<pre><code><b>const</b> <a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>: u64 = 102;
</code></pre>



<a name="0x1_Offer_create"></a>

## Function `create`

Publish a value of type <code>Offered</code> under the sender's account. The value can be claimed by
either the <code>for</code> address or the transaction sender.


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered: store&gt;(account: &signer, offered: Offered, for: <b>address</b>, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered: store&gt;(account: &signer, offered: Offered, for: <b>address</b>, lock_period: u64) {
    <b>let</b> time_lock = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() + lock_period;
    //TODO should support multi <a href="Offer.md#0x1_Offer">Offer</a>?
    <b>move_to</b>(account, <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for, time_lock });
}
</code></pre>



</details>

<a name="0x1_Offer_redeem"></a>

## Function `redeem`

Claim the value of type <code>Offered</code> published at <code>offer_address</code>.
Only succeeds if the sender is the intended recipient stored in <code>for</code> or the original
publisher <code>offer_address</code>, and now >= time_lock
Also fails if no such value exists.


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered: store&gt;(account: &signer, offer_address: <b>address</b>): Offered
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered: store&gt;(account: &signer, offer_address: <b>address</b>): Offered <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
    <b>let</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for, time_lock } = <b>move_from</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>assert</b>!(sender == for || sender == offer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>));
    <b>assert</b>!(now &gt;= time_lock, <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>));
    offered
}
</code></pre>



</details>

<a name="0x1_Offer_exists_at"></a>

## Function `exists_at`

Returns true if an offer of type <code>Offered</code> exists at <code>offer_address</code>.


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool {
    <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)
}
</code></pre>



</details>

<a name="0x1_Offer_address_of"></a>

## Function `address_of`

Returns the address of the <code>Offered</code> type stored at <code>offer_address</code>.
Fails if no such <code><a href="Offer.md#0x1_Offer">Offer</a></code> exists.


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b> <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
    <b>borrow_global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for
}
</code></pre>



</details>

<a name="0x1_Offer_take_offer"></a>

## Function `take_offer`

Take Offer and put to signer's Collection<Offered>.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Offer.md#0x1_Offer_take_offer">take_offer</a>&lt;Offered: store&gt;(signer: signer, offer_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Offer.md#0x1_Offer_take_offer">take_offer</a>&lt;Offered: store&gt;(
    signer: signer,
    offer_address: <b>address</b>,
) <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
    <b>let</b> offered = <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered&gt;(&signer, offer_address);
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(&signer, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&signer), offered);
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_create"></a>

### Function `create`


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered: store&gt;(account: &signer, offered: Offered, for: <b>address</b>, lock_period: u64)
</code></pre>




<pre><code><b>include</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfTimestampNotExists">Timestamp::AbortsIfTimestampNotExists</a>;
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() + lock_period &gt; max_u64();
<b>aborts_if</b> <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_redeem"></a>

### Function `redeem`


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered: store&gt;(account: &signer, offer_address: <b>address</b>): Offered
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for && <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != offer_address;
<b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() &lt; <b>global</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).time_lock;
<b>include</b> <a href="Timestamp.md#0x1_Timestamp_AbortsIfTimestampNotExists">Timestamp::AbortsIfTimestampNotExists</a>;
</code></pre>



<a name="@Specification_1_exists_at"></a>

### Function `exists_at`


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): bool
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_address_of"></a>

### Function `address_of`


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered: store&gt;(offer_address: <b>address</b>): <b>address</b>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
</code></pre>



<a name="@Specification_1_take_offer"></a>

### Function `take_offer`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Offer.md#0x1_Offer_take_offer">take_offer</a>&lt;Offered: store&gt;(signer: signer, offer_address: <b>address</b>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
