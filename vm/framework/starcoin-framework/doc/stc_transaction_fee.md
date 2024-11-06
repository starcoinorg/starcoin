
<a id="0x1_stc_transaction_fee"></a>

# Module `0x1::stc_transaction_fee`

<code><a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a></code> collect gas fees used by transactions in blocks temporarily.
Then they are distributed in <code>TransactionManager</code>.


-  [Resource `TransactionFee`](#0x1_stc_transaction_fee_TransactionFee)
-  [Function `initialize`](#0x1_stc_transaction_fee_initialize)
-  [Function `add_txn_fee_token`](#0x1_stc_transaction_fee_add_txn_fee_token)
-  [Function `pay_fee`](#0x1_stc_transaction_fee_pay_fee)
-  [Function `distribute_transaction_fees`](#0x1_stc_transaction_fee_distribute_transaction_fees)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `add_txn_fee_token`](#@Specification_0_add_txn_fee_token)
    -  [Function `pay_fee`](#@Specification_0_pay_fee)
    -  [Function `distribute_transaction_fees`](#@Specification_0_distribute_transaction_fees)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_stc_transaction_fee_TransactionFee"></a>

## Resource `TransactionFee`

The <code><a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a></code> resource holds a preburn resource for each
fiat <code>TokenType</code> that can be collected as a transaction fee.


<pre><code><b>struct</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>fee: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_transaction_fee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees from the
<code><a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a></code> resource with the TreasuryCompliance account.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    // Timestamp::assert_genesis();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    // accept fees in all the currencies
    <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;STC&gt;(<a href="account.md#0x1_account">account</a>);
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_add_txn_fee_token"></a>

## Function `add_txn_fee_token`

publishing a wrapper of the <code>Preburn&lt;TokenType&gt;</code> resource under <code>fee_account</code>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>move_to</b>(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt; {
            fee: <a href="coin.md#0x1_coin_zero">coin::zero</a>(),
        }
    )
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into the transaction fees bucket


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;) <b>acquires</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a> {
    <b>let</b> txn_fees = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(
        <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    );
    <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> txn_fees.fee, token)
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_distribute_transaction_fees"></a>

## Function `distribute_transaction_fees`

Distribute the transaction fees collected in the <code>TokenType</code> token.
If the <code>TokenType</code> is STC, it unpacks the token and preburns the
underlying fiat.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt; <b>acquires</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a> {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | Entered"));

    <b>let</b> fee_address = <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    // extract fees
    <b>let</b> txn_fees = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(fee_address);
    <b>let</b> value = <a href="coin.md#0x1_coin_value">coin::value</a>&lt;TokenType&gt;(&txn_fees.fee);
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | value : "));
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&value);

    <b>if</b> (value &gt; 0) {
        <a href="coin.md#0x1_coin_extract">coin::extract</a>(&<b>mut</b> txn_fees.fee, value)
    } <b>else</b> {
        <a href="coin.md#0x1_coin_zero">coin::zero</a>&lt;TokenType&gt;()
    }
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;STC&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_0_add_txn_fee_token"></a>

### Function `add_txn_fee_token`


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_0_pay_fee"></a>

### Function `pay_fee`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
<b>aborts_if</b> <b>global</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(
    <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
).fee.value + token.value &gt; max_u128();
</code></pre>



<a id="@Specification_0_distribute_transaction_fees"></a>

### Function `distribute_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_TransactionFee">TransactionFee</a>&lt;TokenType&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
