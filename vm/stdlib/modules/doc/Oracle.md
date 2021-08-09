
<a name="0x1_PriceOracleScripts"></a>

# Module `0x1::PriceOracleScripts`



-  [Function `register_oracle`](#0x1_PriceOracleScripts_register_oracle)
-  [Function `init_data_source`](#0x1_PriceOracleScripts_init_data_source)
-  [Function `update`](#0x1_PriceOracleScripts_update)


<pre><code><b>use</b> <a href="Oracle.md#0x1_PriceOracle">0x1::PriceOracle</a>;
</code></pre>



<a name="0x1_PriceOracleScripts_register_oracle"></a>

## Function `register_oracle`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Oracle.md#0x1_PriceOracleScripts_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: signer, precision: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Oracle.md#0x1_PriceOracleScripts_register_oracle">register_oracle</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: signer, precision: u8){
    <a href="Oracle.md#0x1_PriceOracle_register_oracle">PriceOracle::register_oracle</a>&lt;OracleT&gt;(&sender, precision)
}
</code></pre>



</details>

<a name="0x1_PriceOracleScripts_init_data_source"></a>

## Function `init_data_source`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Oracle.md#0x1_PriceOracleScripts_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: signer, init_value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="Oracle.md#0x1_PriceOracleScripts_init_data_source">init_data_source</a>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: signer, init_value: u128){
    <a href="Oracle.md#0x1_PriceOracle_init_data_source">PriceOracle::init_data_source</a>&lt;OracleT&gt;(&sender, init_value);
}
</code></pre>



</details>

<a name="0x1_PriceOracleScripts_update"></a>

## Function `update`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>, drop, store&gt;(sender: signer, value: u128)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <b>update</b>&lt;OracleT: <b>copy</b>+store+drop&gt;(sender: signer, value: u128){
    <a href="Oracle.md#0x1_PriceOracle_update">PriceOracle::update</a>&lt;OracleT&gt;(&sender, value);
}
</code></pre>



</details>
