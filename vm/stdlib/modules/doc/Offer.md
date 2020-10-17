
<a name="0x1_Offer"></a>

# Module `0x1::Offer`



-  [Resource `Offer`](#0x1_Offer_Offer)
-  [Constants](#@Constants_0)
-  [Function `create`](#0x1_Offer_create)
-  [Function `redeem`](#0x1_Offer_redeem)
-  [Function `exists_at`](#0x1_Offer_exists_at)
-  [Function `address_of`](#0x1_Offer_address_of)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_Offer_Offer"></a>

## Resource `Offer`



<pre><code><b>resource</b> <b>struct</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;
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
<code>for: address</code>
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



<pre><code><b>const</b> <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>: u64 = 101;
</code></pre>



<a name="0x1_Offer_EOFFER_NOT_UNLOCKED"></a>



<pre><code><b>const</b> <a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>: u64 = 102;
</code></pre>



<a name="0x1_Offer_create"></a>

## Function `create`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address, lock_peroid: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_create">create</a>&lt;Offered&gt;(account: &signer, offered: Offered, for: address, lock_peroid: u64) {
    <b>let</b> time_lock = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>() + lock_peroid;
    //TODO should support multi <a href="Offer.md#0x1_Offer">Offer</a>?
    move_to(account, <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for, time_lock });
}
</code></pre>



</details>

<a name="0x1_Offer_redeem"></a>

## Function `redeem`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_redeem">redeem</a>&lt;Offered&gt;(account: &signer, offer_address: address): Offered <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
    <b>let</b> <a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt; { offered, for, time_lock } = move_from&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address);
    <b>let</b> sender = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> now = <a href="Timestamp.md#0x1_Timestamp_now_seconds">Timestamp::now_seconds</a>();
    <b>assert</b>(sender == for || sender == offer_address, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>));
    <b>assert</b>(now &gt;= time_lock, <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>));
    offered
}
</code></pre>



</details>

<a name="0x1_Offer_exists_at"></a>

## Function `exists_at`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_exists_at">exists_at</a>&lt;Offered&gt;(offer_address: address): bool {
    <b>exists</b>&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address)
}
</code></pre>



</details>

<a name="0x1_Offer_address_of"></a>

## Function `address_of`



<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="Offer.md#0x1_Offer_address_of">address_of</a>&lt;Offered&gt;(offer_address: address): address <b>acquires</b> <a href="Offer.md#0x1_Offer">Offer</a> {
    borrow_global&lt;<a href="Offer.md#0x1_Offer">Offer</a>&lt;Offered&gt;&gt;(offer_address).for
}
</code></pre>



</details>
