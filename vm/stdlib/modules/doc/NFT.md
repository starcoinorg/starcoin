
<a name="0x1_NFTGallery"></a>

# Module `0x1::NFTGallery`

NFTGallery is user collection of NFT.


-  [Struct `WithdrawEvent`](#0x1_NFTGallery_WithdrawEvent)
-  [Struct `DepositEvent`](#0x1_NFTGallery_DepositEvent)
-  [Resource `NFTGallery`](#0x1_NFTGallery_NFTGallery)
-  [Constants](#@Constants_0)
-  [Function `accept`](#0x1_NFTGallery_accept)
-  [Function `transfer`](#0x1_NFTGallery_transfer)
-  [Function `get_nft_info`](#0x1_NFTGallery_get_nft_info)
-  [Function `deposit`](#0x1_NFTGallery_deposit)
-  [Function `deposit_to`](#0x1_NFTGallery_deposit_to)
-  [Function `withdraw_one`](#0x1_NFTGallery_withdraw_one)
-  [Function `withdraw`](#0x1_NFTGallery_withdraw)
-  [Function `do_withdraw`](#0x1_NFTGallery_do_withdraw)
-  [Function `find_by_uid`](#0x1_NFTGallery_find_by_uid)
-  [Function `count_of`](#0x1_NFTGallery_count_of)


<pre><code><b>use</b> <a href="Collection2.md#0x1_Collection2">0x1::Collection2</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_NFTGallery_WithdrawEvent"></a>

## Struct `WithdrawEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTType: <b>copy</b>, drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_DepositEvent"></a>

## Struct `DepositEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTType: <b>copy</b>, drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_NFTGallery"></a>

## Resource `NFTGallery`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType: <b>copy</b>, drop, store&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>withdraw_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_WithdrawEvent">NFTGallery::WithdrawEvent</a>&lt;NFTType&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>deposit_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_DepositEvent">NFTGallery::DepositEvent</a>&lt;NFTType&gt;&gt;</code>
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

Init a NFTGallery to accept NFTType


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer) {
    <b>let</b> gallery = <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
        withdraw_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTType&gt;&gt;(sender),
        deposit_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTType&gt;&gt;(sender),
    };
    move_to&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(sender, gallery);
    <a href="Collection2.md#0x1_Collection2_accept">Collection2::accept</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(sender);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_transfer"></a>

## Function `transfer`

Transfer NFT from <code>sender</code> to <code>receiver</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer">transfer</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer, uid: u64, receiver: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer">transfer</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer, uid: u64, receiver: address) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> nft = <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTType&gt;(sender, uid);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&nft), <a href="Errors.md#0x1_Errors_not_published">Errors::not_published</a>(<a href="NFT.md#0x1_NFTGallery_ERR_NFT_NOT_EXISTS">ERR_NFT_NOT_EXISTS</a>));
    <b>let</b> nft = <a href="Option.md#0x1_Option_destroy_some">Option::destroy_some</a>(nft);
    <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>(sender, receiver, nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_get_nft_info"></a>

## Function `get_nft_info`

Get the NFT info


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info">get_nft_info</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(account: &signer, uid: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info">get_nft_info</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(account: &signer, uid: u64): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt; {
    <b>let</b> nfts = <a href="Collection2.md#0x1_Collection2_borrow_collection">Collection2::borrow_collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(account, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>let</b> idx = <a href="NFT.md#0x1_NFTGallery_find_by_uid">find_by_uid</a>&lt;NFTType&gt;(&nfts, uid);

    <b>let</b> info = <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&idx)) {
        <b>let</b> i = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> idx);
        <b>let</b> nft = <a href="Collection2.md#0x1_Collection2_borrow">Collection2::borrow</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(&<b>mut</b> nfts, i);
        <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="NFT.md#0x1_NFT_get_info">NFT::get_info</a>(nft))
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt;()
    };
    <a href="Collection2.md#0x1_Collection2_return_collection">Collection2::return_collection</a>(nfts);
    <b>return</b> info
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit"></a>

## Function `deposit`

Deposit nft to <code>sender</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer, nft:<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>(sender, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender), nft)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit_to"></a>

## Function `deposit_to`

Deposit nft to <code>receiver</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer, receiver: address, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer, receiver: address, nft:<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(receiver);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.deposit_events, <a href="NFT.md#0x1_NFTGallery_DepositEvent">DepositEvent</a>&lt;NFTType&gt; { uid: <a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(&nft) });
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(sender, receiver, nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw_one"></a>

## Function `withdraw_one`

Withdraw one nft of NFTType from <code>sender</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTType&gt;(sender, <a href="Option.md#0x1_Option_none">Option::none</a>())
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw"></a>

## Function `withdraw`

Withdraw nft of NFTType and uid from <code>sender</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer, uid: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer, uid: u64) : <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
   <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>(sender, <a href="Option.md#0x1_Option_some">Option::some</a>(uid))
}
</code></pre>



</details>

<a name="0x1_NFTGallery_do_withdraw"></a>

## Function `do_withdraw`

Withdraw nft of NFTType and uid from <code>sender</code>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(sender: &signer, uid: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_do_withdraw">do_withdraw</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(sender: &signer, uid: <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt;) : <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt; <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>{
    <b>let</b> sender_addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(sender_addr);
    <b>let</b> nfts = <a href="Collection2.md#0x1_Collection2_borrow_collection">Collection2::borrow_collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(sender, sender_addr);
    <b>let</b> len = <a href="Collection2.md#0x1_Collection2_length">Collection2::length</a>(&nfts);
    <b>let</b> nft = <b>if</b>(len == 0){
        <a href="Option.md#0x1_Option_none">Option::none</a>()
    }<b>else</b>{
        <b>let</b> idx = <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&uid)){
            <b>let</b> uid = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> uid);
            <a href="NFT.md#0x1_NFTGallery_find_by_uid">find_by_uid</a>(&nfts, uid)
        }<b>else</b>{
            //default withdraw the last nft.
            <a href="Option.md#0x1_Option_some">Option::some</a>(len -1)
        };

        <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&idx)){
            <b>let</b> i = <a href="Option.md#0x1_Option_extract">Option::extract</a>(&<b>mut</b> idx);
            <b>let</b> nft = <a href="Collection2.md#0x1_Collection2_remove">Collection2::remove</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(&<b>mut</b> nfts, i);
            <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.withdraw_events, <a href="NFT.md#0x1_NFTGallery_WithdrawEvent">WithdrawEvent</a>&lt;NFTType&gt; { uid: <a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(&nft) });
            <a href="Option.md#0x1_Option_some">Option::some</a>(nft)
        }<b>else</b>{
            <a href="Option.md#0x1_Option_none">Option::none</a>()
        }
    };
    <a href="Collection2.md#0x1_Collection2_return_collection">Collection2::return_collection</a>(nfts);
    nft
}
</code></pre>



