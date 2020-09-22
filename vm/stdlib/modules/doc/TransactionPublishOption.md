
<a name="0x1_TransactionPublishOption"></a>

# Module `0x1::TransactionPublishOption`

### Table of Contents

-  [Struct `TransactionPublishOption`](#0x1_TransactionPublishOption_TransactionPublishOption)
-  [Const `SCRIPT_HASH_LENGTH`](#0x1_TransactionPublishOption_SCRIPT_HASH_LENGTH)
-  [Const `EINVALID_SCRIPT_HASH`](#0x1_TransactionPublishOption_EINVALID_SCRIPT_HASH)
-  [Const `EALLOWLIST_ALREADY_CONTAINS_SCRIPT`](#0x1_TransactionPublishOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT)
-  [Function `initialize`](#0x1_TransactionPublishOption_initialize)
-  [Function `is_script_allowed`](#0x1_TransactionPublishOption_is_script_allowed)
-  [Function `is_module_allowed`](#0x1_TransactionPublishOption_is_module_allowed)
-  [Function `add_to_script_allow_list`](#0x1_TransactionPublishOption_add_to_script_allow_list)
-  [Function `set_open_script`](#0x1_TransactionPublishOption_set_open_script)
-  [Function `set_open_module`](#0x1_TransactionPublishOption_set_open_module)



<a name="0x1_TransactionPublishOption_TransactionPublishOption"></a>

## Struct `TransactionPublishOption`

Defines and holds the publishing policies for the VM. There are three possible configurations:
1. No module publishing, only allowlisted scripts are allowed.
2. No module publishing, custom scripts are allowed.
3. Both module publishing and custom scripts are allowed.
We represent these as the following resource.


<pre><code><b>struct</b> <a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>script_allow_list: vector&lt;vector&lt;u8&gt;&gt;</code>
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

<a name="0x1_TransactionPublishOption_SCRIPT_HASH_LENGTH"></a>

## Const `SCRIPT_HASH_LENGTH`



<pre><code><b>const</b> SCRIPT_HASH_LENGTH: u64 = 32;
</code></pre>



<a name="0x1_TransactionPublishOption_EINVALID_SCRIPT_HASH"></a>

## Const `EINVALID_SCRIPT_HASH`

The script hash has an invalid length


<pre><code><b>const</b> EINVALID_SCRIPT_HASH: u64 = 1001;
</code></pre>



<a name="0x1_TransactionPublishOption_EALLOWLIST_ALREADY_CONTAINS_SCRIPT"></a>

## Const `EALLOWLIST_ALREADY_CONTAINS_SCRIPT`

The script hash already exists in the allowlist


<pre><code><b>const</b> EALLOWLIST_ALREADY_CONTAINS_SCRIPT: u64 = 1002;
</code></pre>



<a name="0x1_TransactionPublishOption_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_initialize">initialize</a>(account: &signer, script_allow_list: vector&lt;vector&lt;u8&gt;&gt;, module_publishing_allowed: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_initialize">initialize</a>(
    account: &signer,
    script_allow_list: vector&lt;vector&lt;u8&gt;&gt;,
    module_publishing_allowed: bool,
) {
    <b>assert</b>(<a href="Timestamp.md#0x1_Timestamp_is_genesis">Timestamp::is_genesis</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS">ErrorCode::ENOT_GENESIS</a>());
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    <a href="Config.md#0x1_Config_publish_new_config">Config::publish_new_config</a>(
        account,
        <a href="#0x1_TransactionPublishOption">TransactionPublishOption</a> {
            script_allow_list,
            module_publishing_allowed
        }
    );
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_is_script_allowed"></a>

## Function `is_script_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_is_script_allowed">is_script_allowed</a>(account: &signer, hash: &vector&lt;u8&gt;): bool {
    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);

    <a href="Vector.md#0x1_Vector_is_empty">Vector::is_empty</a>(&publish_option.script_allow_list)
        || <a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, hash)
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_is_module_allowed"></a>

## Function `is_module_allowed`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_is_module_allowed">is_module_allowed</a>(account: &signer): bool {
    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);

    publish_option.module_publishing_allowed
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_add_to_script_allow_list"></a>

## Function `add_to_script_allow_list`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_add_to_script_allow_list">add_to_script_allow_list</a>(account: &signer, new_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_add_to_script_allow_list">add_to_script_allow_list</a>(account: &signer, new_hash: vector&lt;u8&gt;) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());
    <b>assert</b>(<a href="Vector.md#0x1_Vector_length">Vector::length</a>(&new_hash) == SCRIPT_HASH_LENGTH, <a href="ErrorCode.md#0x1_ErrorCode_EINVALID_ARGUMENT">ErrorCode::EINVALID_ARGUMENT</a>());

    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);
    <b>if</b> (<a href="Vector.md#0x1_Vector_contains">Vector::contains</a>(&publish_option.script_allow_list, &new_hash)) {
        <b>abort</b> EALLOWLIST_ALREADY_CONTAINS_SCRIPT
    };
    <a href="Vector.md#0x1_Vector_push_back">Vector::push_back</a>(&<b>mut</b> publish_option.script_allow_list, new_hash);

    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account, publish_option);
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_set_open_script"></a>

## Function `set_open_script`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_set_open_script">set_open_script</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_set_open_script">set_open_script</a>(account: &signer) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);

    publish_option.script_allow_list = <a href="Vector.md#0x1_Vector_empty">Vector::empty</a>();
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account, publish_option);
}
</code></pre>



</details>

<a name="0x1_TransactionPublishOption_set_open_module"></a>

## Function `set_open_module`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_set_open_module">set_open_module</a>(account: &signer, open_module: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_TransactionPublishOption_set_open_module">set_open_module</a>(account: &signer, open_module: bool) {
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>(), <a href="ErrorCode.md#0x1_ErrorCode_PROLOGUE_ACCOUNT_DOES_NOT_EXIST">ErrorCode::PROLOGUE_ACCOUNT_DOES_NOT_EXIST</a>());

    <b>let</b> publish_option = <a href="Config.md#0x1_Config_get">Config::get</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account);

    publish_option.module_publishing_allowed = open_module;
    <a href="Config.md#0x1_Config_set">Config::set</a>&lt;<a href="#0x1_TransactionPublishOption">TransactionPublishOption</a>&gt;(account, publish_option);
}
</code></pre>



</details>
