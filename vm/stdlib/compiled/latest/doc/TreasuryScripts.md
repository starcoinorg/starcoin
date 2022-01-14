
<a name="0x1_TreasuryScripts"></a>

# Module `0x1::TreasuryScripts`



-  [Function `withdraw_and_split_lt_withdraw_cap`](#0x1_TreasuryScripts_withdraw_and_split_lt_withdraw_cap)
-  [Function `withdraw_token_with_linear_withdraw_capability`](#0x1_TreasuryScripts_withdraw_token_with_linear_withdraw_capability)
-  [Function `propose_withdraw`](#0x1_TreasuryScripts_propose_withdraw)
-  [Function `execute_withdraw_proposal`](#0x1_TreasuryScripts_execute_withdraw_proposal)
-  [Specification](#@Specification_0)
    -  [Function `withdraw_and_split_lt_withdraw_cap`](#@Specification_0_withdraw_and_split_lt_withdraw_cap)
    -  [Function `withdraw_token_with_linear_withdraw_capability`](#@Specification_0_withdraw_token_with_linear_withdraw_capability)
    -  [Function `propose_withdraw`](#@Specification_0_propose_withdraw)
    -  [Function `execute_withdraw_proposal`](#@Specification_0_execute_withdraw_proposal)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Offer.md#0x1_Offer">0x1::Offer</a>;
<b>use</b> <a href="Token.md#0x1_Token">0x1::Token</a>;
<b>use</b> <a href="Treasury.md#0x1_Treasury">0x1::Treasury</a>;
<b>use</b> <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal">0x1::TreasuryWithdrawDaoProposal</a>;
</code></pre>



<a name="0x1_TreasuryScripts_withdraw_and_split_lt_withdraw_cap"></a>

## Function `withdraw_and_split_lt_withdraw_cap`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT: store&gt;(signer: signer, for_address: <b>address</b>, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT: store&gt;(
    signer: signer,
    for_address: <b>address</b>,
    amount: u128,
    lock_period: u64,
) {
    // 1. take cap: LinearWithdrawCapability&lt;TokenT&gt;
    <b>let</b> cap = <a href="Treasury.md#0x1_Treasury_remove_linear_withdraw_capability">Treasury::remove_linear_withdraw_capability</a>&lt;TokenT&gt;(&signer);

    // 2. withdraw token and split
    <b>let</b> (tokens, new_cap) = <a href="Treasury.md#0x1_Treasury_split_linear_withdraw_cap">Treasury::split_linear_withdraw_cap</a>(&<b>mut</b> cap, amount);

    // 3. deposit
    <a href="Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(&signer, tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="Treasury.md#0x1_Treasury_is_empty_linear_withdraw_capability">Treasury::is_empty_linear_withdraw_capability</a>(&cap)) {
        <a href="Treasury.md#0x1_Treasury_destroy_linear_withdraw_capability">Treasury::destroy_linear_withdraw_capability</a>(cap);
    } <b>else</b> {
        <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">Treasury::add_linear_withdraw_capability</a>(&signer, cap);
    };

    // 5. offer
    <a href="Offer.md#0x1_Offer_create">Offer::create</a>(&signer, new_cap, for_address, lock_period);
}
</code></pre>



</details>

<a name="0x1_TreasuryScripts_withdraw_token_with_linear_withdraw_capability"></a>

## Function `withdraw_token_with_linear_withdraw_capability`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT: store&gt;(signer: signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT: store&gt;(
    signer: signer,
) {
    // 1. take cap
    <b>let</b> cap = <a href="Treasury.md#0x1_Treasury_remove_linear_withdraw_capability">Treasury::remove_linear_withdraw_capability</a>&lt;TokenT&gt;(&signer);

    // 2. withdraw token
    <b>let</b> tokens = <a href="Treasury.md#0x1_Treasury_withdraw_with_linear_capability">Treasury::withdraw_with_linear_capability</a>(&<b>mut</b> cap);

    // 3. deposit
    <a href="Account.md#0x1_Account_deposit_to_self">Account::deposit_to_self</a>(&signer, tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="Treasury.md#0x1_Treasury_is_empty_linear_withdraw_capability">Treasury::is_empty_linear_withdraw_capability</a>(&cap)) {
        <a href="Treasury.md#0x1_Treasury_destroy_linear_withdraw_capability">Treasury::destroy_linear_withdraw_capability</a>(cap);
    } <b>else</b> {
        <a href="Treasury.md#0x1_Treasury_add_linear_withdraw_capability">Treasury::add_linear_withdraw_capability</a>(&signer, cap);
    };
}
</code></pre>



</details>

<a name="0x1_TreasuryScripts_propose_withdraw"></a>

## Function `propose_withdraw`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b> + drop + store&gt;(signer: signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64){
    <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_propose_withdraw">TreasuryWithdrawDaoProposal::propose_withdraw</a>&lt;TokenT&gt;(&signer, receiver, amount, period, exec_delay)
}
</code></pre>



</details>

<a name="0x1_TreasuryScripts_execute_withdraw_proposal"></a>

## Function `execute_withdraw_proposal`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT:<b>copy</b> + drop + store&gt;(signer: signer, proposer_address: <b>address</b>,
                                                                   proposal_id: u64,){
    <a href="TreasuryWithdrawDaoProposal.md#0x1_TreasuryWithdrawDaoProposal_execute_withdraw_proposal">TreasuryWithdrawDaoProposal::execute_withdraw_proposal</a>&lt;TokenT&gt;(&signer, proposer_address, proposal_id);
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification


<a name="@Specification_0_withdraw_and_split_lt_withdraw_cap"></a>

### Function `withdraw_and_split_lt_withdraw_cap`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT: store&gt;(signer: signer, for_address: <b>address</b>, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_withdraw_token_with_linear_withdraw_capability"></a>

### Function `withdraw_token_with_linear_withdraw_capability`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT: store&gt;(signer: signer)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_propose_withdraw"></a>

### Function `propose_withdraw`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_propose_withdraw">propose_withdraw</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_0_execute_withdraw_proposal"></a>

### Function `execute_withdraw_proposal`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TreasuryScripts.md#0x1_TreasuryScripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT: <b>copy</b>, drop, store&gt;(signer: signer, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>