</details>

<a name="0x1_NFTGallery_find_by_uid"></a>

## Function `find_by_uid`



<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_find_by_uid">find_by_uid</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(c: &<a href="Collection2.md#0x1_Collection2_Collection">Collection2::Collection</a>&lt;<a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;&gt;, uid: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="NFT.md#0x1_NFTGallery_find_by_uid">find_by_uid</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(c: &<a href="Collection.md#0x1_Collection">Collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;, uid: u64): <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt;{
    <b>let</b> len = <a href="Collection2.md#0x1_Collection2_length">Collection2::length</a>(c);
    <b>if</b>(len == 0){
        <b>return</b> <a href="Option.md#0x1_Option_none">Option::none</a>()
    };
    <b>let</b> idx = len - 1;
    <b>loop</b> {
        <b>let</b> nft = <a href="Collection2.md#0x1_Collection2_borrow">Collection2::borrow</a>(c, idx);
        <b>if</b> (<a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(nft) == uid){
            <b>return</b> <a href="Option.md#0x1_Option_some">Option::some</a>(idx)
        };
        <b>if</b>(idx == 0){
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


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTType: <b>copy</b>, drop, store&gt;(owner: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTType: <b>copy</b> + store + drop&gt;(owner: address):u64 {
    <a href="Collection2.md#0x1_Collection2_length_of">Collection2::length_of</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(owner)
}
</code></pre>



</details>
