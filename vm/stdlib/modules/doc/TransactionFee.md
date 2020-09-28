
<a name="0x1_TransactionFee"></a>

# Module `0x1::TransactionFee`



-  [Resource <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code>](#0x1_TransactionFee_TransactionFee)
-  [Function <code>initialize</code>](#0x1_TransactionFee_initialize)
-  [Function <code>add_txn_fee_token</code>](#0x1_TransactionFee_add_txn_fee_token)
-  [Function <code>pay_fee</code>](#0x1_TransactionFee_pay_fee)
-  [Function <code>distribute_transaction_fees</code>](#0x1_TransactionFee_distribute_transaction_fees)
-  [Specification](#@Specification_0)
    -  [Function <code>initialize</code>](#@Specification_0_initialize)
    -  [Function <code>add_txn_fee_token</code>](#@Specification_0_add_txn_fee_token)
    -  [Function <code>pay_fee</code>](#@Specification_0_pay_fee)
    -  [Function <code>distribute_transaction_fees</code>](#@Specification_0_distribute_transaction_fees)


<a name="0x1_TransactionFee_TransactionFee"></a>

## Resource `TransactionFee`

The <code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource holds a preburn resource for each
fiat <code>TokenType</code> that can be collected as a transaction fee.


<pre><code><b>resource</b> <b>struct</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fee: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_TransactionFee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code><a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a></code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(
    account: &signer,
) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    // accept fees in all the currencies
    <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_token">add_txn_fee_token</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;(account);
}
</code></pre>



</details>

<a name="0x1_TransactionFee_add_txn_fee_token"></a>

## Function `add_txn_fee_token`

publishing a wrapper of the <code>Preburn&lt;TokenType&gt;</code> resource under <code>fee_account</code>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(
    account: &signer,
) {
    move_to(
        account,
        <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt; {
            fee: <a href="Token.md#0x1_Token_zero">Token::zero</a>(),
        }
    )
 }
</code></pre>



</details>

<a name="0x1_TransactionFee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt;) <b>acquires</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a> {
    <b>let</b> txn_fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(
        <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()
    );
    <a href="Token.md#0x1_Token_deposit">Token::deposit</a>(&<b>mut</b> txn_fees.fee, token)
}
</code></pre>



</details>

<a name="0x1_TransactionFee_distribute_transaction_fees"></a>

## Function `distribute_transaction_fees`

Distribute the transaction fees collected in the <code>TokenType</code> token.
If the <code>TokenType</code> is STC, it unpacks the token and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(account: &signer): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(
    account: &signer,
): <a href="Token.md#0x1_Token">Token</a>&lt;TokenType&gt; <b>acquires</b> <a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a> {
    <b>let</b> fee_address =  <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>();
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == fee_address, <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());

    // extract fees
    <b>let</b> txn_fees = borrow_global_mut&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(fee_address);
    <b>let</b> value = <a href="Token.md#0x1_Token_value">Token::value</a>&lt;TokenType&gt;(&txn_fees.fee);
    <b>if</b> (value &gt; 0) {
        <a href="Token.md#0x1_Token_withdraw">Token::withdraw</a>(&<b>mut</b> txn_fees.fee, value)
    }<b>else</b> {
        <a href="Token.md#0x1_Token_zero">Token::zero</a>&lt;TokenType&gt;()
    }
}
</code></pre>



</details>

<a name="@Specification_0"></a>

## Specification



<pre><code>pragma verify;
pragma aborts_if_is_strict;
</code></pre>



<a name="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_initialize">initialize</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;<a href="STC.md#0x1_STC">STC</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_add_txn_fee_token"></a>

### Function `add_txn_fee_token`


<pre><code><b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="Signer.md#0x1_Signer_spec_address_of">Signer::spec_address_of</a>(account));
</code></pre>



<a name="@Specification_0_pay_fee"></a>

### Function `pay_fee`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>());
<b>aborts_if</b> <b>global</b>&lt;<a href="TransactionFee.md#0x1_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()).fee.value + token.value &gt; max_u128();
</code></pre>



<a name="@Specification_0_distribute_transaction_fees"></a>

### Function `distribute_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionFee.md#0x1_TransactionFee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(account: &signer): <a href="Token.md#0x1_Token_Token">Token::Token</a>&lt;TokenType&gt;
</code></pre>




<pre><code>pragma verify = <b>false</b>;
</code></pre>
