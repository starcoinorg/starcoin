
<a name="0x1_SignerManagingProposal"></a>

# Module `0x1::SignerManagingProposal`



-  [Resource `WrappedSignerCapability`](#0x1_SignerManagingProposal_WrappedSignerCapability)
-  [Struct `BorrowSignerProposal`](#0x1_SignerManagingProposal_BorrowSignerProposal)
-  [Constants](#@Constants_0)
-  [Function `plugin`](#0x1_SignerManagingProposal_plugin)
-  [Function `propose_borrow_signer`](#0x1_SignerManagingProposal_propose_borrow_signer)
-  [Function `execute_borrow_signer_proposal`](#0x1_SignerManagingProposal_execute_borrow_signer_proposal)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Dao.md#0x1_Dao">0x1::Dao</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
</code></pre>



<a name="0x1_SignerManagingProposal_WrappedSignerCapability"></a>

## Resource `WrappedSignerCapability`



<pre><code><b>struct</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_WrappedSignerCapability">WrappedSignerCapability</a> has key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_SignerManagingProposal_BorrowSignerProposal"></a>

## Struct `BorrowSignerProposal`



<pre><code><b>struct</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_BorrowSignerProposal">BorrowSignerProposal</a>&lt;Borrower&gt; has <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>allow: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_SignerManagingProposal_ERR_NOT_AUTHORIZED"></a>



<pre><code><b>const</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>: u64 = 401;
</code></pre>



<a name="0x1_SignerManagingProposal_plugin"></a>

## Function `plugin`

Plugin in this module to manage token address signer capability using DAO.


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer_cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_plugin">plugin</a>&lt;TokenT: store&gt;(signer_cap: <a href="Account.md#0x1_Account_SignerCapability">Account::SignerCapability</a>) {
    <b>let</b> token_issuer = <a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;();
    <b>assert</b>(<a href="Account.md#0x1_Account_signer_address">Account::signer_address</a>(&signer_cap) == token_issuer, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="SignerManagingProposal.md#0x1_SignerManagingProposal_ERR_NOT_AUTHORIZED">ERR_NOT_AUTHORIZED</a>));

    move_to(&<a href="Account.md#0x1_Account_borrow_signer_with_capability">Account::borrow_signer_with_capability</a>(&signer_cap), <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_WrappedSignerCapability">WrappedSignerCapability</a>{cap: signer_cap});
}
</code></pre>



</details>

<a name="0x1_SignerManagingProposal_propose_borrow_signer"></a>

## Function `propose_borrow_signer`

Propose <code>Borrower</code> can be borrow signer using <code><a href="Account.md#0x1_Account_borrow_signer">Account::borrow_signer</a>(borrower, signer_address)</code>


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_propose_borrow_signer">propose_borrow_signer</a>&lt;TokenT: <b>copy</b>, drop, store, Borrower: <b>copy</b>, drop, store&gt;(signer: &signer, allow: bool, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_propose_borrow_signer">propose_borrow_signer</a>&lt;TokenT: drop + store + <b>copy</b>, Borrower: drop + store + <b>copy</b>&gt;(signer: &signer, allow: bool, exec_delay: u64) {
    <a href="Dao.md#0x1_Dao_propose">Dao::propose</a>&lt;TokenT, <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_BorrowSignerProposal">BorrowSignerProposal</a>&lt;Borrower&gt;&gt;(signer, <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_BorrowSignerProposal">BorrowSignerProposal</a>&lt;Borrower&gt; { allow }, exec_delay);
}
</code></pre>



</details>

<a name="0x1_SignerManagingProposal_execute_borrow_signer_proposal"></a>

## Function `execute_borrow_signer_proposal`

Execute the borrow_signer proposal.


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_execute_borrow_signer_proposal">execute_borrow_signer_proposal</a>&lt;TokenT: <b>copy</b>, drop, store, Borrower: <b>copy</b>, drop, store&gt;(proposer_address: address, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_execute_borrow_signer_proposal">execute_borrow_signer_proposal</a>&lt;TokenT: drop + store + <b>copy</b>, Borrower: drop + store + <b>copy</b>&gt;(proposer_address: address, proposal_id: u64)
<b>acquires</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_WrappedSignerCapability">WrappedSignerCapability</a> {
    <b>let</b> <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_BorrowSignerProposal">BorrowSignerProposal</a>&lt;Borrower&gt; { allow } = <a href="Dao.md#0x1_Dao_extract_proposal_action">Dao::extract_proposal_action</a>&lt;TokenT, <a href="SignerManagingProposal.md#0x1_SignerManagingProposal_BorrowSignerProposal">BorrowSignerProposal</a>&lt;Borrower&gt;&gt;(
        proposer_address,
        proposal_id,
    );
    <b>let</b> cap = borrow_global&lt;<a href="SignerManagingProposal.md#0x1_SignerManagingProposal_WrappedSignerCapability">WrappedSignerCapability</a>&gt;(<a href="Token.md#0x1_Token_token_address">Token::token_address</a>&lt;TokenT&gt;());
    <b>if</b> (allow) {
        <a href="Account.md#0x1_Account_allow_borrow_signer">Account::allow_borrow_signer</a>&lt;Borrower&gt;(&cap.cap);
    } <b>else</b> {
        <a href="Account.md#0x1_Account_disallow_borrow_signer">Account::disallow_borrow_signer</a>&lt;Borrower&gt;(&cap.cap);
    };
}
</code></pre>



</details>
