
<a name="0x1_TransferScripts"></a>

# Module `0x1::TransferScripts`



-  [Constants](#@Constants_0)
-  [Function `peer_to_peer`](#0x1_TransferScripts_peer_to_peer)
-  [Function `peer_to_peer_v2`](#0x1_TransferScripts_peer_to_peer_v2)
-  [Function `batch_peer_to_peer`](#0x1_TransferScripts_batch_peer_to_peer)
-  [Function `batch_peer_to_peer_v2`](#0x1_TransferScripts_batch_peer_to_peer_v2)
-  [Function `peer_to_peer_batch`](#0x1_TransferScripts_peer_to_peer_batch)
-  [Function `peer_to_peer_with_metadata`](#0x1_TransferScripts_peer_to_peer_with_metadata)
-  [Function `peer_to_peer_with_metadata_v2`](#0x1_TransferScripts_peer_to_peer_with_metadata_v2)


<pre><code><b>use</b> <a href="Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="0x1_TransferScripts_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="TransferScripts.md#0x1_TransferScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 19;
</code></pre>



<a name="0x1_TransferScripts_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="TransferScripts.md#0x1_TransferScripts_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 101;
</code></pre>



<a name="0x1_TransferScripts_ELENGTH_MISMATCH"></a>



<pre><code><b>const</b> <a href="TransferScripts.md#0x1_TransferScripts_ELENGTH_MISMATCH">ELENGTH_MISMATCH</a>: u64 = 102;
</code></pre>



<a name="0x1_TransferScripts_peer_to_peer"></a>

## Function `peer_to_peer`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer">peer_to_peer</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, _payee_auth_key: vector&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer">peer_to_peer</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, _payee_auth_key: vector&lt;u8&gt;, amount: u128) {
     <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(account, payee, amount)
}
</code></pre>



</details>

<a name="0x1_TransferScripts_peer_to_peer_v2"></a>

## Function `peer_to_peer_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, amount: u128) {
    <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
        <a href="Account.md#0x1_Account_create_account_with_address">Account::create_account_with_address</a>&lt;TokenType&gt;(payee);
    };
    <a href="Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(&account, payee, amount)
}
</code></pre>



</details>

<a name="0x1_TransferScripts_batch_peer_to_peer"></a>

## Function `batch_peer_to_peer`

Batch transfer token to others.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(account: signer, payeees: vector&lt;<b>address</b>&gt;, _payee_auth_keys: vector&lt;vector&lt;u8&gt;&gt;, amounts: vector&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(account: signer, payeees: vector&lt;<b>address</b>&gt;, _payee_auth_keys: vector&lt;vector&lt;u8&gt;&gt;, amounts: vector&lt;u128&gt;) {
     <a href="TransferScripts.md#0x1_TransferScripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType&gt;(account, payeees, amounts)
}
</code></pre>



</details>

<a name="0x1_TransferScripts_batch_peer_to_peer_v2"></a>

## Function `batch_peer_to_peer_v2`

Batch transfer token to others.


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType: store&gt;(account: signer, payeees: vector&lt;<b>address</b>&gt;, amounts: vector&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType: store&gt;(account: signer, payeees: vector&lt;<b>address</b>&gt;, amounts: vector&lt;u128&gt;) {
    <b>let</b> len = <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&payeees);
    <b>assert</b>!(len == <a href="Vector.md#0x1_Vector_length">Vector::length</a>(&amounts), <a href="TransferScripts.md#0x1_TransferScripts_ELENGTH_MISMATCH">ELENGTH_MISMATCH</a>);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len){
        <b>let</b> payee = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&payeees, i);
        <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
            <a href="Account.md#0x1_Account_create_account_with_address">Account::create_account_with_address</a>&lt;TokenType&gt;(payee);
        };
        <b>let</b> amount = *<a href="Vector.md#0x1_Vector_borrow">Vector::borrow</a>(&amounts, i);
        <a href="Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(&account, payee, amount);
        i = i + 1;
    }
}
</code></pre>



</details>

<a name="0x1_TransferScripts_peer_to_peer_batch"></a>

## Function `peer_to_peer_batch`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_batch">peer_to_peer_batch</a>&lt;TokenType: store&gt;(_account: signer, _payeees: vector&lt;u8&gt;, _payee_auth_keys: vector&lt;u8&gt;, _amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_batch">peer_to_peer_batch</a>&lt;TokenType: store&gt;(_account: signer, _payeees: vector&lt;u8&gt;, _payee_auth_keys: vector&lt;u8&gt;, _amount: u128) {
    <b>abort</b> <a href="Errors.md#0x1_Errors_deprecated">Errors::deprecated</a>(<a href="TransferScripts.md#0x1_TransferScripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>)
}
</code></pre>



</details>

<a name="0x1_TransferScripts_peer_to_peer_with_metadata"></a>

## Function `peer_to_peer_with_metadata`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, _payee_auth_key: vector&lt;u8&gt;, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_with_metadata">peer_to_peer_with_metadata</a>&lt;TokenType: store&gt;(
    account: signer,
    payee: <b>address</b>,
    _payee_auth_key: vector&lt;u8&gt;,
    amount: u128,
    metadata: vector&lt;u8&gt;,
) {
     <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_with_metadata_v2">peer_to_peer_with_metadata_v2</a>&lt;TokenType&gt;(account, payee, amount, metadata)
}
</code></pre>



</details>

<a name="0x1_TransferScripts_peer_to_peer_with_metadata_v2"></a>

## Function `peer_to_peer_with_metadata_v2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_with_metadata_v2">peer_to_peer_with_metadata_v2</a>&lt;TokenType: store&gt;(account: signer, payee: <b>address</b>, amount: u128, metadata: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="TransferScripts.md#0x1_TransferScripts_peer_to_peer_with_metadata_v2">peer_to_peer_with_metadata_v2</a>&lt;TokenType: store&gt;(
        account: signer,
        payee: <b>address</b>,
        amount: u128,
        metadata: vector&lt;u8&gt;,
) {
    <b>if</b> (!<a href="Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
        <a href="Account.md#0x1_Account_create_account_with_address">Account::create_account_with_address</a>&lt;TokenType&gt;(payee);
    };
    <a href="Account.md#0x1_Account_pay_from_with_metadata">Account::pay_from_with_metadata</a>&lt;TokenType&gt;(&account,payee, amount, metadata)
}
</code></pre>



</details>
