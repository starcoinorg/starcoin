
<a name="peer_to_peer_batch"></a>

# Script `peer_to_peer_batch`



-  [Constants](#@Constants_0)


<pre><code><b>use</b> <a href="../../modules/doc/Account.md#0x1_Account">0x1::Account</a>;
<b>use</b> <a href="../../modules/doc/BCS.md#0x1_BCS">0x1::BCS</a>;
<b>use</b> <a href="../../modules/doc/Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="../../modules/doc/Vector.md#0x1_Vector">0x1::Vector</a>;
</code></pre>



<a name="@Constants_0"></a>

## Constants


<a name="peer_to_peer_batch_EADDRESS_AND_AUTH_KEY_MISMATCH"></a>



<pre><code><b>const</b> <a href="peer_to_peer_batch.md#peer_to_peer_batch_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>: u64 = 101;
</code></pre>




<pre><code><b>public</b> <b>fun</b> <a href="peer_to_peer_batch.md#peer_to_peer_batch">peer_to_peer_batch</a>&lt;TokenType&gt;(account: &signer, payeees: vector&lt;u8&gt;, payee_auth_keys: vector&lt;u8&gt;, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="peer_to_peer_batch.md#peer_to_peer_batch">peer_to_peer_batch</a>&lt;TokenType&gt;(account: &signer, payeees: vector&lt;u8&gt;, payee_auth_keys: vector&lt;u8&gt;, amount: u128) {
    <b>let</b> payee_bytes_vec = <a href="../../modules/doc/Vector.md#0x1_Vector_split">Vector::split</a>&lt;u8&gt;(&payeees, 16);
    <b>let</b> auth_key_bytes_vec = <a href="../../modules/doc/Vector.md#0x1_Vector_split">Vector::split</a>&lt;u8&gt;(&payee_auth_keys, 32);
    <b>let</b> len = <a href="../../modules/doc/Vector.md#0x1_Vector_length">Vector::length</a>(&payee_bytes_vec);
    <b>let</b> i = 0;
    <b>while</b> (i &lt; len){
        <b>let</b> payee_bytes  = *<a href="../../modules/doc/Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;vector&lt;u8&gt;&gt;(&payee_bytes_vec, i);
        <b>let</b> payee = <a href="../../modules/doc/BCS.md#0x1_BCS_to_address">BCS::to_address</a>(payee_bytes);
        <b>let</b> payee_auth_key = *<a href="../../modules/doc/Vector.md#0x1_Vector_borrow">Vector::borrow</a>&lt;vector&lt;u8&gt;&gt;(&auth_key_bytes_vec, i);
        <b>if</b> (!<a href="../../modules/doc/Account.md#0x1_Account_exists_at">Account::exists_at</a>(payee)) {
        <b>let</b> created_address = <a href="../../modules/doc/Account.md#0x1_Account_create_account">Account::create_account</a>&lt;TokenType&gt;(payee_auth_key);
        <b>assert</b>(payee == created_address, <a href="../../modules/doc/Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="peer_to_peer_batch.md#peer_to_peer_batch_EADDRESS_AND_AUTH_KEY_MISMATCH">EADDRESS_AND_AUTH_KEY_MISMATCH</a>));
        };
        <a href="../../modules/doc/Account.md#0x1_Account_pay_from">Account::pay_from</a>&lt;TokenType&gt;(account, payee, amount);
        i = i + 1;
    }
}
</code></pre>



</details>
