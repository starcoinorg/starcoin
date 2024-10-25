
<a id="0x1_easy_gas_script"></a>

# Module `0x1::easy_gas_script`



-  [Function `register`](#0x1_easy_gas_script_register)
-  [Function `init_data_source`](#0x1_easy_gas_script_init_data_source)
-  [Function `update`](#0x1_easy_gas_script_update)
-  [Function `withdraw_gas_fee_entry`](#0x1_easy_gas_script_withdraw_gas_fee_entry)


<pre><code><b>use</b> <a href="easy_gas.md#0x1_easy_gas">0x1::easy_gas</a>;
</code></pre>



<a id="0x1_easy_gas_script_register"></a>

## Function `register`



<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_register">register</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_register">register</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, precision: u8) {
    <a href="easy_gas.md#0x1_easy_gas_register_oracle">easy_gas::register_oracle</a>&lt;TokenType&gt;(&sender, precision)
}
</code></pre>



</details>

<a id="0x1_easy_gas_script_init_data_source"></a>

## Function `init_data_source`



<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_init_data_source">init_data_source</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_init_data_source">init_data_source</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, init_value: u128) {
    <a href="easy_gas.md#0x1_easy_gas_init_oracle_source">easy_gas::init_oracle_source</a>&lt;TokenType&gt;(&sender, init_value);
}
</code></pre>



</details>

<a id="0x1_easy_gas_script_update"></a>

## Function `update`



<pre><code><b>public</b> entry <b>fun</b> <b>update</b>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <b>update</b>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, value: u128) {
    <a href="easy_gas.md#0x1_easy_gas_update_oracle">easy_gas::update_oracle</a>&lt;TokenType&gt;(&sender, value)
}
</code></pre>



</details>

<a id="0x1_easy_gas_script_withdraw_gas_fee_entry"></a>

## Function `withdraw_gas_fee_entry`



<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_withdraw_gas_fee_entry">withdraw_gas_fee_entry</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> entry <b>fun</b> <a href="easy_gas_script.md#0x1_easy_gas_script_withdraw_gas_fee_entry">withdraw_gas_fee_entry</a>&lt;TokenType: store&gt;(sender: <a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, amount: u128) {
    <a href="easy_gas.md#0x1_easy_gas_withdraw_gas_fee">easy_gas::withdraw_gas_fee</a>&lt;TokenType&gt;(&sender, amount);
}
</code></pre>



</details>


[move-book]: https://starcoin.dev/move/book/SUMMARY
