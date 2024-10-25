
<a id="0x1_stc_transaction_package_validation"></a>

# Module `0x1::stc_transaction_package_validation`

The module provides strategies for module upgrading.


-  [Resource `UpgradePlanCapability`](#0x1_stc_transaction_package_validation_UpgradePlanCapability)
-  [Struct `UpgradePlanV2`](#0x1_stc_transaction_package_validation_UpgradePlanV2)
-  [Resource `ModuleUpgradeStrategy`](#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy)
-  [Struct `TwoPhaseUpgradeConfig`](#0x1_stc_transaction_package_validation_TwoPhaseUpgradeConfig)
-  [Resource `TwoPhaseUpgradeV2`](#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2)
-  [Struct `UpgradeEvent`](#0x1_stc_transaction_package_validation_UpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `get_strategy_arbitrary`](#0x1_stc_transaction_package_validation_get_strategy_arbitrary)
-  [Function `get_strategy_two_phase`](#0x1_stc_transaction_package_validation_get_strategy_two_phase)
-  [Function `get_strategy_new_module`](#0x1_stc_transaction_package_validation_get_strategy_new_module)
-  [Function `get_strategy_freeze`](#0x1_stc_transaction_package_validation_get_strategy_freeze)
-  [Function `get_default_min_time_limit`](#0x1_stc_transaction_package_validation_get_default_min_time_limit)
-  [Function `update_module_upgrade_strategy`](#0x1_stc_transaction_package_validation_update_module_upgrade_strategy)
-  [Function `account_address`](#0x1_stc_transaction_package_validation_account_address)
-  [Function `destroy_upgrade_plan_cap`](#0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap)
-  [Function `extract_submit_upgrade_plan_cap`](#0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap)
-  [Function `submit_upgrade_plan_v2`](#0x1_stc_transaction_package_validation_submit_upgrade_plan_v2)
-  [Function `submit_upgrade_plan_with_cap_v2`](#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2)
-  [Function `cancel_upgrade_plan`](#0x1_stc_transaction_package_validation_cancel_upgrade_plan)
-  [Function `cancel_upgrade_plan_with_cap`](#0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap)
-  [Function `get_module_upgrade_strategy`](#0x1_stc_transaction_package_validation_get_module_upgrade_strategy)
-  [Function `get_upgrade_plan_v2`](#0x1_stc_transaction_package_validation_get_upgrade_plan_v2)
-  [Function `check_package_txn`](#0x1_stc_transaction_package_validation_check_package_txn)
-  [Function `check_package_txn_v2`](#0x1_stc_transaction_package_validation_check_package_txn_v2)
-  [Function `finish_upgrade_plan`](#0x1_stc_transaction_package_validation_finish_upgrade_plan)
-  [Function `package_txn_prologue_v2`](#0x1_stc_transaction_package_validation_package_txn_prologue_v2)
-  [Function `package_txn_epilogue`](#0x1_stc_transaction_package_validation_package_txn_epilogue)
-  [Specification](#@Specification_1)
    -  [Function `update_module_upgrade_strategy`](#@Specification_1_update_module_upgrade_strategy)
    -  [Function `destroy_upgrade_plan_cap`](#@Specification_1_destroy_upgrade_plan_cap)
    -  [Function `extract_submit_upgrade_plan_cap`](#@Specification_1_extract_submit_upgrade_plan_cap)
    -  [Function `submit_upgrade_plan_v2`](#@Specification_1_submit_upgrade_plan_v2)
    -  [Function `submit_upgrade_plan_with_cap_v2`](#@Specification_1_submit_upgrade_plan_with_cap_v2)
    -  [Function `cancel_upgrade_plan`](#@Specification_1_cancel_upgrade_plan)
    -  [Function `cancel_upgrade_plan_with_cap`](#@Specification_1_cancel_upgrade_plan_with_cap)
    -  [Function `get_module_upgrade_strategy`](#@Specification_1_get_module_upgrade_strategy)
    -  [Function `get_upgrade_plan_v2`](#@Specification_1_get_upgrade_plan_v2)
    -  [Function `check_package_txn`](#@Specification_1_check_package_txn)
    -  [Function `finish_upgrade_plan`](#@Specification_1_finish_upgrade_plan)
    -  [Function `package_txn_epilogue`](#@Specification_1_package_txn_epilogue)


<pre><code><b>use</b> <a href="account.md#0x1_account">0x1::account</a>;
<b>use</b> <a href="../../move-stdlib/doc/error.md#0x1_error">0x1::error</a>;
<b>use</b> <a href="event.md#0x1_event">0x1::event</a>;
<b>use</b> <a href="on_chain_config.md#0x1_on_chain_config">0x1::on_chain_config</a>;
<b>use</b> <a href="../../move-stdlib/doc/option.md#0x1_option">0x1::option</a>;
<b>use</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer">0x1::signer</a>;
<b>use</b> <a href="stc_version.md#0x1_stc_version">0x1::stc_version</a>;
<b>use</b> <a href="system_addresses.md#0x1_system_addresses">0x1::system_addresses</a>;
<b>use</b> <a href="timestamp.md#0x1_timestamp">0x1::timestamp</a>;
</code></pre>



<a id="0x1_stc_transaction_package_validation_UpgradePlanCapability"></a>

## Resource `UpgradePlanCapability`

The holder of UpgradePlanCapability for account_address can submit UpgradePlan for account_address.


<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: <b>address</b></code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_transaction_package_validation_UpgradePlanV2"></a>

## Struct `UpgradePlanV2`



<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>active_after_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="version.md#0x1_version">version</a>: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>enforced: bool</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_transaction_package_validation_ModuleUpgradeStrategy"></a>

## Resource `ModuleUpgradeStrategy`

module upgrade strategy


<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> <b>has</b> store, key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>strategy: u8</code>
</dt>
<dd>
 0 arbitrary
 1 two phase upgrade
 2 only new module
 3 freeze
</dd>
</dl>


</details>

<a id="0x1_stc_transaction_package_validation_TwoPhaseUpgradeConfig"></a>

## Struct `TwoPhaseUpgradeConfig`

config of two phase upgrade strategy.


<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeConfig">TwoPhaseUpgradeConfig</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>min_time_limit: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2"></a>

## Resource `TwoPhaseUpgradeV2`

data of two phase upgrade strategy.


<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeConfig">stc_transaction_package_validation::TwoPhaseUpgradeConfig</a></code>
</dt>
<dd>

</dd>
<dt>
<code>plan: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">stc_transaction_package_validation::UpgradePlanV2</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>version_cap: <a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>upgrade_event: <a href="event.md#0x1_event_EventHandle">event::EventHandle</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">stc_transaction_package_validation::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="0x1_stc_transaction_package_validation_UpgradeEvent"></a>

## Struct `UpgradeEvent`

module upgrade event.


<pre><code><b>struct</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">UpgradeEvent</a> <b>has</b> drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>package_address: <b>address</b></code>
</dt>
<dd>

</dd>
<dt>
<code>package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code><a href="version.md#0x1_version">version</a>: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a id="@Constants_0"></a>

## Constants


<a id="0x1_stc_transaction_package_validation_DEFAULT_MIN_TIME_LIMIT"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a>: u64 = 86400000;
</code></pre>



<a id="0x1_stc_transaction_package_validation_EACTIVE_TIME_INCORRECT"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>: u64 = 104;
</code></pre>



<a id="0x1_stc_transaction_package_validation_EPACKAGE_HASH_INCORRECT"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>: u64 = 103;
</code></pre>



<a id="0x1_stc_transaction_package_validation_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>: u64 = 109;
</code></pre>



<a id="0x1_stc_transaction_package_validation_ESTRATEGY_FREEZED"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>: u64 = 105;
</code></pre>



<a id="0x1_stc_transaction_package_validation_ESTRATEGY_INCORRECT"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_INCORRECT">ESTRATEGY_INCORRECT</a>: u64 = 106;
</code></pre>



<a id="0x1_stc_transaction_package_validation_ESTRATEGY_NOT_TWO_PHASE"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>: u64 = 107;
</code></pre>



<a id="0x1_stc_transaction_package_validation_EUNKNOWN_STRATEGY"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>: u64 = 108;
</code></pre>



<a id="0x1_stc_transaction_package_validation_EUPGRADE_PLAN_IS_NONE"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>: u64 = 102;
</code></pre>



<a id="0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>: u8 = 0;
</code></pre>



<a id="0x1_stc_transaction_package_validation_STRATEGY_FREEZE"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_FREEZE">STRATEGY_FREEZE</a>: u8 = 3;
</code></pre>



<a id="0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>: u8 = 2;
</code></pre>



<a id="0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE"></a>



<pre><code><b>const</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>: u8 = 1;
</code></pre>



<a id="0x1_stc_transaction_package_validation_get_strategy_arbitrary"></a>

## Function `get_strategy_arbitrary`

arbitary stragegy


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_arbitrary">get_strategy_arbitrary</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_arbitrary">get_strategy_arbitrary</a>(): u8 { <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a> }
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_strategy_two_phase"></a>

## Function `get_strategy_two_phase`

two phase stragegy


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_two_phase">get_strategy_two_phase</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_two_phase">get_strategy_two_phase</a>(): u8 { <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a> }
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_strategy_new_module"></a>

## Function `get_strategy_new_module`

new module strategy


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_new_module">get_strategy_new_module</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_new_module">get_strategy_new_module</a>(): u8 { <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a> }
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_strategy_freeze"></a>

## Function `get_strategy_freeze`

freezed strategy


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_freeze">get_strategy_freeze</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_strategy_freeze">get_strategy_freeze</a>(): u8 { <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_FREEZE">STRATEGY_FREEZE</a> }
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_default_min_time_limit"></a>

## Function `get_default_min_time_limit`

default min time limit


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_default_min_time_limit">get_default_min_time_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_default_min_time_limit">get_default_min_time_limit</a>(): u64 { <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a> }
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_update_module_upgrade_strategy"></a>

## Function `update_module_upgrade_strategy`

Update account's ModuleUpgradeStrategy


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, strategy: u8, min_time: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    strategy: u8,
    min_time: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> {
    <b>assert</b>!(
        strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a> || strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a> || strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a> || strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_FREEZE">STRATEGY_FREEZE</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>)
    );

    <b>let</b> account_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> previous_strategy = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address);
    <b>assert</b>!(strategy &gt; previous_strategy, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_INCORRECT">ESTRATEGY_INCORRECT</a>));
    <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address)) {
        <b>borrow_global_mut</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address).strategy = strategy;
    } <b>else</b> {
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> { strategy: strategy });
    };

    <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>) {
        <b>let</b> version_cap = <a href="on_chain_config.md#0x1_on_chain_config_extract_modify_config_capability">on_chain_config::extract_modify_config_capability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;(<a href="account.md#0x1_account">account</a>);
        <b>let</b> min_time_limit = <a href="../../move-stdlib/doc/option.md#0x1_option_get_with_default">option::get_with_default</a>(&min_time, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a>);
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> { account_address: account_address });
        <b>move_to</b>(<a href="account.md#0x1_account">account</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> {
            config: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeConfig">TwoPhaseUpgradeConfig</a> { min_time_limit: min_time_limit },
            plan: <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;(),
            version_cap: version_cap,
            upgrade_event: <a href="account.md#0x1_account_new_event_handle">account::new_event_handle</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">Self::UpgradeEvent</a>&gt;(<a href="account.md#0x1_account">account</a>)
        }
        );
    };

    //clean two phase upgrade resource
    <b>if</b> (previous_strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>) {
        // <b>if</b> (<b>exists</b>&lt;TwoPhaseUpgrade&gt;(account_address)) {
        //     <b>let</b> tpu = <b>move_from</b>&lt;TwoPhaseUpgrade&gt;(account_address);
        //     <b>let</b> TwoPhaseUpgrade { plan: _, version_cap, upgrade_event, config: _ } = tpu;
        //     <a href="event.md#0x1_event_destroy_handle">event::destroy_handle</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">Self::UpgradeEvent</a>&gt;(upgrade_event);
        //     <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">on_chain_config::destroy_modify_config_capability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;(version_cap);
        // };
        <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(account_address)) {
            <b>let</b> tpu = <b>move_from</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(account_address);
            <b>let</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> { plan: _, version_cap, upgrade_event, config: _ } = tpu;
            <a href="event.md#0x1_event_destroy_handle">event::destroy_handle</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">Self::UpgradeEvent</a>&gt;(upgrade_event);
            <a href="on_chain_config.md#0x1_on_chain_config_destroy_modify_config_capability">on_chain_config::destroy_modify_config_capability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;(version_cap);
        };
        // <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> may be extracted
        <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)) {
            <b>let</b> cap = <b>move_from</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
            <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap);
        };
    };
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_account_address"></a>

## Function `account_address`

Get account address of UpgradePlanCapability


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_account_address">account_address</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_account_address">account_address</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>): <b>address</b> {
    cap.account_address
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap"></a>

## Function `destroy_upgrade_plan_cap`

destroy the given UpgradePlanCapability


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>) {
    <b>let</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> { account_address: _ } = cap;
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap"></a>

## Function `extract_submit_upgrade_plan_cap`

extract out UpgradePlanCapability from <code><a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a></code>.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
): <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a> {
    <b>let</b> account_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>assert</b>!(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>)
    );
    <b>move_from</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_submit_upgrade_plan_v2"></a>

## Function `submit_upgrade_plan_v2`



<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="version.md#0x1_version">version</a>: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="version.md#0x1_version">version</a>: u64,
    enforced: bool
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> account_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap, package_hash, <a href="version.md#0x1_version">version</a>, enforced);
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2"></a>

## Function `submit_upgrade_plan_with_cap_v2`



<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="version.md#0x1_version">version</a>: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(
    cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;,
    <a href="version.md#0x1_version">version</a>: u64,
    enforced: bool
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> package_address = cap.account_address;
    <b>assert</b>!(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address) == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>)
    );
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>let</b> active_after_time = <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>() + tpu.config.min_time_limit;
    tpu.plan = <a href="../../move-stdlib/doc/option.md#0x1_option_some">option::some</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a> { package_hash, active_after_time, <a href="version.md#0x1_version">version</a>, enforced });
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_cancel_upgrade_plan"></a>

## Function `cancel_upgrade_plan`

Cancel a module upgrade plan.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan">cancel_upgrade_plan</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan">cancel_upgrade_plan</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> account_address = <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap);
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap"></a>

## Function `cancel_upgrade_plan_with_cap`

Cancel a module upgrade plan with given cap.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(
    cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> package_address = cap.account_address;
    <b>assert</b>!(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address) == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>,
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>)
    );
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&tpu.plan), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_state">error::invalid_state</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
    tpu.plan = <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;();
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_module_upgrade_strategy"></a>

## Function `get_module_upgrade_strategy`

Get module upgrade strategy of an module address.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8 <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
        <b>borrow_global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
    }<b>else</b> {
        0
    }
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_get_upgrade_plan_v2"></a>

## Function `get_upgrade_plan_v2`

Get module upgrade plan of an address.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">stc_transaction_package_validation::UpgradePlanV2</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt; <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address)) {
        *&<b>borrow_global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address).plan
    } <b>else</b> {
        <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;()
    }
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_check_package_txn"></a>

## Function `check_package_txn`

Check againest on the given package data.


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn">check_package_txn</a>(package_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn">check_package_txn</a>(
    package_address: <b>address</b>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> strategy = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>) {
        //do nothing
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>) {
        <b>let</b> plan_opt = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(package_address);
        <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&plan_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
        <b>let</b> plan = <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(&plan_opt);
        <b>assert</b>!(*&plan.package_hash == package_hash, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>));
        <b>assert</b>!(
            plan.active_after_time &lt;= <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>(),
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>)
        );
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>) {
        //do check at VM runtime.
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_FREEZE">STRATEGY_FREEZE</a>) {
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>);
    };
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_check_package_txn_v2"></a>

## Function `check_package_txn_v2`



<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn_v2">check_package_txn_v2</a>(txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn_v2">check_package_txn_v2</a>(
    txn_sender: <b>address</b>,
    package_address: <b>address</b>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>let</b> strategy = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>) {
        <b>assert</b>!(txn_sender == package_address, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>));
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>) {
        <b>let</b> plan_opt = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(package_address);
        <b>assert</b>!(<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&plan_opt), <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
        <b>let</b> plan = <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;(&plan_opt);
        <b>assert</b>!(*&plan.package_hash == package_hash, <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>));
        <b>assert</b>!(
            plan.active_after_time &lt;= <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>(),
            <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>)
        );
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>) {
        //do check at VM runtime.
        <b>assert</b>!(txn_sender == package_address, <a href="../../move-stdlib/doc/error.md#0x1_error_not_found">error::not_found</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>));
    }<b>else</b> <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_FREEZE">STRATEGY_FREEZE</a>) {
        <a href="../../move-stdlib/doc/error.md#0x1_error_invalid_argument">error::invalid_argument</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>);
    };
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_finish_upgrade_plan"></a>

## Function `finish_upgrade_plan`



<pre><code><b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> {
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>if</b> (<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(&tpu.plan)) {
        <b>let</b> plan = <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(&tpu.plan);
        <a href="on_chain_config.md#0x1_on_chain_config_set_with_capability">on_chain_config::set_with_capability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;(
            &<b>mut</b> tpu.version_cap,
            <a href="stc_version.md#0x1_stc_version_new_version">stc_version::new_version</a>(plan.<a href="version.md#0x1_version">version</a>)
        );
        <a href="event.md#0x1_event_emit_event">event::emit_event</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">Self::UpgradeEvent</a>&gt;(&<b>mut</b> tpu.upgrade_event, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradeEvent">UpgradeEvent</a> {
            package_address,
            package_hash: *&plan.package_hash,
            <a href="version.md#0x1_version">version</a>: plan.<a href="version.md#0x1_version">version</a>
        });
    };
    tpu.plan = <a href="../../move-stdlib/doc/option.md#0x1_option_none">option::none</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;();
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_package_txn_prologue_v2"></a>

## Function `package_txn_prologue_v2`



<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_prologue_v2">package_txn_prologue_v2</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_prologue_v2">package_txn_prologue_v2</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    txn_sender: <b>address</b>,
    package_address: <b>address</b>,
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by <a href="genesis.md#0x1_genesis">genesis</a> <a href="account.md#0x1_account">account</a>
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn_v2">check_package_txn_v2</a>(txn_sender, package_address, package_hash);
}
</code></pre>



</details>

<a id="0x1_stc_transaction_package_validation_package_txn_epilogue"></a>

## Function `package_txn_epilogue`

Package txn finished, and clean UpgradePlan


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_epilogue">package_txn_epilogue</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _txn_sender: <b>address</b>, package_address: <b>address</b>, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_epilogue">package_txn_epilogue</a>(
    <a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>,
    _txn_sender: <b>address</b>,
    package_address: <b>address</b>,
    success: bool
) <b>acquires</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by <a href="genesis.md#0x1_genesis">genesis</a> <a href="account.md#0x1_account">account</a>
    <a href="system_addresses.md#0x1_system_addresses_assert_starcoin_framework">system_addresses::assert_starcoin_framework</a>(<a href="account.md#0x1_account">account</a>);
    <b>let</b> strategy = <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>) {
        <b>if</b> (success) {
            <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_finish_upgrade_plan">finish_upgrade_plan</a>(package_address);
        };
    };
}
</code></pre>



</details>

<a id="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a id="@Specification_1_update_module_upgrade_strategy"></a>

### Function `update_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, strategy: u8, min_time: <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;u64&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> strategy != 0
    && strategy != 1
    && strategy != 2
    && strategy != 3;
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>))
    && strategy &lt;= <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).strategy;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)) && strategy == 0;
<b>aborts_if</b> strategy == 1 && <b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> strategy == 1 && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)
);
<b>let</b> holder = <b>global</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapabilityHolder">on_chain_config::ModifyConfigCapabilityHolder</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)
);
<b>aborts_if</b> strategy == 1 && <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>&lt;<a href="on_chain_config.md#0x1_on_chain_config_ModifyConfigCapability">on_chain_config::ModifyConfigCapability</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;&gt;(
    holder.cap
);
<b>aborts_if</b> strategy == 1 && <b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> <b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)) && <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(
    <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)
).strategy == 1
    && !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_1_destroy_upgrade_plan_cap"></a>

