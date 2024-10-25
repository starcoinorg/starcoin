
<a id="0x1_transaction_publish_option"></a>

# Module `0x1::transaction_publish_option`

<code><a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a></code> provide an option to limit:
- whether user can use script or publish custom modules on chain.


-  [Struct `TransactionPublishOption`](#0x1_transaction_publish_option_TransactionPublishOption)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_transaction_publish_option_initialize)
-  [Function `new_transaction_publish_option`](#0x1_transaction_publish_option_new_transaction_publish_option)
-  [Function `is_script_allowed`](#0x1_transaction_publish_option_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_transaction_publish_option_is_module_allowed)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `new_transaction_publish_option`](#@Specification_1_new_transaction_publish_option)
    -  [Function `is_script_allowed`](#@Specification_1_is_script_allowed)
    -  [Function `is_module_allowed`](#@Specification_1_is_module_allowed)


<pre><code><b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
</code></pre>



<a id="0x1_transaction_publish_option_TransactionPublishOption"></a>

## Struct `TransactionPublishOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1.  !script_allowed && !module_publishing_allowed No module publishing, only script function in module are allowed.
2.  script_allowed && !module_publishing_allowed No module publishing, custom scripts are allowed.
3.  script_allowed && module_publishing_allowed Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>script_allowed: bool</code>
</dt>
<dd>

</dd>
<dt>
<code>module_publishing_allowed: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_transaction_publish_option_EINVALID_ARGUMENT"></a>



<pre><code><b>const</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>: u64 = 18;
</code></pre>



<a id="0x1_transaction_publish_option_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1002;
</code></pre>



<a id="0x1_transaction_publish_option_EINVALID_SCRIPT_HASH"></a>

The script hash has an invalid length


<pre><code><b>const</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 1001;
</code></pre>



<a id="0x1_transaction_publish_option_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>: u64 = 0;
</code></pre>



<a id="0x1_transaction_publish_option_SCRIPT_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a id="0x1_transaction_publish_option_initialize"></a>

## Function `initialize`

Module initialization.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, script_allowed: bool, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_initialize">initialize</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    script_allowed: bool,
    module_publishing_allowed: bool,
) {
    // timestamp::assert_genesis();
    <b>assert</b>!(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) == <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>(),
        <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>),
    );
    <b>let</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option">transaction_publish_option</a> = <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_new_transaction_publish_option">Self::new_transaction_publish_option</a>(
        script_allowed,
        module_publishing_allowed
    );
    <a href="on_chain_config.md#0x1_on_chain_config_publish_new_config">on_chain_config::publish_new_config</a>(
        <a href="account.md#0x1_account">account</a>,
        <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option">transaction_publish_option</a>,
    );
}
</code></pre>



</details>

<a id="0x1_transaction_publish_option_new_transaction_publish_option"></a>

## Function `new_transaction_publish_option`

Create a new option. Mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_new_transaction_publish_option">new_transaction_publish_option</a>(script_allowed: bool, module_publishing_allowed: bool): <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">transaction_publish_option::TransactionPublishOption</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_new_transaction_publish_option">new_transaction_publish_option</a>(
    script_allowed: bool,
    module_publishing_allowed: bool,
): <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a> {
    <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a> { script_allowed, module_publishing_allowed }
}
</code></pre>



</details>

<a id="0x1_transaction_publish_option_is_script_allowed"></a>

## Function `is_script_allowed`

Check if sender can execute script with


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_script_allowed">is_script_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_script_allowed">is_script_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool {
    <b>let</b> publish_option = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;(<a href="account.md#0x1_account">account</a>);
    publish_option.script_allowed
}
</code></pre>



</details>

<a id="0x1_transaction_publish_option_is_module_allowed"></a>

## Function `is_module_allowed`

Check if a sender can publish a module


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_module_allowed">is_module_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_module_allowed">is_module_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool {
    <b>let</b> publish_option = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;(<a href="account.md#0x1_account">account</a>);
    publish_option.module_publishing_allowed
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>




<a id="0x1_transaction_publish_option_spec_is_script_allowed"></a>


<pre><code><b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_spec_is_script_allowed">spec_is_script_allowed</a>(addr: <b>address</b>): bool {
   <b>let</b> publish_option = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;(addr);
   publish_option.script_allowed
}
</code></pre>




<a id="0x1_transaction_publish_option_spec_is_module_allowed"></a>


<pre><code><b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_spec_is_module_allowed">spec_is_module_allowed</a>(addr: <b>address</b>): bool {
   <b>let</b> publish_option = <a href="on_chain_config.md#0x1_on_chain_config_get_by_address">on_chain_config::get_by_address</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;(addr);
   publish_option.module_publishing_allowed
}
</code></pre>



<a id="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_initialize">initialize</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, script_allowed: bool, module_publishing_allowed: bool)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigAbortsIf">on_chain_config::PublishNewConfigAbortsIf</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;;
<b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_PublishNewConfigEnsures">on_chain_config::PublishNewConfigEnsures</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;;
</code></pre>



<a id="@Specification_1_new_transaction_publish_option"></a>

### Function `new_transaction_publish_option`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_new_transaction_publish_option">new_transaction_publish_option</a>(script_allowed: bool, module_publishing_allowed: bool): <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">transaction_publish_option::TransactionPublishOption</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_is_script_allowed"></a>

### Function `is_script_allowed`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_script_allowed">is_script_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt; {
    addr: <a href="account.md#0x1_account">account</a>
};
</code></pre>



<a id="@Specification_1_is_module_allowed"></a>

### Function `is_module_allowed`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_is_module_allowed">is_module_allowed</a>(<a href="account.md#0x1_account">account</a>: <b>address</b>): bool
</code></pre>




<pre><code><b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt; {
    addr: <a href="account.md#0x1_account">account</a>
};
</code></pre>




<a id="0x1_transaction_publish_option_AbortsIfTxnPublishOptionNotExist"></a>


<pre><code><b>schema</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_AbortsIfTxnPublishOptionNotExist">AbortsIfTxnPublishOptionNotExist</a> {
    <b>include</b> <a href="on_chain_config.md#0x1_on_chain_config_AbortsIfConfigNotExist">on_chain_config::AbortsIfConfigNotExist</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt; {
        addr: <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    };
}
</code></pre>




<a id="0x1_transaction_publish_option_AbortsIfTxnPublishOptionNotExistWithBool"></a>


<pre><code><b>schema</b> <a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_AbortsIfTxnPublishOptionNotExistWithBool">AbortsIfTxnPublishOptionNotExistWithBool</a> {
    is_script_or_package: bool;
    <b>aborts_if</b> is_script_or_package && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_transaction_publish_option.md#0x1_transaction_publish_option_TransactionPublishOption">TransactionPublishOption</a>&gt;&gt;(
        <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>()
    );
}
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
