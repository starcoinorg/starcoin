
<a name="0x1_NFTGalleryScripts"></a>

# Module `0x1::NFTGalleryScripts`



-  [Function `accept`](#0x1_NFTGalleryScripts_accept)
-  [Function `transfer`](#0x1_NFTGalleryScripts_transfer)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="NFT.md#0x1_NFTGallery">0x1::NFTGallery</a>;
</code></pre>



<a name="0x1_NFTGalleryScripts_accept"></a>

## Function `accept`

Init a  NFTGallery for accept NFT<NFTMeta, NFTBody>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFTGalleryScripts_accept">accept</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFTGalleryScripts_accept">accept</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: signer) {
    <a href="NFT.md#0x1_NFTGallery_accept">NFTGallery::accept</a>&lt;NFTMeta, NFTBody&gt;(&sender);
}
</code></pre>



</details>

<a name="0x1_NFTGalleryScripts_transfer"></a>

## Function `transfer`

Transfer NFT<NFTMeta, NFTBody> with <code>id</code> from <code>sender</code> to <code>receiver</code>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFTGalleryScripts_transfer">transfer</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store&gt;(sender: signer, id: u64, receiver: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="NFT.md#0x1_NFTGalleryScripts_transfer">transfer</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store&gt;(sender: signer, id: u64, receiver: <b>address</b>) {
    <a href="NFT.md#0x1_NFTGallery_transfer">NFTGallery::transfer</a>&lt;NFTMeta, NFTBody&gt;(&sender, id, receiver);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
