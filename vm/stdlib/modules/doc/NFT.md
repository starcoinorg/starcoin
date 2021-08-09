
<a name="0x1_NFTGallery"></a>

# Module `0x1::NFTGallery`

NFTGallery is user collection of NFT.


-  [Struct `WithdrawEvent`](#0x1_NFTGallery_WithdrawEvent)
-  [Struct `DepositEvent`](#0x1_NFTGallery_DepositEvent)
-  [Resource `NFTGallery`](#0x1_NFTGallery_NFTGallery)
-  [Constants](#@Constants_0)
-  [Function `accept`](#0x1_NFTGallery_accept)
-  [Function `transfer`](#0x1_NFTGallery_transfer)
-  [Function `get_nft_info_by_id`](#0x1_NFTGallery_get_nft_info_by_id)
-  [Function `get_nft_info_by_idx`](#0x1_NFTGallery_get_nft_info_by_idx)
-  [Function `get_nft_infos`](#0x1_NFTGallery_get_nft_infos)
-  [Function `deposit`](#0x1_NFTGallery_deposit)
-  [Function `deposit_to`](#0x1_NFTGallery_deposit_to)
-  [Function `withdraw_one`](#0x1_NFTGallery_withdraw_one)
-  [Function `withdraw`](#0x1_NFTGallery_withdraw)
-  [Function `do_withdraw`](#0x1_NFTGallery_do_withdraw)
-  [Function `find_by_id`](#0x1_NFTGallery_find_by_id)
-  [Function `count_of`](#0x1_NFTGallery_count_of)


<pre><code><b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_NFTGallery_WithdrawEvent"></a>

## Struct `WithdrawEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owner: address</code>
</dt>
<dd>

</dd>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_DepositEvent"></a>

## Struct `DepositEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>owner: address</code>
</dt>
<dd>

</dd>
<dt>
<code>id: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_NFTGallery"></a>

## Resource `NFTGallery`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>withdraw_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_WithdrawEvent">NFTGallery::WithdrawEvent</a>&lt;NFTMeta&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>deposit_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_DepositEvent">NFTGallery::DepositEvent</a>&lt;NFTMeta&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>items: vector&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_NFTGallery_ERR_NFT_NOT_EXISTS"></a>



<pre><code><b>const</b> <a href="NFT.md#0x1_NFTGallery_ERR_NFT_NOT_EXISTS">ERR_NFT_NOT_EXISTS</a>: u64 = 101;
</code></pre>



<a name="0x1_NFTGallery_accept"></a>

## Function `accept`

Init a NFTGallery to accept NFT<NFTMeta, NFTBody>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer) {
    <b>let</b> gallery = <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
        withdraw_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTMeta&gt;&gt;(sender),
        deposit_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTMeta&gt;&gt;(sender),
        items: <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;(),
    };
    move_to(sender, gallery);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_transfer"></a>

## Function `transfer`

Transfer NFT from <code>sender</code> to <code>receiver</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer">transfer</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer, id: u64, receiver: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer">transfer</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer, id: u64, receiver: address) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> nft = <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTMeta, NFTBody&gt;(sender, id);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&nft), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NFT.md#0x1_NFTGallery_ERR_NFT_NOT_EXISTS">ERR_NFT_NOT_EXISTS</a>));
    <b>let</b> nft = <a href="Option.md#0x1_Option_destroy_some">Option::destroy_some</a>(nft);
    <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>(receiver, nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_get_nft_info_by_id"></a>

## Function `get_nft_info_by_id`

Get the NFT info by the NFT id.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info_by_id">get_nft_info_by_id</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(owner: address, id: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info_by_id">get_nft_info_by_id</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(owner: address, id: u64): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(owner);
    <b>let</b> idx = <a href="NFT.md#0x1_NFTGallery_find_by_id">find_by_id</a>&lt;NFTMeta, NFTBody&gt;(&gallery.items, id);

    <b>let</b> info = <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&idx)) {
        <b>let</b> i = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> idx);
        <b>let</b> nft = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;(&gallery.items, i);
        <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="NFT.md#0x1_NFT_get_info">NFT::get_info</a>(nft))
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;&gt;()
    };
    <b>return</b> info
}
</code></pre>



</details>

<a name="0x1_NFTGallery_get_nft_info_by_idx"></a>

## Function `get_nft_info_by_idx`

Get the NFT info by the NFT idx in NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info_by_idx">get_nft_info_by_idx</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(owner: address, idx: u64): <a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info_by_idx">get_nft_info_by_idx</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(owner: address, idx: u64): <a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(owner);
    <b>let</b> nft = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;(&gallery.items, idx);
    <a href="NFT.md#0x1_NFT_get_info">NFT::get_info</a>(nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_get_nft_infos"></a>

## Function `get_nft_infos`

Get the all NFT info


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_infos">get_nft_infos</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(owner: address): vector&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_infos">get_nft_infos</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(owner: address): vector&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTMeta&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(owner);
    <b>let</b> infos = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&gallery.items);
    <b>let</b> idx = 0;
    <b>while</b>(len &gt; idx) {
        <b>let</b> nft = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;(&gallery.items, idx);
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> infos, <a href="NFT.md#0x1_NFT_get_info">NFT::get_info</a>(nft));
        idx = idx + 1;
    };
    infos
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit"></a>

## Function `deposit`

Deposit nft to <code>sender</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer, nft: <a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>(sender_addr, nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit_to"></a>

## Function `deposit_to`

Deposit nft to <code>receiver</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(receiver: address, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(receiver: address, nft: <a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(receiver);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.deposit_events, <a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTMeta&gt; { id: <a href="NFT.md#0x1_NFT_get_id">NFT::get_id</a>(&nft), owner: receiver });
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> gallery.items, nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw_one"></a>

## Function `withdraw_one`

Withdraw one nft of NFTMeta from <code>sender</code>, caller should ensure at least one NFT in the Gallery.


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer): <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer): <a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> nft = <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTMeta, NFTBody&gt;(sender, <a href="Option.md#0x1_Option_none">Option::none</a>());
    <a href="Option.md#0x1_Option_destroy_some">Option::destroy_some</a>(nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw"></a>

## Function `withdraw`

Withdraw nft of NFTMeta and id from <code>sender</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer, id: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer, id: u64): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>(sender, <a href="Option.md#0x1_Option_some">Option::some</a>(id))
}
</code></pre>



</details>

<a name="0x1_NFTGallery_do_withdraw"></a>

## Function `do_withdraw`

Withdraw nft of NFTMeta and id from <code>sender</code>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: &signer, id: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: &signer, id: <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt;): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(sender_addr);
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&gallery.items);
    <b>let</b> nft = <b>if</b> (len == 0) {
        <a href="Option.md#0x1_Option_none">Option::none</a>()
    }<b>else</b> {
        <b>let</b> idx = <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&id)) {
            <b>let</b> id = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> id);
            <a href="NFT.md#0x1_NFTGallery_find_by_id">find_by_id</a>(&gallery.items, id)
        }<b>else</b> {
            //default withdraw the last nft.
            <a href="Option.md#0x1_Option_some">Option::some</a>(len - 1)
        };

        <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&idx)) {
            <b>let</b> i = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> idx);
            <b>let</b> nft = <a href="Vector.md#0x1_Vector_remove">Vector::remove</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;(&<b>mut</b> gallery.items, i);
            <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.withdraw_events, <a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTMeta&gt; { id: <a href="NFT.md#0x1_NFT_get_id">NFT::get_id</a>(&nft), owner: sender_addr });
            <a href="Option.md#0x1_Option_some">Option::some</a>(nft)
        }<b>else</b> {
            <a href="Option.md#0x1_Option_none">Option::none</a>()
        }
    };
    nft
}
</code></pre>



</details>

<a name="0x1_NFTGallery_find_by_id"></a>

## Function `find_by_id`



<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_find_by_id">find_by_id</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(c: &vector&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;, id: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_find_by_id">find_by_id</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(c: &vector&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;&gt;, id: u64): <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt; {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(c);
    <b>if</b> (len == 0) {
        <b>return</b> <a href="Option.md#0x1_Option_none">Option::none</a>()
    };
    <b>let</b> idx = len - 1;
    <b>loop</b> {
        <b>let</b> nft = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(c, idx);
        <b>if</b> (<a href="NFT.md#0x1_NFT_get_id">NFT::get_id</a>(nft) == id) {
            <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(idx)
        };
        <b>if</b> (idx == 0) {
            <b>return</b> <a href="Option.md#0x1_Option_none">Option::none</a>()
        };
        idx = idx - 1;
    }
}
</code></pre>



</details>

<a name="0x1_NFTGallery_count_of"></a>

## Function `count_of`

Count all NFTs assigned to an owner


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(owner: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(owner: address): u64 <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTMeta, NFTBody&gt;&gt;(owner);
    <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&gallery.items)
}
</code></pre>



</details>
