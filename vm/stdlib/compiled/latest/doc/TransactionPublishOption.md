
<a name="0x1_TransactionPublishOption"></a>

# Module `0x1::TransactionPublishOption`

<code><a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a></code> provide an option to limit:
- whether user can use script or publish custom modules on chain.


-  [Struct `TransactionPublishOption`](#0x1_TransactionPublishOption_TransactionPublishOption)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x1_TransactionPublishOption_initialize)
-  [Function `new_transaction_publish_option`](#0x1_TransactionPublishOption_new_transaction_publish_option)
-  [Function `is_script_allowed`](#0x1_TransactionPublishOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_TransactionPublishOption_is_module_allowed)
-  [Specification](#@Specification_1)
    -  [Function `initialize`](#@Specification_1_initialize)
    -  [Function `new_transaction_publish_option`](#@Specification_1_new_transaction_publish_option)
    -  [Function `is_script_allowed`](#@Specification_1_is_script_allowed)
    -  [Function `is_module_allowed`](#@Specification_1_is_module_allowed)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
</code></pre>



<a name="0x1_TransactionPublishOption_TransactionPublishOption"></a>

## Struct `TransactionPublishOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1.  !script_allowed && !module_publishing_allowed No module publishing, only script function in module are allowed.
2.  script_allowed && !module_publishing_allowed No module publishing, custom scripts are allowed.
3.  script_allowed && module_publishing_allowed Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="@Constants_0"></a>

## Constants


<a name="0x1_TransactionPublishOption_EINVALID_ARGUMENT"></a>



<pre><code><b>const</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_EINVALID_ARGUMENT">EINVALID_ARGUMENT</a>: u64 = 18;
</code></pre>



<a name="0x1_TransactionPublishOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

The script hash already exists in the allowlist


<pre><code><b>const</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT">EALLOWLIST_ALREADY_CONTAINS_SCRIPT</a>: u64 = 1002;
</code></pre>



<a name="0x1_TransactionPublishOption_EINVALID_SCRIPT_HASH"></a>

The script hash has an invalid length


<pre><code><b>const</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_EINVALID_SCRIPT_HASH">EINVALID_SCRIPT_HASH</a>: u64 = 1001;
</code></pre>



<a name="0x1_TransactionPublishOption_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST"></a>



<pre><code><b>const</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>: u64 = 0;
</code></pre>



<a name="0x1_TransactionPublishOption_SCRIPT_HASH_LENGTH"></a>



<pre><code><b>const</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_SCRIPT_HASH_LENGTH">SCRIPT_HASH_LENGTH</a>: u64 = 32;
</code></pre>



<a name="0x1_TransactionPublishOption_initialize"></a>

## Function `initialize`

Module initialization.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">initialize</a>(account: &signer, script_allowed: bool, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">initialize</a>(
    account: &signer,
    script_allowed: bool,
    module_publishing_allowed: bool,
) {
    <a href="Timestamp.md#0x1_Timestamp_assert_genesis">Timestamp::assert_genesis</a>();
    <b>assert</b>!(
        <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(),
        <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="TransactionPublishOption.md#0x1_TransactionPublishOption_EPROLOGUE_ACCOUNT_DOES_NOT_EXIST">EPROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>),
    );
    <b>let</b> transaction_publish_option = <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">Self::new_transaction_publish_option</a>(script_allowed, module_publishing_allowed);
    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>(
        account,
        transaction_publish_option,
    );
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_new_transaction_publish_option"></a>

## Function `new_transaction_publish_option`

Create a new option. Mainly used in DAO.


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">new_transaction_publish_option</a>(script_allowed: bool, module_publishing_allowed: bool): <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">new_transaction_publish_option</a>(
    script_allowed: bool,
    module_publishing_allowed: bool,
): <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a> {
    <a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a> { script_allowed, module_publishing_allowed }
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_is_script_allowed"></a>

## Function `is_script_allowed`

Check if sender can execute script with


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_script_allowed">is_script_allowed</a>(account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_script_allowed">is_script_allowed</a>(account: <b>address</b>): bool {
    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);
    publish_option.script_allowed
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_is_module_allowed"></a>

## Function `is_module_allowed`

Check if a sender can publish a module


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_module_allowed">is_module_allowed</a>(account: <b>address</b>): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_module_allowed">is_module_allowed</a>(account: <b>address</b>): bool {
    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);
    publish_option.module_publishing_allowed
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>




<a name="0x1_TransactionPublishOption_spec_is_script_allowed"></a>


<pre><code><b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_spec_is_script_allowed">spec_is_script_allowed</a>(addr: <b>address</b>) : bool{
   <b>let</b> publish_option = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(addr);
   publish_option.script_allowed
}
</code></pre>




<a name="0x1_TransactionPublishOption_spec_is_module_allowed"></a>


<pre><code><b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_spec_is_module_allowed">spec_is_module_allowed</a>(addr: <b>address</b>) : bool{
   <b>let</b> publish_option = <a href="Config.md#0x1_Config_get_by_address">Config::get_by_address</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(addr);
   publish_option.module_publishing_allowed
}
</code></pre>



<a name="@Specification_1_initialize"></a>

### Function `initialize`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_initialize">initialize</a>(account: &signer, script_allowed: bool, module_publishing_allowed: bool)
</code></pre>




<pre><code><b>aborts_if</b> !<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>();
<b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigAbortsIf">Config::PublishNewConfigAbortsIf</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;;
<b>include</b> <a href="Config.md#0x1_Config_PublishNewConfigEnsures">Config::PublishNewConfigEnsures</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;;
</code></pre>



<a name="@Specification_1_new_transaction_publish_option"></a>

### Function `new_transaction_publish_option`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_new_transaction_publish_option">new_transaction_publish_option</a>(script_allowed: bool, module_publishing_allowed: bool): <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_TransactionPublishOption">TransactionPublishOption::TransactionPublishOption</a>
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_is_script_allowed"></a>

### Function `is_script_allowed`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_script_allowed">is_script_allowed</a>(account: <b>address</b>): bool
</code></pre>




<pre><code><b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;{
    addr: account
};
</code></pre>



<a name="@Specification_1_is_module_allowed"></a>

### Function `is_module_allowed`


<pre><code><b>public</b> <b>fun</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_is_module_allowed">is_module_allowed</a>(account: <b>address</b>): bool
</code></pre>




<pre><code><b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;{
    addr: account
};
</code></pre>




<a name="0x1_TransactionPublishOption_AbortsIfTxnPublishOptionNotExist"></a>


<pre><code><b>schema</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_AbortsIfTxnPublishOptionNotExist">AbortsIfTxnPublishOptionNotExist</a> {
    <b>include</b> <a href="Config.md#0x1_Config_AbortsIfConfigNotExist">Config::AbortsIfConfigNotExist</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;{
        addr: <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>()
    };
}
</code></pre>




<a name="0x1_TransactionPublishOption_AbortsIfTxnPublishOptionNotExistWithBool"></a>


<pre><code><b>schema</b> <a href="TransactionPublishOption.md#0x1_TransactionPublishOption_AbortsIfTxnPublishOptionNotExistWithBool">AbortsIfTxnPublishOptionNotExistWithBool</a> {
    is_script_or_package : bool;
    <b>aborts_if</b> is_script_or_package && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="TransactionPublishOption.md#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
}
</code></pre>
