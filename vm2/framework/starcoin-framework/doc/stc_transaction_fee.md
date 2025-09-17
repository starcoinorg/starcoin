
<a id="0x1_stc_transaction_fee"></a>

# Module `0x1::stc_transaction_fee`

<code>TransactionFee</code> collect gas fees used by transactions in blocks temporarily.


-  [Function `atomic_counter_fetch_add`](#0x1_stc_transaction_fee_atomic_counter_fetch_add)
-  [Function `next_storage_address`](#0x1_stc_transaction_fee_next_storage_address)
-  [Function `pay_fee`](#0x1_stc_transaction_fee_pay_fee)
-  [Function `distribute_transaction_fees`](#0x1_stc_transaction_fee_distribute_transaction_fees)
-  [Specification](#@Specification_0)
    -  [Function `distribute_transaction_fees`](#@Specification_0_distribute_transaction_fees)


<pre><code><b>use</b> <a href="coin.md#0x1_coin">0x1::coin</a>;
<b>use</b> <a href="create_signer.md#0x1_create_signer">0x1::create_signer</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/debug.md#0x1_debug">0x1::debug</a>;
<b>use</b> <a href="../../starcoin-stdlib/doc/from_bcs.md#0x1_from_bcs">0x1::from_bcs</a>;
<b>use</b> <a href="../../move-stdlib/doc/string.md#0x1_string">0x1::string</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_stc_transaction_fee_atomic_counter_fetch_add"></a>

## Function `atomic_counter_fetch_add`



<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_atomic_counter_fetch_add">atomic_counter_fetch_add</a>(): u128
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>native</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_atomic_counter_fetch_add">atomic_counter_fetch_add</a>() : u128;
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_next_storage_address"></a>

## Function `next_storage_address`

Helper function to create a storage account address from predefined addresses


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_next_storage_address">next_storage_address</a>&lt;TokenType&gt;(range_from: u128, range_to: u128): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_next_storage_address">next_storage_address</a>&lt;TokenType&gt;(range_from: u128, range_to: u128): <b>address</b> {
    <b>assert</b>!(range_to &gt; range_from, 0);
    <b>if</b> (range_to == range_from + 1) {
        <a href="../../starcoin-stdlib/doc/from_bcs.md#0x1_from_bcs_u128_to_address">from_bcs::u128_to_address</a>(range_from)
    } <b>else</b> {
        <b>loop</b> {
            <b>let</b> counter = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_atomic_counter_fetch_add">atomic_counter_fetch_add</a>();

            <b>let</b> range = range_to - range_from - 1;
            <b>let</b> addr_u128 = range_from + (counter % range);

            <b>let</b> addr = <a href="../../starcoin-stdlib/doc/from_bcs.md#0x1_from_bcs_u128_to_address">from_bcs::u128_to_address</a>(addr_u128);
            // avoid using the framework <a href="account.md#0x1_account">account</a> <b>address</b>, which is prone <b>to</b> create conflict
            <b>if</b> (addr != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()) {
                <b>return</b> addr;
            }
        }
    }
}
</code></pre>



</details>

<a id="0x1_stc_transaction_fee_pay_fee"></a>

## Function `pay_fee`

Deposit <code>token</code> into one of the storage accounts


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_pay_fee">pay_fee</a>&lt;TokenType&gt;(token: <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;) {
    // Get the target genesis <a href="account.md#0x1_account">account</a> <b>address</b>
    <b>let</b> range_from = <a href="system_addresses.md#0x1_system_addresses_reserved_account_from">system_addresses::reserved_account_from</a>();
    <b>let</b> range_to = <a href="system_addresses.md#0x1_system_addresses_reserved_account_to">system_addresses::reserved_account_to</a>();
    <b>let</b> deposit_address = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_next_storage_address">next_storage_address</a>&lt;TokenType&gt;(range_from, range_to);

    // Deposit the fee directly <b>to</b> the selected genesis <a href="account.md#0x1_account">account</a>
    <a href="coin.md#0x1_coin_deposit">coin::deposit</a>(deposit_address, token);
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

    <b>let</b> range_from = <a href="system_addresses.md#0x1_system_addresses_reserved_account_from">system_addresses::reserved_account_from</a>();
    <b>let</b> range_to = <a href="system_addresses.md#0x1_system_addresses_reserved_account_to">system_addresses::reserved_account_to</a>();
    <b>let</b> first_withdraw_address = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_next_storage_address">next_storage_address</a>&lt;TokenType&gt;(range_from, range_to);

    <b>while</b> (<b>true</b>) {
        <b>let</b> withdraw_address = <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_next_storage_address">next_storage_address</a>&lt;TokenType&gt;(range_from, range_to);

        <b>let</b> balance = <a href="coin.md#0x1_coin_balance">coin::balance</a>&lt;TokenType&gt;(withdraw_address);
        <b>if</b> (balance &gt; 0) {
            // Create <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a> for the genesis <a href="account.md#0x1_account">account</a> and withdraw all funds
            <b>let</b> genesis_signer = <a href="create_signer.md#0x1_create_signer_create_signer">create_signer::create_signer</a>(withdraw_address);
            <b>let</b> withdrawn_coin = <a href="coin.md#0x1_coin_withdraw">coin::withdraw</a>&lt;TokenType&gt;(&genesis_signer, balance);
            <a href="coin.md#0x1_coin_merge">coin::merge</a>(&<b>mut</b> total_fees, withdrawn_coin);
        };

        <b>if</b> (withdraw_address == first_withdraw_address) <b>break</b>;
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



<a id="@Specification_0_distribute_transaction_fees"></a>

### Function `distribute_transaction_fees`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_fee.md#0x1_stc_transaction_fee_distribute_transaction_fees">distribute_transaction_fees</a>&lt;TokenType&gt;(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="coin.md#0x1_coin_Coin">coin::Coin</a>&lt;TokenType&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
