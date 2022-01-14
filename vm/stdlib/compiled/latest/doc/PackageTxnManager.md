
<a name="0x1_PackageTxnManager"></a>

# Module `0x1::PackageTxnManager`

The module provides strategies for module upgrading.


-  [Struct `UpgradePlan`](#0x1_PackageTxnManager_UpgradePlan)
-  [Resource `UpgradePlanCapability`](#0x1_PackageTxnManager_UpgradePlanCapability)
-  [Struct `UpgradePlanV2`](#0x1_PackageTxnManager_UpgradePlanV2)
-  [Resource `ModuleUpgradeStrategy`](#0x1_PackageTxnManager_ModuleUpgradeStrategy)
-  [Resource `TwoPhaseUpgrade`](#0x1_PackageTxnManager_TwoPhaseUpgrade)
-  [Struct `TwoPhaseUpgradeConfig`](#0x1_PackageTxnManager_TwoPhaseUpgradeConfig)
-  [Resource `TwoPhaseUpgradeV2`](#0x1_PackageTxnManager_TwoPhaseUpgradeV2)
-  [Struct `UpgradeEvent`](#0x1_PackageTxnManager_UpgradeEvent)
-  [Constants](#@Constants_0)
-  [Function `get_strategy_arbitrary`](#0x1_PackageTxnManager_get_strategy_arbitrary)
-  [Function `get_strategy_two_phase`](#0x1_PackageTxnManager_get_strategy_two_phase)
-  [Function `get_strategy_new_module`](#0x1_PackageTxnManager_get_strategy_new_module)
-  [Function `get_strategy_freeze`](#0x1_PackageTxnManager_get_strategy_freeze)
-  [Function `get_default_min_time_limit`](#0x1_PackageTxnManager_get_default_min_time_limit)
-  [Function `update_module_upgrade_strategy`](#0x1_PackageTxnManager_update_module_upgrade_strategy)
-  [Function `account_address`](#0x1_PackageTxnManager_account_address)
-  [Function `destroy_upgrade_plan_cap`](#0x1_PackageTxnManager_destroy_upgrade_plan_cap)
-  [Function `extract_submit_upgrade_plan_cap`](#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap)
-  [Function `convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2`](#0x1_PackageTxnManager_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2)
-  [Function `submit_upgrade_plan_v2`](#0x1_PackageTxnManager_submit_upgrade_plan_v2)
-  [Function `submit_upgrade_plan_with_cap_v2`](#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2)
-  [Function `cancel_upgrade_plan`](#0x1_PackageTxnManager_cancel_upgrade_plan)
-  [Function `cancel_upgrade_plan_with_cap`](#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap)
-  [Function `get_module_upgrade_strategy`](#0x1_PackageTxnManager_get_module_upgrade_strategy)
-  [Function `get_upgrade_plan`](#0x1_PackageTxnManager_get_upgrade_plan)
-  [Function `get_upgrade_plan_v2`](#0x1_PackageTxnManager_get_upgrade_plan_v2)
-  [Function `check_package_txn`](#0x1_PackageTxnManager_check_package_txn)
-  [Function `check_package_txn_v2`](#0x1_PackageTxnManager_check_package_txn_v2)
-  [Function `finish_upgrade_plan`](#0x1_PackageTxnManager_finish_upgrade_plan)
-  [Function `package_txn_prologue`](#0x1_PackageTxnManager_package_txn_prologue)
-  [Function `package_txn_prologue_v2`](#0x1_PackageTxnManager_package_txn_prologue_v2)
-  [Function `package_txn_epilogue`](#0x1_PackageTxnManager_package_txn_epilogue)
-  [Specification](#@Specification_1)
    -  [Function `update_module_upgrade_strategy`](#@Specification_1_update_module_upgrade_strategy)
    -  [Function `destroy_upgrade_plan_cap`](#@Specification_1_destroy_upgrade_plan_cap)
    -  [Function `extract_submit_upgrade_plan_cap`](#@Specification_1_extract_submit_upgrade_plan_cap)
    -  [Function `convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2`](#@Specification_1_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2)
    -  [Function `submit_upgrade_plan_v2`](#@Specification_1_submit_upgrade_plan_v2)
    -  [Function `submit_upgrade_plan_with_cap_v2`](#@Specification_1_submit_upgrade_plan_with_cap_v2)
    -  [Function `cancel_upgrade_plan`](#@Specification_1_cancel_upgrade_plan)
    -  [Function `cancel_upgrade_plan_with_cap`](#@Specification_1_cancel_upgrade_plan_with_cap)
    -  [Function `get_module_upgrade_strategy`](#@Specification_1_get_module_upgrade_strategy)
    -  [Function `get_upgrade_plan`](#@Specification_1_get_upgrade_plan)
    -  [Function `get_upgrade_plan_v2`](#@Specification_1_get_upgrade_plan_v2)
    -  [Function `check_package_txn`](#@Specification_1_check_package_txn)
    -  [Function `finish_upgrade_plan`](#@Specification_1_finish_upgrade_plan)
    -  [Function `package_txn_prologue`](#@Specification_1_package_txn_prologue)
    -  [Function `package_txn_epilogue`](#@Specification_1_package_txn_epilogue)


<pre><code><b>use</b> <a href="Config.md#0x1_Config">0x1::Config</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Event.md#0x1_Event">0x1::Event</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
<b>use</b> <a href="Timestamp.md#0x1_Timestamp">0x1::Timestamp</a>;
<b>use</b> <a href="Version.md#0x1_Version">0x1::Version</a>;
</code></pre>



<a name="0x1_PackageTxnManager_UpgradePlan"></a>

## Struct `UpgradePlan`

module upgrade plan


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>package_hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>active_after_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>version: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_UpgradePlanCapability"></a>

## Resource `UpgradePlanCapability`

The holder of UpgradePlanCapability for account_address can submit UpgradePlan for account_address.


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> <b>has</b> store, key
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

<a name="0x1_PackageTxnManager_UpgradePlanV2"></a>

## Struct `UpgradePlanV2`



<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a> <b>has</b> <b>copy</b>, drop, store
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>package_hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>active_after_time: u64</code>
</dt>
<dd>

</dd>
<dt>
<code>version: u64</code>
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

<a name="0x1_PackageTxnManager_ModuleUpgradeStrategy"></a>

## Resource `ModuleUpgradeStrategy`

module upgrade strategy


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> <b>has</b> store, key
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

<a name="0x1_PackageTxnManager_TwoPhaseUpgrade"></a>

## Resource `TwoPhaseUpgrade`

data of two phase upgrade strategy.


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeConfig">PackageTxnManager::TwoPhaseUpgradeConfig</a></code>
</dt>
<dd>

</dd>
<dt>
<code>plan: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>version_cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>upgrade_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">PackageTxnManager::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_TwoPhaseUpgradeConfig"></a>

## Struct `TwoPhaseUpgradeConfig`

config of two phase upgrade strategy.


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeConfig">TwoPhaseUpgradeConfig</a> <b>has</b> <b>copy</b>, drop, store
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

<a name="0x1_PackageTxnManager_TwoPhaseUpgradeV2"></a>

## Resource `TwoPhaseUpgradeV2`

data of two phase upgrade strategy.


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> <b>has</b> key
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>config: <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeConfig">PackageTxnManager::TwoPhaseUpgradeConfig</a></code>
</dt>
<dd>

</dd>
<dt>
<code>plan: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">PackageTxnManager::UpgradePlanV2</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>version_cap: <a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>upgrade_event: <a href="Event.md#0x1_Event_EventHandle">Event::EventHandle</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">PackageTxnManager::UpgradeEvent</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_UpgradeEvent"></a>

## Struct `UpgradeEvent`

module upgrade event.


<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">UpgradeEvent</a> <b>has</b> drop, store
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
<code>package_hash: vector&lt;u8&gt;</code>
</dt>
<dd>

</dd>
<dt>
<code>version: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_PackageTxnManager_DEFAULT_MIN_TIME_LIMIT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a>: u64 = 86400000;
</code></pre>



<a name="0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>: u64 = 104;
</code></pre>



<a name="0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>: u64 = 103;
</code></pre>



<a name="0x1_PackageTxnManager_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>: u64 = 109;
</code></pre>



<a name="0x1_PackageTxnManager_ESTRATEGY_FREEZED"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>: u64 = 105;
</code></pre>



<a name="0x1_PackageTxnManager_ESTRATEGY_INCORRECT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_INCORRECT">ESTRATEGY_INCORRECT</a>: u64 = 106;
</code></pre>



<a name="0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>: u64 = 107;
</code></pre>



<a name="0x1_PackageTxnManager_EUNKNOWN_STRATEGY"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>: u64 = 108;
</code></pre>



<a name="0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>: u64 = 102;
</code></pre>



<a name="0x1_PackageTxnManager_STRATEGY_ARBITRARY"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>: u8 = 0;
</code></pre>



<a name="0x1_PackageTxnManager_STRATEGY_FREEZE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>: u8 = 3;
</code></pre>



<a name="0x1_PackageTxnManager_STRATEGY_NEW_MODULE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>: u8 = 2;
</code></pre>



<a name="0x1_PackageTxnManager_STRATEGY_TWO_PHASE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>: u8 = 1;
</code></pre>



<a name="0x1_PackageTxnManager_get_strategy_arbitrary"></a>

## Function `get_strategy_arbitrary`

arbitary stragegy


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_arbitrary">get_strategy_arbitrary</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_arbitrary">get_strategy_arbitrary</a>(): u8 { <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a> }
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_strategy_two_phase"></a>

## Function `get_strategy_two_phase`

two phase stragegy


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">get_strategy_two_phase</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_two_phase">get_strategy_two_phase</a>(): u8 { <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a> }
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_strategy_new_module"></a>

## Function `get_strategy_new_module`

new module strategy


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_new_module">get_strategy_new_module</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_new_module">get_strategy_new_module</a>(): u8 { <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a> }
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_strategy_freeze"></a>

## Function `get_strategy_freeze`

freezed strategy


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_freeze">get_strategy_freeze</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_strategy_freeze">get_strategy_freeze</a>(): u8 { <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a> }
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_default_min_time_limit"></a>

## Function `get_default_min_time_limit`

default min time limit


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_default_min_time_limit">get_default_min_time_limit</a>(): u64
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_default_min_time_limit">get_default_min_time_limit</a>(): u64 { <a href="PackageTxnManager.md#0x1_PackageTxnManager_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a> }
</code></pre>



</details>

<a name="0x1_PackageTxnManager_update_module_upgrade_strategy"></a>

## Function `update_module_upgrade_strategy`

Update account's ModuleUpgradeStrategy


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8, min_time: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8, min_time: <a href="Option.md#0x1_Option">Option</a>&lt;u64&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>assert</b>!(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a> || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a> || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a> || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>));
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> previous_strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address);
    <b>assert</b>!(strategy &gt; previous_strategy, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_INCORRECT">ESTRATEGY_INCORRECT</a>));
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address)) {
        <b>borrow_global_mut</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address).strategy = strategy;
    }<b>else</b>{
        <b>move_to</b>(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{ strategy: strategy});
    };
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>){
        <b>let</b> version_cap = <a href="Config.md#0x1_Config_extract_modify_config_capability">Config::extract_modify_config_capability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(account);
        <b>let</b> min_time_limit = <a href="Option.md#0x1_Option_get_with_default">Option::get_with_default</a>(&min_time, <a href="PackageTxnManager.md#0x1_PackageTxnManager_DEFAULT_MIN_TIME_LIMIT">DEFAULT_MIN_TIME_LIMIT</a>);
        <b>move_to</b>(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{ account_address: account_address});
        <b>move_to</b>(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>{
            config: <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeConfig">TwoPhaseUpgradeConfig</a>{min_time_limit: min_time_limit},
            plan: <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt;(),
            version_cap: version_cap,
            upgrade_event: <a href="Event.md#0x1_Event_new_event_handle">Event::new_event_handle</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(account)}
        );
    };
    //clean two phase upgrade resource
    <b>if</b> (previous_strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>){
        <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address)) {
            <b>let</b> tpu = <b>move_from</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
            <b>let</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{plan:_, version_cap, upgrade_event, config: _} = tpu;
            <a href="Event.md#0x1_Event_destroy_handle">Event::destroy_handle</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(upgrade_event);
            <a href="Config.md#0x1_Config_destroy_modify_config_capability">Config::destroy_modify_config_capability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(version_cap);
        };
        <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(account_address)) {
            <b>let</b> tpu = <b>move_from</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(account_address);
            <b>let</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>{plan:_, version_cap, upgrade_event, config: _} = tpu;
            <a href="Event.md#0x1_Event_destroy_handle">Event::destroy_handle</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(upgrade_event);
            <a href="Config.md#0x1_Config_destroy_modify_config_capability">Config::destroy_modify_config_capability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(version_cap);
        };
        // <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> may be extracted
        <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)) {
            <b>let</b> cap = <b>move_from</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
            <a href="PackageTxnManager.md#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap);
        };
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_account_address"></a>

## Function `account_address`

Get account address of UpgradePlanCapability


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">account_address</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>): <b>address</b>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">account_address</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>): <b>address</b> {
    cap.account_address
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_destroy_upgrade_plan_cap"></a>

## Function `destroy_upgrade_plan_cap`

destroy the given UpgradePlanCapability


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>){
    <b>let</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{account_address:_} = cap;
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_extract_submit_upgrade_plan_cap"></a>

## Function `extract_submit_upgrade_plan_cap`

extract out UpgradePlanCapability from <code>signer</code>.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>!(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    <b>move_from</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2"></a>

## Function `convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2`



<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2">convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2</a>(account: signer, package_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2">convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2</a>(account: signer, package_address: <b>address</b>) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> {
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(&account);
    // sender should be package owner
    <b>assert</b>!(account_address == package_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>));
    <b>let</b> tpu = <b>move_from</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
    <b>let</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{config, plan, version_cap, upgrade_event} = tpu;
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&plan)) {
        <b>let</b> old_plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&plan);
        <b>move_to</b>(&account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>{
            config: config,
            plan: <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a> {
                package_hash: *&old_plan.package_hash,
                active_after_time: old_plan.active_after_time,
                version: old_plan.version,
                enforced: <b>false</b> }),
            version_cap: version_cap,
            upgrade_event: upgrade_event
        });
    } <b>else</b> {
        <b>move_to</b>(&account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>{
            config: config,
            plan: <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt;(),
            version_cap: version_cap,
            upgrade_event: upgrade_event
        });
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan_v2"></a>

## Function `submit_upgrade_plan_v2`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(account: &signer, package_hash: vector&lt;u8&gt;, version: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(account: &signer, package_hash: vector&lt;u8&gt;, version:u64, enforced: bool) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap, package_hash, version, enforced);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2"></a>

## Function `submit_upgrade_plan_with_cap_v2`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, version: u64, enforced: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, version: u64, enforced: bool) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> package_address = cap.account_address;
    <b>assert</b>!(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>let</b> active_after_time = <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>() + tpu.config.min_time_limit;
    tpu.plan = <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a> { package_hash, active_after_time, version, enforced });
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan"></a>

## Function `cancel_upgrade_plan`

Cancel a module upgrade plan.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = <b>borrow_global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan_with_cap"></a>

## Function `cancel_upgrade_plan_with_cap`

Cancel a module upgrade plan with given cap.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> package_address = cap.account_address;
    <b>assert</b>!(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>assert</b>!(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&tpu.plan), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_module_upgrade_strategy"></a>

## Function `get_module_upgrade_strategy`

Get module upgrade strategy of an module address.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8 <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
        <b>borrow_global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_upgrade_plan"></a>

## Function `get_upgrade_plan`

Get module upgrade plan of an address.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(_module_address: <b>address</b>): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(_module_address: <b>address</b>): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt; {
    // DEPRECATED_CODE
    <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_upgrade_plan_v2"></a>

## Function `get_upgrade_plan_v2`

Get module upgrade plan of an address.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">PackageTxnManager::UpgradePlanV2</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt; <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address)) {
        *&<b>borrow_global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(module_address).plan
    } <b>else</b> {
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt;()
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_check_package_txn"></a>

## Function `check_package_txn`

Check againest on the given package data.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(package_address: <b>address</b>, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>){
        //do nothing
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>){
        <b>let</b> plan_opt = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(package_address);
        <b>assert</b>!(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&plan_opt), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
        <b>let</b> plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&plan_opt);
        <b>assert</b>!(*&plan.package_hash == package_hash, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>));
        <b>assert</b>!(plan.active_after_time &lt;= <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>));
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>){
        //do check at VM runtime.
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>){
        <b>abort</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>)
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_check_package_txn_v2"></a>

## Function `check_package_txn_v2`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn_v2">check_package_txn_v2</a>(txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn_v2">check_package_txn_v2</a>(txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>){
        <b>assert</b>!(txn_sender == package_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>));
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>){
        <b>let</b> plan_opt = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(package_address);
        <b>assert</b>!(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&plan_opt), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
        <b>let</b> plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&plan_opt);
        <b>assert</b>!(*&plan.package_hash == package_hash, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>));
        <b>assert</b>!(plan.active_after_time &lt;= <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>));
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>){
        //do check at VM runtime.
        <b>assert</b>!(txn_sender == package_address, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_AND_PACKAGE_ADDRESS_MISMATCH">ESENDER_AND_PACKAGE_ADDRESS_MISMATCH</a>));
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>){
        <b>abort</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>)
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_finish_upgrade_plan"></a>

## Function `finish_upgrade_plan`



<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a> {
    <b>let</b> tpu = <b>borrow_global_mut</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>&gt;(package_address);
    <b>if</b> (<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&tpu.plan)) {
        <b>let</b> plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&tpu.plan);
        <a href="Config.md#0x1_Config_set_with_capability">Config::set_with_capability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;(&<b>mut</b> tpu.version_cap, <a href="Version.md#0x1_Version_new_version">Version::new_version</a>(plan.version));
        <a href="Event.md#0x1_Event_emit_event">Event::emit_event</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">Self::UpgradeEvent</a>&gt;(&<b>mut</b> tpu.upgrade_event, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradeEvent">UpgradeEvent</a> {
            package_address: package_address,
            package_hash: *&plan.package_hash,
            version: plan.version});
    };
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">UpgradePlanV2</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_prologue"></a>

## Function `package_txn_prologue`

Prologue of package transaction.


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(package_address, package_hash);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_prologue_v2"></a>

## Function `package_txn_prologue_v2`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue_v2">package_txn_prologue_v2</a>(account: &signer, txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue_v2">package_txn_prologue_v2</a>(account: &signer, txn_sender: <b>address</b>, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn_v2">check_package_txn_v2</a>(txn_sender, package_address, package_hash);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_epilogue"></a>

## Function `package_txn_epilogue`

Package txn finished, and clean UpgradePlan


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: <b>address</b>, package_address: <b>address</b>, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: <b>address</b>, package_address: <b>address</b>, success: bool) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgradeV2">TwoPhaseUpgradeV2</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>let</b> strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>){
        <b>if</b> (success) {
            <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address);
        };
    };
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>false</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_update_module_upgrade_strategy"></a>

### Function `update_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8, min_time: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;u64&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> strategy != 0 && strategy != 1 && strategy != 2 && strategy != 3;
<b>aborts_if</b> <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && strategy &lt;= <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).strategy;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && strategy == 0;
<b>aborts_if</b> strategy == 1 && <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> strategy == 1 && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>let</b> holder = <b>global</b>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapabilityHolder">Config::ModifyConfigCapabilityHolder</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> strategy == 1 && <a href="Option.md#0x1_Option_is_none">Option::is_none</a>&lt;<a href="Config.md#0x1_Config_ModifyConfigCapability">Config::ModifyConfigCapability</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;&gt;(holder.cap);
<b>aborts_if</b> strategy == 1 && <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).strategy == 1
    && !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_destroy_upgrade_plan_cap"></a>

### Function `destroy_upgrade_plan_cap`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_extract_submit_upgrade_plan_cap"></a>

### Function `extract_submit_upgrade_plan_cap`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>aborts_if</b> <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).strategy != 1;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
</code></pre>



<a name="@Specification_1_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2"></a>

### Function `convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2`


<pre><code><b>public</b>(<b>script</b>) <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2">convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2</a>(account: signer, package_address: <b>address</b>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
</code></pre>



<a name="@Specification_1_submit_upgrade_plan_v2"></a>

### Function `submit_upgrade_plan_v2`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_v2">submit_upgrade_plan_v2</a>(account: &signer, package_hash: vector&lt;u8&gt;, version: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a>{account: <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address).plan);
</code></pre>



<a name="@Specification_1_submit_upgrade_plan_with_cap_v2"></a>

### Function `submit_upgrade_plan_with_cap_v2`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap_v2">submit_upgrade_plan_with_cap_v2</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, version: u64, enforced: bool)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a>{account: cap.account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(cap.account_address).plan);
</code></pre>




<a name="0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a> {
    account: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account);
    <b>aborts_if</b> <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account);
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="Timestamp.md#0x1_Timestamp_CurrentTimeMilliseconds">Timestamp::CurrentTimeMilliseconds</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>aborts_if</b> <a href="Timestamp.md#0x1_Timestamp_now_milliseconds">Timestamp::now_milliseconds</a>() + <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account).config.min_time_limit &gt; max_u64();
}
</code></pre>



<a name="@Specification_1_cancel_upgrade_plan"></a>

### Function `cancel_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a>{account: <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address).plan);
</code></pre>



<a name="@Specification_1_cancel_upgrade_plan_with_cap"></a>

### Function `cancel_upgrade_plan_with_cap`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a>{account: cap.account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_is_none">Option::is_none</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(cap.account_address).plan);
</code></pre>




<a name="0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a> {
    account: <b>address</b>;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account);
    <b>aborts_if</b> <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account);
    <b>aborts_if</b> !<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account).plan);
}
</code></pre>



<a name="@Specification_1_get_module_upgrade_strategy"></a>

### Function `get_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<a name="0x1_PackageTxnManager_spec_get_module_upgrade_strategy"></a>


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(module_address: <b>address</b>): u8 {
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
   }<b>else</b>{
       0
   }
}
</code></pre>



<a name="@Specification_1_get_upgrade_plan"></a>

### Function `get_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(_module_address: <b>address</b>): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_get_upgrade_plan_v2"></a>

### Function `get_upgrade_plan_v2`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan_v2">get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanV2">PackageTxnManager::UpgradePlanV2</a>&gt;
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> <b>false</b>;
</code></pre>




<a name="0x1_PackageTxnManager_spec_get_upgrade_plan_v2"></a>


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_upgrade_plan_v2">spec_get_upgrade_plan_v2</a>(module_address: <b>address</b>): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt; {
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address).plan
   }<b>else</b>{
       <a href="Option.md#0x1_Option_spec_none">Option::spec_none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()
   }
}
</code></pre>



<a name="@Specification_1_check_package_txn"></a>

### Function `check_package_txn`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a>;
</code></pre>



<a name="@Specification_1_finish_upgrade_plan"></a>

### Function `finish_upgrade_plan`


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: <b>address</b>)
</code></pre>




<pre><code><b>pragma</b> verify = <b>false</b>;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
<b>let</b> tpu = <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
<b>aborts_if</b> <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(tpu.plan) && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;&gt;(tpu.version_cap.account_address);
</code></pre>



<a name="@Specification_1_package_txn_prologue"></a>

### Function `package_txn_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, package_address: <b>address</b>, package_hash: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a>{};
</code></pre>



<a name="@Specification_1_package_txn_epilogue"></a>

### Function `package_txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: <b>address</b>, package_address: <b>address</b>, success: bool)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
    && success && !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
<b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_module_upgrade_strategy">spec_get_module_upgrade_strategy</a>(package_address) == 1
    && success && <a href="Option.md#0x1_Option_is_some">Option::is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address).plan)
    && !<b>exists</b>&lt;<a href="Config.md#0x1_Config_Config">Config::Config</a>&lt;<a href="Version.md#0x1_Version_Version">Version::Version</a>&gt;&gt;(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address).version_cap.account_address);
</code></pre>
