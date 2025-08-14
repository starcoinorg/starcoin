
<a id="0x1_stc_transaction_fee"></a>

# Module `0x1::stc_transaction_fee`

<code>TransactionFee</code> collect gas fees used by transactions in blocks temporarily.
Uses aggregator_v2 for parallel execution and distributes fees across 100 genesis accounts.


-  [Resource `AutoIncrementCounter`](#0x1_stc_transaction_fee_AutoIncrementCounter)
-  [Function `initialize`](#0x1_stc_transaction_fee_initialize)
-  [Function `add_txn_fee_token`](#0x1_stc_transaction_fee_add_txn_fee_token)
-  [Function `get_genesis_account_address`](#0x1_stc_transaction_fee_get_genesis_account_address)
-  [Function `pay_fee`](#0x1_stc_transaction_fee_pay_fee)
-  [Function `distribute_transaction_fees`](#0x1_stc_transaction_fee_distribute_transaction_fees)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `add_txn_fee_token`](#@Specification_0_add_txn_fee_token)
    -  [Function `pay_fee`](#@Specification_0_pay_fee)
    -  [Function `distribute_transaction_fees`](#@Specification_0_distribute_transaction_fees)


<pre><code><b>use</b> <a href="aggregator_v2.md#0x1_aggregator_v2">0x1::aggregator_v2</a>;
<b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/from_bcs.md#0x1_from_bcs">0x1::from_bcs</a>;
<b>use</b> <a href="starcoin_coin.md#0x1_starcoin_coin">0x1::starcoin_coin</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_stc_transaction_fee_AutoIncrementCounter"></a>

## Resource `AutoIncrementCounter`

The <code><a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a></code> resource holds an aggregator counter for parallel execution
and tracks which genesis account to send fees to next.


<pre><code><b>struct</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;TokenType&gt; <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>counter: <a href="aggregator_v2.md#0x1_aggregator_v2_Aggregator">aggregator_v2::Aggregator</a>&lt;u64&gt;</code>
</dt>
<dd>
 Counter that keeps incrementing to determine which genesis account to use
</dd>
</dl>


</details>

<a id="0x1_stc_transaction_fee_initialize"></a>

## Function `initialize`

Called in genesis. Sets up the needed resources to collect transaction fees using
the parallel aggregator approach.


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

publishing a wrapper of the <code><a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a></code> resource under <code>fee_account</code>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>) {
    <b>move_to</b>(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;TokenType&gt; {
            counter: <a href="aggregator_v2.md#0x1_aggregator_v2_create_unbounded_aggregator">aggregator_v2::create_unbounded_aggregator</a>(),
        }
    )
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_get_genesis_account_address"></a>

## Function `get_genesis_account_address`

Helper function to create a genesis account address from index (0-99)


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_get_genesis_account_address">get_genesis_account_address</a>(index: u64): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_get_genesis_account_address">get_genesis_account_address</a>(index: u64): <b>address</b> {
    // Create a 32-byte <b>address</b> for genesis <a href="account.md#0x1_account">account</a> (0x0b + index)
    <b>let</b> addr_value = 0x0b + index;
    <b>let</b> addr_bytes = <a href="../../move-stdlib/doc/vector.md#0x1_vector_empty">vector::empty</a>&lt;u8&gt;();

    // Add 31 zero bytes
    <b>let</b> j = 0;
    <b>while</b> (j &lt; 31) {
        <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> addr_bytes, 0u8);
        j = j + 1;
    };

    // Add the <b>address</b> value <b>as</b> the last byte
    <a href="../../move-stdlib/doc/vector.md#0x1_vector_push_back">vector::push_back</a>(&<b>mut</b> addr_bytes, (addr_value <b>as</b> u8));

    <a href="../../starcoin-stdlib/doc/from_bcs.md#0x1_from_bcs_to_address">from_bcs::to_address</a>(addr_bytes)
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into one of the 100 genesis accounts based on counter


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;) <b>acquires</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a> {
    <b>let</b> counter_resource = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;TokenType&gt;&gt;(
        <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    );

    // Increment counter and get which genesis <a href="account.md#0x1_account">account</a> <b>to</b> <b>use</b>
    <a href="aggregator_v2.md#0x1_aggregator_v2_add">aggregator_v2::add</a>(&<b>mut</b> counter_resource.counter, 1);
    <b>let</b> counter_value = <a href="aggregator_v2.md#0x1_aggregator_v2_read">aggregator_v2::read</a>(&counter_resource.counter);
    <b>let</b> genesis_account_index = counter_value % 100;

    // Get the target genesis <a href="account.md#0x1_account">account</a> <b>address</b>
    <b>let</b> target_address = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_get_genesis_account_address">get_genesis_account_address</a>(genesis_account_index);

    // Deposit the fee directly <b>to</b> the selected genesis <a href="account.md#0x1_account">account</a>
    <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(target_address, token);
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_distribute_transaction_fees"></a>

## Function `distribute_transaction_fees`

Collect transaction fees from all 100 genesis accounts and return total as coin.
This function iterates through all genesis accounts and withdraws available fees.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt; {
    <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | Entered"));

    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);

    // Create accumulator for all collected fees
    <b>let</b> total_fees = <a href="coin.md#0x1_coin_zero">coin::zero</a>&lt;TokenType&gt;();

    // Iterate through all 100 genesis accounts and collect their fees
    <b>let</b> i = 0;
    <b>while</b> (i &lt; 100) {
        <b>let</b> genesis_address = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_get_genesis_account_address">get_genesis_account_address</a>(i);

        // Check <b>if</b> the genesis <a href="account.md#0x1_account">account</a> <b>has</b> <a href="../../starcoin-stdlib/doc/any.md#0x1_any">any</a> balance
        <b>if</b> (<a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;TokenType&gt;(genesis_address) &gt; 0) {
            <b>let</b> account_balance = <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;TokenType&gt;(genesis_address);
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | Collecting from genesis <a href="account.md#0x1_account">account</a>: "));
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&i);
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b" <b>with</b> balance: "));
            <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&account_balance);

            // Create <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a> for the genesis <a href="account.md#0x1_account">account</a> and withdraw all funds
            <b>let</b> genesis_signer = <a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(genesis_address);
            <b>let</b> withdrawn_coin = <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>&lt;TokenType&gt;(&genesis_signer, account_balance);
            <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> total_fees, withdrawn_coin);
        };

        i = i + 1;
    };

    <b>let</b> total_value = <a href="coin.md#0x1_coin_value">coin::value</a>&lt;TokenType&gt;(&total_fees);
    <b>if</b> (total_value &gt; 0) {
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit <b>with</b> total value: "));
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&total_value);
    } <b>else</b> {
        <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug_print">debug::print</a>(&std::string::utf8(b"stc_block::distribute_transaction_fees | Exit <b>with</b> zero"));
    };

    total_fees
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
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;STC&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_0_add_txn_fee_token"></a>

### Function `add_txn_fee_token`


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_add_txn_fee_token">add_txn_fee_token</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;TokenType&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_0_pay_fee"></a>

### Function `pay_fee`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_fee.md#0x1_stc_transaction_fee_AutoIncrementCounter">AutoIncrementCounter</a>&lt;TokenType&gt;&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
</code></pre>



<a id="@Specification_0_distribute_transaction_fees"></a>

### Function `distribute_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
