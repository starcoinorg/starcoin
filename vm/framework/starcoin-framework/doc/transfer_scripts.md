
<a id="0x1_transfer_scripts"></a>

# Module `0x1::transfer_scripts`



-  [Constants](#@Constants_0)
-  [Function `peer_to_peer`](#0x1_transfer_scripts_peer_to_peer)
-  [Function `peer_to_peer_v2`](#0x1_transfer_scripts_peer_to_peer_v2)
-  [Function `batch_peer_to_peer`](#0x1_transfer_scripts_batch_peer_to_peer)
-  [Function `batch_peer_to_peer_v2`](#0x1_transfer_scripts_batch_peer_to_peer_v2)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
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



<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer">peer_to_peer</a>&lt;TokenType: store&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, _payee_auth_key: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer">peer_to_peer</a>&lt;TokenType: store&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
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



<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">peer_to_peer_v2</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payee: <b>address</b>, amount: u128) {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">transfer_scripts::peer_to_peer_v2</a> | Entered"));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="coin.md#0x1_coin_name">coin::name</a>&lt;TokenType&gt;());
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&<a href="coin.md#0x1_coin_symbol">coin::symbol</a>&lt;TokenType&gt;());
    <a href="account.md#0x1_account_create_account_if_does_not_exist">account::create_account_if_does_not_exist</a>(payee);
    <a href="coin.md#0x1_coin_register">coin::register</a>&lt;TokenType&gt;(&<a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(payee));
    <a href="coin.md#0x1_coin_transfer">coin::transfer</a>&lt;TokenType&gt;(&<a href="account.md#0x1_account">account</a>, payee, (amount <b>as</b> u64));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"<a href="transfer_scripts.md#0x1_transfer_scripts_peer_to_peer_v2">transfer_scripts::peer_to_peer_v2</a> | Exited"));
}
</code></pre>



</details>

<a id="0x1_transfer_scripts_batch_peer_to_peer"></a>

## Function `batch_peer_to_peer`

Batch transfer token to others.


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, _payee_auth_keys: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;&gt;, amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer">batch_peer_to_peer</a>&lt;TokenType: store&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
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


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType: store&gt;(<a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;, amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="transfer_scripts.md#0x1_transfer_scripts_batch_peer_to_peer_v2">batch_peer_to_peer_v2</a>&lt;TokenType: store&gt;(
    <a href="account.md#0x1_account">account</a>: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    payeees: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;<b>address</b>&gt;,
    amounts: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u128&gt;
) {
    <b>let</b> len = <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&payeees);
    <b>assert</b>!(len == <a href="../../move-stdlib/doc/vector.md#0x1_vector_length">vector::length</a>(&amounts), <a href="transfer_scripts.md#0x1_transfer_scripts_ELENGTH_MISMATCH">ELENGTH_MISMATCH</a>);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len) {
        <b>let</b> payee = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&payeees, i);
        <a href="account.md#0x1_account_create_account_if_does_not_exist">account::create_account_if_does_not_exist</a>(payee);
        <a href="coin.md#0x1_coin_register">coin::register</a>&lt;TokenType&gt;(&<a href="account.md#0x1_account">account</a>);
        <b>let</b> amount = *<a href="../../move-stdlib/doc/vector.md#0x1_vector_borrow">vector::borrow</a>(&amounts, i);
        <a href="coin.md#0x1_coin_transfer">coin::transfer</a>&lt;TokenType&gt;(&<a href="account.md#0x1_account">account</a>, payee, (amount <b>as</b> u64));
        i = i + 1;
    }
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