### Function `destroy_upgrade_plan_cap`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a id="@Specification_1_extract_submit_upgrade_plan_cap"></a>

### Function `extract_submit_upgrade_plan_cap`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>): <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>aborts_if</b> <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).strategy != 1;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
</code></pre>



<a id="@Specification_1_submit_upgrade_plan_v2"></a>

### Function `submit_upgrade_plan_v2`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="version.md#0x1_version">version</a>: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).account_address
};
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(
    <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).account_address).plan
);
</code></pre>



<a id="@Specification_1_submit_upgrade_plan_with_cap_v2"></a>

### Function `submit_upgrade_plan_with_cap_v2`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;, <a href="version.md#0x1_version">version</a>: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a> { <a href="account.md#0x1_account">account</a>: cap.account_address };
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(cap.account_address).plan);
</code></pre>




<a id="0x1_stc_transaction_package_validation_SubmitUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <b>aborts_if</b> <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="account.md#0x1_account">account</a>).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>aborts_if</b> <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>() + <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="account.md#0x1_account">account</a>).config.min_time_limit &gt; max_u64();
}
</code></pre>



<a id="@Specification_1_cancel_upgrade_plan"></a>

### Function `cancel_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan">cancel_upgrade_plan</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>));
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(
        <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)
    ).account_address
};
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(
    <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>)).account_address).plan
);
</code></pre>



