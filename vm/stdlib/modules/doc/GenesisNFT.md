
<a name="0x1_GenesisNFT"></a>

# Module `0x1::GenesisNFT`



-  [Struct `GenesisNFT`](#0x1_GenesisNFT_GenesisNFT)
-  [Struct `GenesisNFTMeta`](#0x1_GenesisNFT_GenesisNFTMeta)
-  [Struct `GenesisNFTInfo`](#0x1_GenesisNFT_GenesisNFTInfo)
-  [Resource `GenesisNFTMintCapability`](#0x1_GenesisNFT_GenesisNFTMintCapability)
-  [Function `initialize`](#0x1_GenesisNFT_initialize)
-  [Function `mint`](#0x1_GenesisNFT_mint)
-  [Function `get_info`](#0x1_GenesisNFT_get_info)


<pre><code><b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="NFT.md#0x1_IdentifierNFT">0x1::IdentifierNFT</a>;
<b>use</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor">0x1::MerkleNFTDistributor</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
</code></pre>



<a name="0x1_GenesisNFT_GenesisNFT"></a>

## Struct `GenesisNFT`



<pre><code><b>struct</b> <a href="GenesisNFT.md#0x1_GenesisNFT">GenesisNFT</a> has store
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

<a name="0x1_GenesisNFT_GenesisNFTMeta"></a>

## Struct `GenesisNFTMeta`



<pre><code><b>struct</b> <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a> has <b>copy</b>, drop, store
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

<a name="0x1_GenesisNFT_GenesisNFTInfo"></a>

## Struct `GenesisNFTInfo`



<pre><code><b>struct</b> <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTInfo">GenesisNFTInfo</a> has <b>copy</b>, drop, store
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

<a name="0x1_GenesisNFT_GenesisNFTMintCapability"></a>

## Resource `GenesisNFTMintCapability`



<pre><code><b>struct</b> <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMintCapability">GenesisNFTMintCapability</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="NFT.md#0x1_NFT_MintCapability">NFT::MintCapability</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFT::GenesisNFTMeta</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_GenesisNFT_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_initialize">initialize</a>(sender: &signer, merkle_root: vector&lt;u8&gt;, leafs: u64, image: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_initialize">initialize</a>(sender: &signer, merkle_root: vector&lt;u8&gt;, leafs: u64, image: vector&lt;u8&gt;){
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(sender);
    <b>let</b> metadata = <a href="NFT.md#0x1_NFT_new_meta_with_image">NFT::new_meta_with_image</a>(b"StarcoinGenesisNFT", image, b"The starcoin genesis <a href="NFT.md#0x1_NFT">NFT</a>");
    <b>let</b> nft_type_info=<a href="NFT.md#0x1_NFT_new_nft_type_info">NFT::new_nft_type_info</a>(sender, <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTInfo">GenesisNFTInfo</a>{}, metadata);
    <b>let</b> cap = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_register">MerkleNFTDistributor::register</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a>, <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTInfo">GenesisNFTInfo</a>&gt;(sender, merkle_root, leafs, nft_type_info);
    move_to(sender, <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMintCapability">GenesisNFTMintCapability</a>{cap});
}
</code></pre>



</details>

<a name="0x1_GenesisNFT_mint"></a>

## Function `mint`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_mint">mint</a>(sender: &signer, index: u64, merkle_proof: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_mint">mint</a>(sender: &signer, index: u64, merkle_proof:vector&lt;vector&lt;u8&gt;&gt;)
    <b>acquires</b> <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMintCapability">GenesisNFTMintCapability</a>{
        <b>let</b> metadata = <a href="NFT.md#0x1_NFT_empty_meta">NFT::empty_meta</a>();
        <b>let</b> cap = borrow_global_mut&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMintCapability">GenesisNFTMintCapability</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
        <b>let</b> nft = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_mint_with_cap">MerkleNFTDistributor::mint_with_cap</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a>, <a href="GenesisNFT.md#0x1_GenesisNFT">GenesisNFT</a>, <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTInfo">GenesisNFTInfo</a>&gt;(sender, &<b>mut</b> cap.cap, <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), index, metadata, <a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a>{}, <a href="GenesisNFT.md#0x1_GenesisNFT">GenesisNFT</a>{}, merkle_proof);
        <a href="NFT.md#0x1_IdentifierNFT_grant">IdentifierNFT::grant</a>(&<b>mut</b> cap.cap, sender, nft);
    }
</code></pre>



</details>

<a name="0x1_GenesisNFT_get_info"></a>

## Function `get_info`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_get_info">get_info</a>(owner: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFT::GenesisNFTMeta</a>&gt;&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFT_get_info">get_info</a>(owner: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="NFT.md#0x1_NFT_NFTInfo">NFT::NFTInfo</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a>&gt;&gt;{
    <a href="NFT.md#0x1_IdentifierNFT_get_nft_info">IdentifierNFT::get_nft_info</a>&lt;<a href="GenesisNFT.md#0x1_GenesisNFT_GenesisNFTMeta">GenesisNFTMeta</a>, <a href="GenesisNFT.md#0x1_GenesisNFT">GenesisNFT</a>&gt;(owner)
}
</code></pre>



</details>
