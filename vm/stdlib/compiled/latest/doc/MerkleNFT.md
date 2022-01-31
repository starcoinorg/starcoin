
<a name="0x1_MerkleNFTDistributor"></a>

# Module `0x1::MerkleNFTDistributor`



-  [Resource `MerkleNFTDistribution`](#0x1_MerkleNFTDistributor_MerkleNFTDistribution)
-  [Constants](#@Constants_0)
-  [Function `register`](#0x1_MerkleNFTDistributor_register)
-  [Function `mint_with_cap`](#0x1_MerkleNFTDistributor_mint_with_cap)
-  [Function `encode_leaf`](#0x1_MerkleNFTDistributor_encode_leaf)
-  [Function `set_minted_`](#0x1_MerkleNFTDistributor_set_minted_)
-  [Function `verify_proof`](#0x1_MerkleNFTDistributor_verify_proof)
-  [Function `is_minted`](#0x1_MerkleNFTDistributor_is_minted)
-  [Function `is_minted_`](#0x1_MerkleNFTDistributor_is_minted_)


<pre><code><b>use</b> <a href="BCS.md#0x1_BCS">0x1::BCS</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Hash.md#0x1_Hash">0x1::Hash</a>;
<b>use</b> <a href="MerkleNFT.md#0x1_MerkleProof">0x1::MerkleProof</a>;
<b>use</b> <a href="NFT.md#0x1_NFT">0x1::NFT</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="0x1_MerkleNFTDistributor_MerkleNFTDistribution"></a>

## Resource `MerkleNFTDistribution`



<pre><code><b>struct</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>merkle_root: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>claimed_bitmap: vector&lt;u128&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_MerkleNFTDistributor_ERR_NO_MINT_CAPABILITY"></a>



<pre><code><b>const</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_ERR_NO_MINT_CAPABILITY">ERR_NO_MINT_CAPABILITY</a>: u64 = 1002;
</code></pre>



<a name="0x1_MerkleNFTDistributor_ALREADY_MINTED"></a>



<pre><code><b>const</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_ALREADY_MINTED">ALREADY_MINTED</a>: u64 = 1000;
</code></pre>



<a name="0x1_MerkleNFTDistributor_INVALID_PROOF"></a>



<pre><code><b>const</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_INVALID_PROOF">INVALID_PROOF</a>: u64 = 1001;
</code></pre>



<a name="0x1_MerkleNFTDistributor_register"></a>

## Function `register`



<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_register">register</a>&lt;NFTMeta: <b>copy</b>, drop, store, Info: <b>copy</b>, drop, store&gt;(signer: &signer, merkle_root: vector&lt;u8&gt;, leafs: u64, info: Info, meta: <a href="NFT.md#0x1_NFT_Metadata">NFT::Metadata</a>): <a href="NFT.md#0x1_NFT_MintCapability">NFT::MintCapability</a>&lt;NFTMeta&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_register">register</a>&lt;NFTMeta: <b>copy</b> + store + drop, Info: <b>copy</b> + store + drop&gt;(signer: &signer, merkle_root: vector&lt;u8&gt;, leafs: u64, info: Info, meta: Metadata): MintCapability&lt;NFTMeta&gt; {
    <b>let</b> bitmap_count = leafs / 128;
    <b>if</b> (bitmap_count * 128 &lt; leafs) {
        bitmap_count = bitmap_count + 1;
    };
    <b>let</b> claimed_bitmap = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <b>let</b> j = 0;
    <b>while</b> (j &lt; bitmap_count) {
        <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>( &<b>mut</b> claimed_bitmap, 0u128);
        j = j + 1;
    };
    <b>let</b> distribution = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;{
        merkle_root,
        claimed_bitmap
    };
    <a href="NFT.md#0x1_NFT_register">NFT::register</a>&lt;NFTMeta, Info&gt;(signer, info, meta);
    <b>move_to</b>(signer, distribution);
    <a href="NFT.md#0x1_NFT_remove_mint_capability">NFT::remove_mint_capability</a>&lt;NFTMeta&gt;(signer)
}
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_mint_with_cap"></a>

## Function `mint_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_mint_with_cap">mint_with_cap</a>&lt;NFTMeta: <b>copy</b>, drop, store, NFTBody: store, Info: <b>copy</b>, drop, store&gt;(sender: &signer, cap: &<b>mut</b> <a href="NFT.md#0x1_NFT_MintCapability">NFT::MintCapability</a>&lt;NFTMeta&gt;, creator: <b>address</b>, index: u64, base_meta: <a href="NFT.md#0x1_NFT_Metadata">NFT::Metadata</a>, type_meta: NFTMeta, body: NFTBody, merkle_proof: vector&lt;vector&lt;u8&gt;&gt;): <a href="NFT.md#0x1_NFT_NFT">NFT::NFT</a>&lt;NFTMeta, NFTBody&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_mint_with_cap">mint_with_cap</a>&lt;NFTMeta: <b>copy</b> + store + drop, NFTBody: store, Info: <b>copy</b> + store + drop&gt;(sender: &signer, cap:&<b>mut</b> MintCapability&lt;NFTMeta&gt;, creator: <b>address</b>, index: u64, base_meta: Metadata, type_meta: NFTMeta, body: NFTBody, merkle_proof:vector&lt;vector&lt;u8&gt;&gt;): <a href="NFT.md#0x1_NFT">NFT</a>&lt;NFTMeta, NFTBody&gt;
    <b>acquires</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a> {
        <b>let</b> addr = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(sender);
        <b>let</b> distribution = <b>borrow_global_mut</b>&lt;<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;&gt;(creator);
        <b>let</b> minted = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted_">is_minted_</a>&lt;NFTMeta&gt;(distribution, index);
        <b>assert</b>!(!minted, <a href="Errors.md#0x1_Errors_custom">Errors::custom</a>(<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_ALREADY_MINTED">ALREADY_MINTED</a>));
        <b>let</b> leaf_data = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_encode_leaf">encode_leaf</a>(&index, &addr);
        <b>let</b> verified = <a href="MerkleNFT.md#0x1_MerkleProof_verify">MerkleProof::verify</a>(&merkle_proof, &distribution.merkle_root, <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(leaf_data));
        <b>assert</b>!(verified, <a href="Errors.md#0x1_Errors_custom">Errors::custom</a>(<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_INVALID_PROOF">INVALID_PROOF</a>));
        <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_set_minted_">set_minted_</a>(distribution, index);
        <b>let</b> nft = <a href="NFT.md#0x1_NFT_mint_with_cap">NFT::mint_with_cap</a>&lt;NFTMeta, NFTBody, Info&gt;(creator, cap, base_meta, type_meta, body);
        <b>return</b> nft
    }
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_encode_leaf"></a>

## Function `encode_leaf`



<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_encode_leaf">encode_leaf</a>(index: &u64, account: &<b>address</b>): vector&lt;u8&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_encode_leaf">encode_leaf</a>(index: &u64, account: &<b>address</b>): vector&lt;u8&gt; {
    <b>let</b> leaf = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> leaf, <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(index));
    <a href="Vector.md#0x1_Vector_append">Vector::append</a>(&<b>mut</b> leaf, <a href="BCS.md#0x1_BCS_to_bytes">BCS::to_bytes</a>(account));
    leaf
}
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_set_minted_"></a>

## Function `set_minted_`



<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_set_minted_">set_minted_</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt;(distribution: &<b>mut</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistributor::MerkleNFTDistribution</a>&lt;NFTMeta&gt;, index: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_set_minted_">set_minted_</a>&lt;NFTMeta: <b>copy</b> + store + drop&gt;(distribution: &<b>mut</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;, index: u64) {
    <b>let</b> claimed_word_index = index / 128;
    <b>let</b> claimed_bit_index = ((index % 128) <b>as</b> u8);
    <b>let</b> word = <a href="Vector.md#0x1_Vector_borrow_mut">Vector::borrow_mut</a>(&<b>mut</b> distribution.claimed_bitmap, claimed_word_index);
    // word | (1 &lt;&lt; bit_index)
    <b>let</b> mask = 1u128 &lt;&lt; claimed_bit_index;
    *word = (*word | mask);
}
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_verify_proof"></a>

## Function `verify_proof`



<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_verify_proof">verify_proof</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt;(account: <b>address</b>, creator: <b>address</b>, index: u64, merkle_proof: vector&lt;vector&lt;u8&gt;&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_verify_proof">verify_proof</a>&lt;NFTMeta: <b>copy</b> + store + drop&gt;(account: <b>address</b>, creator: <b>address</b>, index: u64, merkle_proof:vector&lt;vector&lt;u8&gt;&gt;): bool
    <b>acquires</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a> {
        <b>let</b> distribution = <b>borrow_global_mut</b>&lt;<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;&gt;(creator);
        <b>let</b> leaf_data = <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_encode_leaf">encode_leaf</a>(&index, &account);
        <a href="MerkleNFT.md#0x1_MerkleProof_verify">MerkleProof::verify</a>(&merkle_proof, &distribution.merkle_root, <a href="Hash.md#0x1_Hash_sha3_256">Hash::sha3_256</a>(leaf_data))
    }
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_is_minted"></a>

## Function `is_minted`



<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted">is_minted</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt;(creator: <b>address</b>, index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted">is_minted</a>&lt;NFTMeta: <b>copy</b> + store + drop&gt;(creator: <b>address</b>, index: u64): bool
    <b>acquires</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a> {
        <b>let</b> distribution = <b>borrow_global_mut</b>&lt;<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;&gt;(creator);
        <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted_">is_minted_</a>&lt;NFTMeta&gt;(distribution, index)
    }
</code></pre>



</details>

<a name="0x1_MerkleNFTDistributor_is_minted_"></a>

## Function `is_minted_`



<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted_">is_minted_</a>&lt;NFTMeta: <b>copy</b>, drop, store&gt;(distribution: &<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistributor::MerkleNFTDistribution</a>&lt;NFTMeta&gt;, index: u64): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="MerkleNFT.md#0x1_MerkleNFTDistributor_is_minted_">is_minted_</a>&lt;NFTMeta: <b>copy</b> + store + drop&gt;(distribution: &<a href="MerkleNFT.md#0x1_MerkleNFTDistributor_MerkleNFTDistribution">MerkleNFTDistribution</a>&lt;NFTMeta&gt;, index: u64): bool {
    <b>let</b> claimed_word_index = index / 128;
    <b>let</b> claimed_bit_index = ((index % 128) <b>as</b> u8);
    <b>let</b> word = <a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>( &distribution.claimed_bitmap, claimed_word_index);
    <b>let</b> mask = 1u128 &lt;&lt; claimed_bit_index;
    (*word & mask) == mask
}
</code></pre>



</details>
