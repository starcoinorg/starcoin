
<a name="0x1_Offer"></a>

# Module `0x1::Offer`



-  [Resource <code><a href="Offer.md#0x1_Offer">Offer</a></code>](#0x1_Offer_Offer)
-  [Function <code>EOFFER_DNE_FOR_ACCOUNT</code>](#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT)
-  [Function <code>EOFFER_NOT_UNLOCKED</code>](#0x1_Offer_EOFFER_NOT_UNLOCKED)
-  [Function <code>create</code>](#0x1_Offer_create)
-  [Function <code>redeem</code>](#0x1_Offer_redeem)
-  [Function <code>exists_at</code>](#0x1_Offer_exists_at)
-  [Function <code>address_of</code>](#0x1_Offer_address_of)


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

<a name="0x1_Offer_EOFFER_DNE_FOR_ACCOUNT"></a>

## Function `EOFFER_DNE_FOR_ACCOUNT`



<pre><code><b>fun</b> <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 1
}
</code></pre>



</details>

<a name="0x1_Offer_EOFFER_NOT_UNLOCKED"></a>

## Function `EOFFER_NOT_UNLOCKED`



<pre><code><b>fun</b> <a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>(): u64 {
    <a href="ErrorCode.md#0x1_ErrorCode_ECODE_BASE">ErrorCode::ECODE_BASE</a>() + 2
}
</code></pre>



</details>

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
    <b>assert</b>(sender == for || sender == offer_address, <a href="Offer.md#0x1_Offer_EOFFER_DNE_FOR_ACCOUNT">EOFFER_DNE_FOR_ACCOUNT</a>());
    <b>assert</b>(now &gt;= time_lock, <a href="Offer.md#0x1_Offer_EOFFER_NOT_UNLOCKED">EOFFER_NOT_UNLOCKED</a>());
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
