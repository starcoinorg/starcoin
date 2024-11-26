
<a id="0x1_flexi_dag_config"></a>

# Module `0x1::flexi_dag_config`



-  [Struct `FlexiDagConfig`](#0x1_flexi_dag_config_FlexiDagConfig)
-  [Function `new_flexidag_config`](#0x1_flexi_dag_config_new_flexidag_config)
-  [Function `initialize`](#0x1_flexi_dag_config_initialize)
-  [Function `effective_height`](#0x1_flexi_dag_config_effective_height)
-  [Specification](#@Specification_0)
    -  [Function `initialize`](#@Specification_0_initialize)
    -  [Function `effective_height`](#@Specification_0_effective_height)


<pre><code><b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_flexi_dag_config_FlexiDagConfig"></a>

## Struct `FlexiDagConfig`

The struct to hold all config data needed for Flexidag.


<pre><code><b>struct</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>effective_height: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_flexi_dag_config_new_flexidag_config"></a>

## Function `new_flexidag_config`

Create a new configuration for flexidag, mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_new_flexidag_config">new_flexidag_config</a>(effective_height: u64): <a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">flexi_dag_config::FlexiDagConfig</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_new_flexidag_config">new_flexidag_config</a>(effective_height: u64): <a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a> {
    <a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a> {
        effective_height,
    }
}
</code></pre>



</details>

<a id="0x1_flexi_dag_config_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, effective_height: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, effective_height: u64) {
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;(<a href="account.md#0x1_account">account</a>, <a href="flexi_dag_config.md#0x1_flexi_dag_config_new_flexidag_config">new_flexidag_config</a>(effective_height));
}
</code></pre>



</details>

<a id="0x1_flexi_dag_config_effective_height"></a>

## Function `effective_height`



<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_effective_height">effective_height</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_effective_height">effective_height</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): u64 {
    <b>let</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config">flexi_dag_config</a> = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <a href="flexi_dag_config.md#0x1_flexi_dag_config">flexi_dag_config</a>.effective_height
}
</code></pre>



</details>

<a id="@Specification_0"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict;
</code></pre>



<a id="@Specification_0_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, effective_height: u64)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b> <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>ensures</b>
    <b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt;&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>),
    );
</code></pre>



<a id="@Specification_0_effective_height"></a>

### Function `effective_height`


<pre><code><b>public</b> <b>fun</b> <a href="flexi_dag_config.md#0x1_flexi_dag_config_effective_height">effective_height</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): u64
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="flexi_dag_config.md#0x1_flexi_dag_config_FlexiDagConfig">FlexiDagConfig</a>&gt; { addr: <a href="account.md#0x1_account">account</a> };
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
