
<a id="0x1_treasury_scripts"></a>

# Module `0x1::treasury_scripts`



-  [Function `withdraw_and_split_lt_withdraw_cap`](#0x1_treasury_scripts_withdraw_and_split_lt_withdraw_cap)
-  [Function `withdraw_token_with_linear_withdraw_capability`](#0x1_treasury_scripts_withdraw_token_with_linear_withdraw_capability)
-  [Function `propose_withdraw`](#0x1_treasury_scripts_propose_withdraw)
-  [Function `execute_withdraw_proposal`](#0x1_treasury_scripts_execute_withdraw_proposal)
-  [Specification](#@Specification_0)
    -  [Function `withdraw_and_split_lt_withdraw_cap`](#@Specification_0_withdraw_and_split_lt_withdraw_cap)
    -  [Function `withdraw_token_with_linear_withdraw_capability`](#@Specification_0_withdraw_token_with_linear_withdraw_capability)
    -  [Function `propose_withdraw`](#@Specification_0_propose_withdraw)
    -  [Function `execute_withdraw_proposal`](#@Specification_0_execute_withdraw_proposal)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal">0x1::dao_treasury_withdraw_proposal</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_offer.md#0x1_stc_offer">0x1::stc_offer</a>;
<b>use</b> <a href="treasury.md#0x1_treasury">0x1::treasury</a>;
</code></pre>



<a id="0x1_treasury_scripts_withdraw_and_split_lt_withdraw_cap"></a>

## Function `withdraw_and_split_lt_withdraw_cap`

Withdraw token from treasury and split the LinearWithdrawCapability.


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, for_address: <b>address</b>, amount: u128, lock_period: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    for_address: <b>address</b>,
    amount: u128,
    lock_period: u64,
) {
    // 1. take cap: LinearWithdrawCapability&lt;TokenT&gt;
    <b>let</b> cap = <a href="treasury.md#0x1_treasury_remove_linear_withdraw_capability">treasury::remove_linear_withdraw_capability</a>&lt;TokenT&gt;(&<a href="account.md#0x1_account">account</a>);

    // 2. withdraw token and split
    <b>let</b> (tokens, new_cap) = <a href="treasury.md#0x1_treasury_split_linear_withdraw_cap">treasury::split_linear_withdraw_cap</a>(&<b>mut</b> cap, amount);

    // 3. deposit
    <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&<a href="account.md#0x1_account">account</a>), tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_capability">treasury::is_empty_linear_withdraw_capability</a>(&cap)) {
        <a href="treasury.md#0x1_treasury_destroy_linear_withdraw_capability">treasury::destroy_linear_withdraw_capability</a>(cap);
    } <b>else</b> {
        <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">treasury::add_linear_withdraw_capability</a>(&<a href="account.md#0x1_account">account</a>, cap);
    };

    // 5. offer
    <a href="stc_offer.md#0x1_stc_offer_create">stc_offer::create</a>(&<a href="account.md#0x1_account">account</a>, new_cap, for_address, lock_period);
}
</code></pre>



</details>

<a id="0x1_treasury_scripts_withdraw_token_with_linear_withdraw_capability"></a>

## Function `withdraw_token_with_linear_withdraw_capability`

Withdraw token from treasury.


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    // 1. take cap
    <b>let</b> cap = <a href="treasury.md#0x1_treasury_remove_linear_withdraw_capability">treasury::remove_linear_withdraw_capability</a>&lt;TokenT&gt;(&<a href="account.md#0x1_account">account</a>);

    // 2. withdraw token
    <b>let</b> tokens = <a href="treasury.md#0x1_treasury_withdraw_with_linear_capability">treasury::withdraw_with_linear_capability</a>(&<b>mut</b> cap);

    // 3. deposit
    <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(&<a href="account.md#0x1_account">account</a>), tokens);

    // 4. put or destroy key
    <b>if</b> (<a href="treasury.md#0x1_treasury_is_empty_linear_withdraw_capability">treasury::is_empty_linear_withdraw_capability</a>(&cap)) {
        <a href="treasury.md#0x1_treasury_destroy_linear_withdraw_capability">treasury::destroy_linear_withdraw_capability</a>(cap);
    } <b>else</b> {
        <a href="treasury.md#0x1_treasury_add_linear_withdraw_capability">treasury::add_linear_withdraw_capability</a>(&<a href="account.md#0x1_account">account</a>, cap);
    };
}
</code></pre>



</details>

<a id="0x1_treasury_scripts_propose_withdraw"></a>

## Function `propose_withdraw`

Propose a withdraw from treasury.


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    receiver: <b>address</b>,
    amount: u128,
    period: u64,
    exec_delay: u64
) {
    <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal_propose_withdraw">dao_treasury_withdraw_proposal::propose_withdraw</a>&lt;TokenT&gt;(&<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver, amount, period, exec_delay)
}
</code></pre>



</details>

<a id="0x1_treasury_scripts_execute_withdraw_proposal"></a>

## Function `execute_withdraw_proposal`

Execute a withdraw proposal.


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64) {
    <a href="dao_treasury_withdraw_proposal.md#0x1_dao_treasury_withdraw_proposal_execute_withdraw_proposal">dao_treasury_withdraw_proposal::execute_withdraw_proposal</a>&lt;TokenT&gt;(&<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address, proposal_id);
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification


<a id="@Specification_0_withdraw_and_split_lt_withdraw_cap"></a>

### Function `withdraw_and_split_lt_withdraw_cap`


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_and_split_lt_withdraw_cap">withdraw_and_split_lt_withdraw_cap</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, for_address: <b>address</b>, amount: u128, lock_period: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_0_withdraw_token_with_linear_withdraw_capability"></a>

### Function `withdraw_token_with_linear_withdraw_capability`


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_withdraw_token_with_linear_withdraw_capability">withdraw_token_with_linear_withdraw_capability</a>&lt;TokenT&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_0_propose_withdraw"></a>

### Function `propose_withdraw`


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_propose_withdraw">propose_withdraw</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, receiver: <b>address</b>, amount: u128, period: u64, exec_delay: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a id="@Specification_0_execute_withdraw_proposal"></a>

### Function `execute_withdraw_proposal`


<pre><code><b>public</b> entry <b>fun</b> <a href="treasury_scripts.md#0x1_treasury_scripts_execute_withdraw_proposal">execute_withdraw_proposal</a>&lt;TokenT&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, proposer_address: <b>address</b>, proposal_id: u64)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
