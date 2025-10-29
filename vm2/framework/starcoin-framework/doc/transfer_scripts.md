
<a id="0x1_transfer_scripts"></a>

# Module `0x1::transfer_scripts`



-  [Constants](#@Constants_0)
-  [Function `peer_to_peer`](#0x1_transfer_scripts_peer_to_peer)
-  [Function `peer_to_peer_v2`](#0x1_transfer_scripts_peer_to_peer_v2)
-  [Function `batch_peer_to_peer`](#0x1_transfer_scripts_batch_peer_to_peer)
-  [Function `batch_peer_to_peer_v2`](#0x1_transfer_scripts_batch_peer_to_peer_v2)


<pre><code><b>use</b> <a href="starcoin_account.md#0x1_starcoin_account">0x1::starcoin_account</a>;
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x1_transfer_scripts_EDEPRECATED_FUNCTION"></a>



<pre><code><b>const</b> <a href="transfer_scripts.md#0x1_transfer_scripts_EDEPRECATED_FUNCTION">EDEPRECATED_FUNCTION</a>: u64 = 19;
</code></pre>



<a id="0x1_transfer_scripts_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="transfer_scripts.md#0x1_transfer_scripts_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 101;
</code></pre>



<a id="0x1_transfer_scripts_ELENGTH_MISMATCH"></a>



<pre><code><b>const</b> <a href="transfer_scripts.md#0x1_transfer_scripts_ELENGTH_MISMATCH">ELENGTH_MISMATCH</a>: u64 = 102;
</code></pre>



<a id="0x1_transfer_scripts_peer_to_peer"></a>

## Function `peer_to_peer`



<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer">peer_to_peer</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, _payee_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer">peer_to_peer</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payee: <b>address</b>,
    _payee_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    amount: u128
) {
    <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>, payee, amount)
}
</code></pre>



</details>

<a id="0x1_transfer_scripts_peer_to_peer_v2"></a>

## Function `peer_to_peer_v2`



<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, amount: u128) {
    <a href="starcoin_account.md#0x1_starcoin_account_transfer_coins">starcoin_account::transfer_coins</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>, payee, (amount <b>as</b> u64));
}
</code></pre>



</details>

<a id="0x1_transfer_scripts_batch_peer_to_peer"></a>

## Function `batch_peer_to_peer`

Batch transfer token to others.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, _payee_auth_keys: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;,
    _payee_auth_keys: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;,
    amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;
) {
    <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>, payeees, amounts)
}
</code></pre>



</details>

<a id="0x1_transfer_scripts_batch_peer_to_peer_v2"></a>

## Function `batch_peer_to_peer_v2`

Batch transfer token to others.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;,
    amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;
) {
    <b>let</b> amounts_u64 = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u64&gt;();
    for (i in 0 .. <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&amounts)) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> amounts_u64, (*<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&amounts, i) <b>as</b> u64));
    };
    <a href="starcoin_account.md#0x1_starcoin_account_batch_transfer_coins">starcoin_account::batch_transfer_coins</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>, payeees, amounts_u64);
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
