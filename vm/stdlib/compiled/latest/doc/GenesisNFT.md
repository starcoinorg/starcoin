
<a name="0x1_GenesisNFTScripts"></a>

# Module `0x1::GenesisNFTScripts`



-  [Function `mint`](#0x1_GenesisNFTScripts_mint)
-  [Specification](#@Specification_0)


<pre><code><b>use</b> <a href="GenesisNFT.md#0x1_GenesisNFT">0x1::GenesisNFT</a>;
</code></pre>



<a name="0x1_GenesisNFTScripts_mint"></a>

## Function `mint`

Mint a GenesisNFT


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFTScripts_mint">mint</a>(sender: signer, index: u64, merkle_proof: vector&lt;vector&lt;u8&gt;&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="GenesisNFT.md#0x1_GenesisNFTScripts_mint">mint</a>(sender: signer, index: u64, merkle_proof:vector&lt;vector&lt;u8&gt;&gt;) {
    <a href="GenesisNFT.md#0x1_GenesisNFT_mint">GenesisNFT::mint</a>(&sender, index, merkle_proof);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