<a id="@Specification_1_cancel_upgrade_plan_with_cap"></a>

### Function `cancel_upgrade_plan_with_cap`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanCapability">stc_transaction_package_validation::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a> { <a href="account.md#0x1_account">account</a>: cap.account_address };
<b>ensures</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(cap.account_address).plan);
</code></pre>




<a id="0x1_stc_transaction_package_validation_CancelUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a> {
    <a href="account.md#0x1_account">account</a>: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <b>aborts_if</b> <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="account.md#0x1_account">account</a>).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="account.md#0x1_account">account</a>);
    <b>aborts_if</b> !<a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(<a href="account.md#0x1_account">account</a>).plan);
}
</code></pre>



<a id="@Specification_1_get_module_upgrade_strategy"></a>

### Function `get_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<a id="0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy"></a>


<pre><code><b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8 {
   <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
   } <b>else</b> {
       0
   }
}
</code></pre>



<a id="@Specification_1_get_upgrade_plan_v2"></a>

### Function `get_upgrade_plan_v2`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">stc_transaction_package_validation::UpgradePlanV2</a>&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <b>false</b>;
</code></pre>




<a id="0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2"></a>


<pre><code><b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="../../move-stdlib/doc/option.md#0x1_option_Option">option::Option</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt; {
   <b>if</b> (<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address).plan
   }<b>else</b> {
       <a href="../../move-stdlib/doc/option.md#0x1_option_spec_none">option::spec_none</a>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_UpgradePlanV2">UpgradePlanV2</a>&gt;()
   }
}
</code></pre>



