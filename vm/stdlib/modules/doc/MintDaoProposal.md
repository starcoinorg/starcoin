
<a name="0x1_MintDaoProposal"></a>

# Module `0x1::MintDaoProposal`



-  [Resource <code><a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a></code>](#0x1_MintDaoProposal_WrappedMintCapability)
-  [Struct <code><a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a></code>](#0x1_MintDaoProposal_MintToken)
-  [Const <code><a href="MintDaoProposal.md#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a></code>](#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED)
-  [Function <code>plugin</code>](#0x1_MintDaoProposal_plugin)
-  [Function <code>propose_mint_to</code>](#0x1_MintDaoProposal_propose_mint_to)
-  [Function <code>execute_mint_proposal</code>](#0x1_MintDaoProposal_execute_mint_proposal)


<a name="0x1_MintDaoProposal_WrappedMintCapability"></a>

## Resource `WrappedMintCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Token.md#0x1_Token_MintCapability">Token::MintCapability</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_MintDaoProposal_MintToken"></a>

## Struct `MintToken`



<pre><code><b>struct</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>receiver: address</code>
</dt>
<dd>

</dd>
<dt>
<code>amount: u128</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_MintDaoProposal_ERR_NOT_AUTHORIZED"></a>

## Const `ERR_NOT_AUTHORIZED`



<pre><code><b>const</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_MintDaoProposal_plugin"></a>

## Function `plugin`



<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_plugin">plugin</a>&lt;TokenT&gt;(signer: &signer) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(signer) == token_issuer, <a href="MintDaoProposal.md#0x1_MintDaoProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>);
    <b>let</b> mint_cap = <a href="Token.md#0x1_Token_remove_mint_capability">Token::remove_mint_capability</a>&lt;TokenT&gt;(signer);
    move_to(signer, <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a> { cap: mint_cap });
}
</code></pre>



</details>

<a name="0x1_MintDaoProposal_propose_mint_to"></a>

## Function `propose_mint_to`



<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_propose_mint_to">propose_mint_to</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, receiver: address, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_propose_mint_to">propose_mint_to</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, receiver: address, amount: u128) {
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;(
        signer,
        <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a> { receiver, amount },
        <a href="Dao.md#0x1_Dao_min_action_delay">Dao::min_action_delay</a>&lt;TokenT&gt;(),
    );
}
</code></pre>



</details>

<a name="0x1_MintDaoProposal_execute_mint_proposal"></a>

## Function `execute_mint_proposal`



<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_execute_mint_proposal">execute_mint_proposal</a>&lt;TokenT: <b>copyable</b>&gt;(signer: &signer, proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_execute_mint_proposal">execute_mint_proposal</a>&lt;TokenT: <b>copyable</b>&gt;(
    signer: &signer,
    proposer_address: address,
    proposal_id: u64,
) <b>acquires</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a> {
    <b>let</b> <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a> { receiver, amount } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="MintDaoProposal.md#0x1_MintDaoProposal_MintToken">MintToken</a>&gt;(
        proposer_address,
        proposal_id,
    );
    <b>let</b> cap = borrow_global&lt;<a href="MintDaoProposal.md#0x1_MintDaoProposal_WrappedMintCapability">WrappedMintCapability</a>&lt;TokenT&gt;&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>let</b> tokens = <a href="Token.md#0x1_Token_mint_with_capability">Token::mint_with_capability</a>&lt;TokenT&gt;(&cap.cap, amount);
    <a href="Account.md#0x1_Account_deposit_to">Account::deposit_to</a>(signer, receiver, tokens);
}
</code></pre>



</details>
