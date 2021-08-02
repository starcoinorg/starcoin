
<a name="0x1_NFTGallery"></a>

# Module `0x1::NFTGallery`



-  [Struct `CreateEvent`](#0x1_NFTGallery_CreateEvent)
-  [Struct `TransferEvent`](#0x1_NFTGallery_TransferEvent)
-  [Resource `NFTGallery`](#0x1_NFTGallery_NFTGallery)
-  [Function `init`](#0x1_NFTGallery_init)
-  [Function `create_nft`](#0x1_NFTGallery_create_nft)
-  [Function `transfer_nft`](#0x1_NFTGallery_transfer_nft)
-  [Function `get_nft_info`](#0x1_NFTGallery_get_nft_info)
-  [Function `accept`](#0x1_NFTGallery_accept)
-  [Function `deposit`](#0x1_NFTGallery_deposit)
-  [Function `deposit_to`](#0x1_NFTGallery_deposit_to)
-  [Function `withdraw_one`](#0x1_NFTGallery_withdraw_one)
-  [Function `withdraw`](#0x1_NFTGallery_withdraw)
-  [Function `count_of`](#0x1_NFTGallery_count_of)


<pre><code><b>use</b> <a href="Collection2.md#0x1_Collection2">0x1::Collection2</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_NFTGallery_CreateEvent"></a>

## Struct `CreateEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_CreateEvent">CreateEvent</a>&lt;NFTType: drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>uid: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>creator: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_TransferEvent"></a>

## Struct `TransferEvent`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery_TransferEvent">TransferEvent</a>&lt;NFTType: drop, store&gt; has drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>from: address</code>
</dt>
<dd>

</dd>
<dt>
<code><b>to</b>: address</code>
</dt>
<dd>

</dd>
<dt>
<code>uid: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_NFTGallery"></a>

## Resource `NFTGallery`



<pre><code><b>struct</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType: drop, store&gt; has store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>create_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_CreateEvent">NFTGallery::CreateEvent</a>&lt;NFTType&gt;&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>transfer_events: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="NFT.md#0x1_NFTGallery_TransferEvent">NFTGallery::TransferEvent</a>&lt;NFTType&gt;&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_NFTGallery_init"></a>

## Function `init`

Init a NFTGallery to collect NFTs


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_init">init</a>&lt;NFTType: drop, store&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_init">init</a>&lt;NFTType: store + drop&gt;(signer: &signer) {
    <b>let</b> gallery = <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
        create_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_CreateEvent">CreateEvent</a>&lt;NFTType&gt;&gt;(signer),
        transfer_events: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="NFT.md#0x1_NFTGallery_TransferEvent">TransferEvent</a>&lt;NFTType&gt;&gt;(signer),
    };
    move_to&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(signer, gallery);
    <b>let</b> address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>if</b> (!<a href="Collection2.md#0x1_Collection2_exists_at">Collection2::exists_at</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(address)) {
        <a href="Collection2.md#0x1_Collection2_create_collection">Collection2::create_collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(signer, <b>false</b>, <b>false</b>);
    };
}
</code></pre>



</details>

<a name="0x1_NFTGallery_create_nft"></a>

## Function `create_nft`

Create a NFT under the signer


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_create_nft">create_nft</a>&lt;NFTType: drop, store&gt;(signer: &signer, hash: vector&lt;u8&gt;, nft_type: NFTType)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_create_nft">create_nft</a>&lt;NFTType: store + drop&gt;(signer: &signer, hash: vector&lt;u8&gt;, nft_type: NFTType) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(address);

    <b>let</b> nft = <a href="NFT.md#0x1_NFT_mint">NFT::mint</a>&lt;NFTType&gt;(signer, hash, nft_type);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.create_events, <a href="NFT.md#0x1_NFTGallery_CreateEvent">CreateEvent</a>&lt;NFTType&gt; {
        uid: <a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(&nft),
        hash: <a href="NFT.md#0x1_NFT_get_hash">NFT::get_hash</a>(&nft),
        creator: <a href="NFT.md#0x1_NFT_get_creator">NFT::get_creator</a>(&nft)
    });
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(signer, address, nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_transfer_nft"></a>

## Function `transfer_nft`

Transfer NFT from signer to reciver


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer_nft">transfer_nft</a>&lt;NFTType: drop, store&gt;(signer: &signer, uid: u64, receiver: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_transfer_nft">transfer_nft</a>&lt;NFTType: store + drop&gt;(signer: &signer, uid: u64, receiver: address) <b>acquires</b> <a href="NFT.md#0x1_NFTGallery">NFTGallery</a> {
    <b>let</b> address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer);
    <b>let</b> gallery = borrow_global_mut&lt;<a href="NFT.md#0x1_NFTGallery">NFTGallery</a>&lt;NFTType&gt;&gt;(address);
    <b>let</b> nfts = <a href="Collection2.md#0x1_Collection2_borrow_collection">Collection2::borrow_collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(signer, address);
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Collection2.md#0x1_Collection2_length">Collection2::length</a>(&nfts);
    // TODO: cache it?
    <b>while</b> (i &lt; len) {
        <b>if</b> (&<a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(<a href="Collection2.md#0x1_Collection2_borrow">Collection2::borrow</a>(&nfts, i)) == &uid) <b>break</b>;
        i = i + 1;
    };
    <b>let</b> nft = <a href="Collection2.md#0x1_Collection2_remove">Collection2::remove</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(&<b>mut</b> nfts, i);
    <a href="Collection2.md#0x1_Collection2_return_collection">Collection2::return_collection</a>(nfts);
    <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>(&<b>mut</b> gallery.transfer_events, <a href="NFT.md#0x1_NFTGallery_TransferEvent">TransferEvent</a>&lt;NFTType&gt; { from: address, <b>to</b>: receiver, uid: <a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(&nft) });
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(signer, receiver, nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_get_nft_info"></a>

## Function `get_nft_info`

Get the NFT info


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info">get_nft_info</a>&lt;NFTType: drop, store&gt;(account: &signer, uid: u64): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_get_nft_info">get_nft_info</a>&lt;NFTType: store + drop&gt;(account: &signer, uid: u64): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt; {
    <b>let</b> nfts = <a href="Collection2.md#0x1_Collection2_borrow_collection">Collection2::borrow_collection</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(account, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
    <b>let</b> i = 0;
    <b>let</b> len = <a href="Collection2.md#0x1_Collection2_length">Collection2::length</a>(&nfts);
    //TODO: cache it?
    <b>while</b> (i &lt; len) {
        <b>if</b> (&<a href="NFT.md#0x1_NFT_get_uid">NFT::get_uid</a>(<a href="Collection2.md#0x1_Collection2_borrow">Collection2::borrow</a>(&nfts, i)) == &uid) <b>break</b>;
        i = i + 1;
    };
    <b>let</b> nft = <b>if</b> (i != len) {
        <b>let</b> nft = <a href="Collection2.md#0x1_Collection2_borrow">Collection2::borrow</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(&<b>mut</b> nfts, i);
        <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="NFT.md#0x1_NFT_get_info">NFT::get_info</a>(nft))
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;NFTType&gt;&gt;()
    };
    <a href="Collection2.md#0x1_Collection2_return_collection">Collection2::return_collection</a>(nfts);
    <b>return</b> nft
}
</code></pre>



</details>

<a name="0x1_NFTGallery_accept"></a>

## Function `accept`



<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTType: drop, store&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_accept">accept</a>&lt;NFTType: store + drop&gt;(account: &signer) {
    <a href="Collection2.md#0x1_Collection2_accept">Collection2::accept</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(account);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit"></a>

## Function `deposit`

Deposit nft to <code>sender</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTType: drop, store&gt;(sender: &signer, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit">deposit</a>&lt;NFTType: store + drop&gt;(sender: &signer, nft:<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;){
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(sender, <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender), nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_deposit_to"></a>

## Function `deposit_to`

Deposit nft to <code>receiver</code> NFTGallery


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTType: drop, store&gt;(sender: &signer, receiver: address, nft: <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_deposit_to">deposit_to</a>&lt;NFTType: store + drop&gt;(sender: &signer, receiver: address, nft:<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;){
    <a href="Collection2.md#0x1_Collection2_put">Collection2::put</a>(sender, receiver, nft);
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw_one"></a>

## Function `withdraw_one`

Withdraw one nft of NFTType from <code>sender</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTType: drop, store&gt;(sender: &signer): <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw_one">withdraw_one</a>&lt;NFTType: store + drop&gt;(sender: &signer): <a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;{
    <a href="Collection2.md#0x1_Collection2_take">Collection2::take</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(sender)
}
</code></pre>



</details>

<a name="0x1_NFTGallery_withdraw"></a>

## Function `withdraw`

Withdraw nft of NFTType and uid from <code>sender</code>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTType: drop, store&gt;(_sender: &signer, _uid: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_withdraw">withdraw</a>&lt;NFTType: store + drop&gt;(_sender: &signer, _uid: u64){
    //TODO
}
</code></pre>



</details>

<a name="0x1_NFTGallery_count_of"></a>

## Function `count_of`

Count all NFTs assigned to an owner


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTType: drop, store&gt;(owner: address): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="NFT.md#0x1_NFTGallery_count_of">count_of</a>&lt;NFTType: store + drop&gt;(owner: address):u64 {
    <a href="Collection2.md#0x1_Collection2_length_of">Collection2::length_of</a>&lt;<a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTType&gt;&gt;(owner)
}
</code></pre>



</details>