<a id="@Specification_1_check_package_txn"></a>

### Function `check_package_txn`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_check_package_txn">check_package_txn</a>(package_address: <b>address</b>, package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>include</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a>;
</code></pre>




<a id="0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIf"></a>


<pre><code><b>schema</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a> {
    package_address: <b>address</b>;
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 3;
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
        && <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address));
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
        && <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address)).package_hash != package_hash;
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
        && !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
        && <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address)).active_after_time &gt; <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>();
}
</code></pre>




<a id="0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIfWithType"></a>


<pre><code><b>schema</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_CheckPackageTxnAbortsIfWithType">CheckPackageTxnAbortsIfWithType</a> {
    is_package: bool;
    sender: <b>address</b>;
    package_address: <b>address</b>;
    package_hash: <a href="../../move-stdlib/doc/vector.md#0x1_vector">vector</a>&lt;u8&gt;;
    <b>aborts_if</b> is_package && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 3;
    <b>aborts_if</b> is_package && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="../../move-stdlib/doc/option.md#0x1_option_is_none">option::is_none</a>(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address)
    );
    <b>aborts_if</b> is_package && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address)
    ).package_hash != package_hash;
    <b>aborts_if</b> is_package && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(
        package_address
    ) == 1 && !<b>exists</b>&lt;<a href="timestamp.md#0x1_timestamp_CurrentTimeMicroseconds">timestamp::CurrentTimeMicroseconds</a>&gt;(<a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>());
    <b>aborts_if</b> is_package && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="../../move-stdlib/doc/option.md#0x1_option_borrow">option::borrow</a>(
        <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(package_address)
    ).active_after_time &gt; <a href="timestamp.md#0x1_timestamp_now_milliseconds">timestamp::now_milliseconds</a>();
}
</code></pre>



<a id="@Specification_1_finish_upgrade_plan"></a>

### Function `finish_upgrade_plan`


<pre><code><b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
<b>let</b> tpu = <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
<b>aborts_if</b> <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(tpu.plan) && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;&gt;(
    tpu.version_cap.account_address
);
</code></pre>




<a id="0x1_stc_transaction_package_validation_AbortsIfPackageTxnEpilogue"></a>


<pre><code><b>schema</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_AbortsIfPackageTxnEpilogue">AbortsIfPackageTxnEpilogue</a> {
    is_package: bool;
    package_address: <b>address</b>;
    success: bool;
    <b>aborts_if</b> is_package
        && <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address) == <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>
        && success
        && !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
}
</code></pre>



<a id="@Specification_1_package_txn_epilogue"></a>

### Function `package_txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_package_txn_epilogue">package_txn_epilogue</a>(<a href="account.md#0x1_account">account</a>: &<a href="../../move-stdlib/doc/signer.md#0x1_signer">signer</a>, _txn_sender: <b>address</b>, package_address: <b>address</b>, success: bool)
</code></pre>




<pre><code><b>aborts_if</b> <a href="../../move-stdlib/doc/signer.md#0x1_signer_address_of">signer::address_of</a>(<a href="account.md#0x1_account">account</a>) != <a href="system_addresses.md#0x1_system_addresses_get_starcoin_framework">system_addresses::get_starcoin_framework</a>();
<b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
    && success
    && !<b>exists</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
<b>aborts_if</b> <a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
    && success
    && <a href="../../move-stdlib/doc/option.md#0x1_option_is_some">option::is_some</a>(<b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address).plan)
    && !<b>exists</b>&lt;<a href="on_chain_config.md#0x1_on_chain_config_Config">on_chain_config::Config</a>&lt;<a href="stc_version.md#0x1_stc_version_Version">stc_version::Version</a>&gt;&gt;(
    <b>global</b>&lt;<a href="stc_transaction_package_validation.md#0x1_stc_transaction_package_validation_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address).version_cap.account_address
);
</code></pre>


[move-book]: https://starcoin.dev/move/book/SUMMARY
