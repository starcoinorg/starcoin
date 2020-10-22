
<a name="0x1_PackageTxnManager"></a>

# Module `0x1::PackageTxnManager`



-  [Struct `UpgradePlan`](#0x1_PackageTxnManager_UpgradePlan)
-  [Resource `UpgradePlanCapability`](#0x1_PackageTxnManager_UpgradePlanCapability)
-  [Resource `ModuleUpgradeStrategy`](#0x1_PackageTxnManager_ModuleUpgradeStrategy)
-  [Resource `ModuleMaintainer`](#0x1_PackageTxnManager_ModuleMaintainer)
-  [Resource `TwoPhaseUpgrade`](#0x1_PackageTxnManager_TwoPhaseUpgrade)
-  [Constants](#@Constants_0)
-  [Function `STRATEGY_ARBITRARY`](#0x1_PackageTxnManager_STRATEGY_ARBITRARY)
-  [Function `STRATEGY_TWO_PHASE`](#0x1_PackageTxnManager_STRATEGY_TWO_PHASE)
-  [Function `STRATEGY_NEW_MODULE`](#0x1_PackageTxnManager_STRATEGY_NEW_MODULE)
-  [Function `STRATEGY_FREEZE`](#0x1_PackageTxnManager_STRATEGY_FREEZE)
-  [Function `grant_maintainer`](#0x1_PackageTxnManager_grant_maintainer)
-  [Function `update_module_upgrade_strategy`](#0x1_PackageTxnManager_update_module_upgrade_strategy)
-  [Function `account_address`](#0x1_PackageTxnManager_account_address)
-  [Function `destroy_upgrade_plan_cap`](#0x1_PackageTxnManager_destroy_upgrade_plan_cap)
-  [Function `extract_submit_upgrade_plan_cap`](#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap)
-  [Function `submit_upgrade_plan`](#0x1_PackageTxnManager_submit_upgrade_plan)
-  [Function `submit_upgrade_plan_with_cap`](#0x1_PackageTxnManager_submit_upgrade_plan_with_cap)
-  [Function `cancel_upgrade_plan`](#0x1_PackageTxnManager_cancel_upgrade_plan)
-  [Function `cancel_upgrade_plan_with_cap`](#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap)
-  [Function `get_module_maintainer`](#0x1_PackageTxnManager_get_module_maintainer)
-  [Function `get_module_upgrade_strategy`](#0x1_PackageTxnManager_get_module_upgrade_strategy)
-  [Function `get_upgrade_plan`](#0x1_PackageTxnManager_get_upgrade_plan)
-  [Function `check_package_txn`](#0x1_PackageTxnManager_check_package_txn)
-  [Function `finish_upgrade_plan`](#0x1_PackageTxnManager_finish_upgrade_plan)
-  [Function `package_txn_prologue`](#0x1_PackageTxnManager_package_txn_prologue)
-  [Function `package_txn_epilogue`](#0x1_PackageTxnManager_package_txn_epilogue)
-  [Specification](#@Specification_1)
    -  [Function `grant_maintainer`](#@Specification_1_grant_maintainer)
    -  [Function `update_module_upgrade_strategy`](#@Specification_1_update_module_upgrade_strategy)
    -  [Function `destroy_upgrade_plan_cap`](#@Specification_1_destroy_upgrade_plan_cap)
    -  [Function `extract_submit_upgrade_plan_cap`](#@Specification_1_extract_submit_upgrade_plan_cap)
    -  [Function `submit_upgrade_plan`](#@Specification_1_submit_upgrade_plan)
    -  [Function `submit_upgrade_plan_with_cap`](#@Specification_1_submit_upgrade_plan_with_cap)
    -  [Function `cancel_upgrade_plan`](#@Specification_1_cancel_upgrade_plan)
    -  [Function `cancel_upgrade_plan_with_cap`](#@Specification_1_cancel_upgrade_plan_with_cap)
    -  [Function `get_module_maintainer`](#@Specification_1_get_module_maintainer)
    -  [Function `get_module_upgrade_strategy`](#@Specification_1_get_module_upgrade_strategy)
    -  [Function `get_upgrade_plan`](#@Specification_1_get_upgrade_plan)
    -  [Function `check_package_txn`](#@Specification_1_check_package_txn)
    -  [Function `finish_upgrade_plan`](#@Specification_1_finish_upgrade_plan)
    -  [Function `package_txn_prologue`](#@Specification_1_package_txn_prologue)
    -  [Function `package_txn_epilogue`](#@Specification_1_package_txn_epilogue)


<pre><code><b>use</b> <a href="Block.md#0x1_Block">0x1::Block</a>;
<b>use</b> <a href="CoreAddresses.md#0x1_CoreAddresses">0x1::CoreAddresses</a>;
<b>use</b> <a href="Errors.md#0x1_Errors">0x1::Errors</a>;
<b>use</b> <a href="Option.md#0x1_Option">0x1::Option</a>;
<b>use</b> <a href="Signer.md#0x1_Signer">0x1::Signer</a>;
</code></pre>



<a name="0x1_PackageTxnManager_UpgradePlan"></a>

## Struct `UpgradePlan`



<pre><code><b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>
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
<code>active_after_number: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_UpgradePlanCapability"></a>

## Resource `UpgradePlanCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_ModuleUpgradeStrategy"></a>

## Resource `ModuleUpgradeStrategy`



<pre><code><b>resource</b> <b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>strategy: u8</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_ModuleMaintainer"></a>

## Resource `ModuleMaintainer`



<pre><code><b>resource</b> <b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>account_address: address</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_TwoPhaseUpgrade"></a>

## Resource `TwoPhaseUpgrade`



<pre><code><b>resource</b> <b>struct</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>
<code>plan: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="@Constants_0"></a>

## Constants


<a name="0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>: u64 = 104;
</code></pre>



<a name="0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>: u64 = 103;
</code></pre>



<a name="0x1_PackageTxnManager_ESENDER_IS_NOT_MAINTAINER"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_IS_NOT_MAINTAINER">ESENDER_IS_NOT_MAINTAINER</a>: u64 = 101;
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



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>: u64 = 109;
</code></pre>



<a name="0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>: u64 = 102;
</code></pre>



<a name="0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NOT_NONE"></a>



<pre><code><b>const</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NOT_NONE">EUPGRADE_PLAN_IS_NOT_NONE</a>: u64 = 108;
</code></pre>



<a name="0x1_PackageTxnManager_STRATEGY_ARBITRARY"></a>

## Function `STRATEGY_ARBITRARY`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>() : u8{0}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_TWO_PHASE"></a>

## Function `STRATEGY_TWO_PHASE`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>() : u8{1}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_NEW_MODULE"></a>

## Function `STRATEGY_NEW_MODULE`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>(): u8{2}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_FREEZE"></a>

## Function `STRATEGY_FREEZE`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(): u8{3}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_grant_maintainer"></a>

## Function `grant_maintainer`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_grant_maintainer">grant_maintainer</a>(account: &signer, maintainer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_grant_maintainer">grant_maintainer</a>(account: &signer, maintainer: address) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{
   <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(account_address)) {
     borrow_global_mut&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(account_address).account_address = maintainer;
   }<b>else</b>{
     move_to(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{ account_address: maintainer});
   };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_update_module_upgrade_strategy"></a>

## Function `update_module_upgrade_strategy`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>assert</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>() || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>() || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>() || strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUNKNOWN_STRATEGY">EUNKNOWN_STRATEGY</a>));
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> previous_strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address);
    <b>assert</b>(strategy &gt; previous_strategy, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_INCORRECT">ESTRATEGY_INCORRECT</a>));
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address)) {
        borrow_global_mut&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address).strategy = strategy;
    }<b>else</b>{
        move_to(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{ strategy: strategy});
    };
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        move_to(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{ account_address: account_address});
        move_to(account, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{plan: <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()});
    };
    //clean two phase upgrade <b>resource</b>
    <b>if</b> (previous_strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>let</b> tpu = move_from&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
        <b>let</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{plan:_} = tpu;
        // <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> may be extracted
        <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)){
            <b>let</b> cap = move_from&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
            <a href="PackageTxnManager.md#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap);
        };
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_account_address"></a>

## Function `account_address`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">account_address</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_account_address">account_address</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>): address {
    cap.account_address
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_destroy_upgrade_plan_cap"></a>

## Function `destroy_upgrade_plan_cap`



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



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    move_from&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan"></a>

## Function `submit_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan">submit_upgrade_plan</a>(account: &signer, package_hash: vector&lt;u8&gt;, active_after_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan">submit_upgrade_plan</a>(account: &signer, package_hash: vector&lt;u8&gt;, active_after_number: u64) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = borrow_global&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap, package_hash, active_after_number);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan_with_cap"></a>

## Function `submit_upgrade_plan_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, active_after_number: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, active_after_number: u64) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    //FIXME
    //<b>assert</b>(active_after_number &gt;= <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>(), <a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>());
    <b>let</b> account_address = cap.account_address;
    <b>assert</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    <b>let</b> tpu = borrow_global_mut&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&tpu.plan), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NOT_NONE">EUPGRADE_PLAN_IS_NOT_NONE</a>));
    tpu.plan = <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>{ package_hash, active_after_number});
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan"></a>

## Function `cancel_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = borrow_global&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan_with_cap"></a>

## Function `cancel_upgrade_plan_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = cap.account_address;
    <b>assert</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_NOT_TWO_PHASE">ESTRATEGY_NOT_TWO_PHASE</a>));
    <b>let</b> tpu = borrow_global_mut&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&tpu.plan), <a href="Errors.md#0x1_Errors_invalid_state">Errors::invalid_state</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_module_maintainer"></a>

## Function `get_module_maintainer`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(addr: address): address <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr)) {
        borrow_global&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr).account_address
    }<b>else</b>{
        addr
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_module_upgrade_strategy"></a>

## Function `get_module_upgrade_strategy`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: address): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: address): u8 <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
        borrow_global&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_upgrade_plan"></a>

## Function `get_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt; <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> {
    <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address)) {
        *&borrow_global&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address).plan
    }<b>else</b>{
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_check_package_txn"></a>

## Function `check_package_txn`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(sender: address, package_address: address, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> module_maintainer = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(package_address);
    <b>assert</b>(module_maintainer == sender, <a href="Errors.md#0x1_Errors_requires_address">Errors::requires_address</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESENDER_IS_NOT_MAINTAINER">ESENDER_IS_NOT_MAINTAINER</a>));
    <b>let</b> strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>()){
        //do nothing
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>let</b> plan_opt = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(package_address);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&plan_opt), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EUPGRADE_PLAN_IS_NONE">EUPGRADE_PLAN_IS_NONE</a>));
        <b>let</b> plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&plan_opt);
        <b>assert</b>(*&plan.package_hash == package_hash, <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EPACKAGE_HASH_INCORRECT">EPACKAGE_HASH_INCORRECT</a>));
        <b>assert</b>(plan.active_after_number &lt;= <a href="Block.md#0x1_Block_get_current_block_number">Block::get_current_block_number</a>(), <a href="Errors.md#0x1_Errors_invalid_argument">Errors::invalid_argument</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_EACTIVE_TIME_INCORRECT">EACTIVE_TIME_INCORRECT</a>));
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>()){
        //do check at VM runtime.
    }<b>else</b> <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>()){
        <b>abort</b>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_ESTRATEGY_FREEZED">ESTRATEGY_FREEZED</a>)
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_finish_upgrade_plan"></a>

## Function `finish_upgrade_plan`



<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: address) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> {
    <b>let</b> tpu = borrow_global_mut&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_prologue"></a>

## Function `package_txn_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, txn_sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, txn_sender: address, package_address: address, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(txn_sender, package_address, package_hash);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_epilogue"></a>

## Function `package_txn_epilogue`

Package txn finished, and clean UpgradePlan


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: address, package_address: address, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: address, package_address: address, success: bool) <b>acquires</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <a href="CoreAddresses.md#0x1_CoreAddresses_assert_genesis_address">CoreAddresses::assert_genesis_address</a>(account);
    <b>let</b> strategy = <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b>(strategy == <a href="PackageTxnManager.md#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>if</b> (success) {
            <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address)
            //TODO fire event.
        };
    };
}
</code></pre>



</details>

<a name="@Specification_1"></a>

## Specification



<pre><code><b>pragma</b> verify = <b>true</b>;
<b>pragma</b> aborts_if_is_strict = <b>true</b>;
</code></pre>



<a name="@Specification_1_grant_maintainer"></a>

### Function `grant_maintainer`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_grant_maintainer">grant_maintainer</a>(account: &signer, maintainer: address)
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>



<a name="@Specification_1_update_module_upgrade_strategy"></a>

### Function `update_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8)
</code></pre>




<pre><code><b>aborts_if</b> strategy != 0 && strategy != 1 && strategy != 2 && strategy != 3;
<b>aborts_if</b> <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && strategy &lt;= <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).strategy;
<b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)) && strategy == 0;
<b>aborts_if</b> strategy == 1 && <b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
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



<a name="@Specification_1_submit_upgrade_plan"></a>

### Function `submit_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan">submit_upgrade_plan</a>(account: &signer, package_hash: vector&lt;u8&gt;, active_after_number: u64)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a>{account: <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address).plan);
</code></pre>



<a name="@Specification_1_submit_upgrade_plan_with_cap"></a>

### Function `submit_upgrade_plan_with_cap`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, active_after_number: u64)
</code></pre>




<pre><code><b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a>{account: cap.account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(cap.account_address).plan);
</code></pre>




<a name="0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_SubmitUpgradePlanWithCapAbortsIf">SubmitUpgradePlanWithCapAbortsIf</a> {
    account: address;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account);
    <b>aborts_if</b> <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account);
    <b>aborts_if</b> !<a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account).plan);
}
</code></pre>



<a name="@Specification_1_cancel_upgrade_plan"></a>

### Function `cancel_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account));
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a>{account: <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account)).account_address).plan);
</code></pre>



<a name="@Specification_1_cancel_upgrade_plan_with_cap"></a>

### Function `cancel_upgrade_plan_with_cap`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>




<pre><code><b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a>{account: cap.account_address};
<b>ensures</b> <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(cap.account_address).plan);
</code></pre>




<a name="0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf"></a>


<pre><code><b>schema</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CancelUpgradePlanWithCapAbortsIf">CancelUpgradePlanWithCapAbortsIf</a> {
    account: address;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account);
    <b>aborts_if</b> <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account).strategy != 1;
    <b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account);
    <b>aborts_if</b> !<a href="Option.md#0x1_Option_spec_is_some">Option::spec_is_some</a>(<b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account).plan);
}
</code></pre>



<a name="@Specification_1_get_module_maintainer"></a>

### Function `get_module_maintainer`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(addr: address): address
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<a name="0x1_PackageTxnManager_spec_get_module_maintainer"></a>


<pre><code><b>define</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_module_maintainer">spec_get_module_maintainer</a>(addr: address): address {
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr)) {
       <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr).account_address
   }<b>else</b>{
       addr
   }
}
</code></pre>



<a name="@Specification_1_get_module_upgrade_strategy"></a>

### Function `get_module_upgrade_strategy`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: address): u8
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<a name="0x1_PackageTxnManager_sepc_get_module_upgrade_strategy"></a>


<pre><code><b>define</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(module_address: address): u8 {
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
   }<b>else</b>{
       0
   }
}
</code></pre>



<a name="@Specification_1_get_upgrade_plan"></a>

### Function `get_upgrade_plan`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;
</code></pre>




<pre><code><b>aborts_if</b> <b>false</b>;
</code></pre>




<a name="0x1_PackageTxnManager_sepc_get_upgrade_plan"></a>


<pre><code><b>define</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_upgrade_plan">sepc_get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt; {
   <b>if</b> (<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address)) {
       <b>global</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address).plan
   }<b>else</b>{
       <a href="Option.md#0x1_Option_spec_none">Option::spec_none</a>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()
   }
}
</code></pre>



<a name="@Specification_1_check_package_txn"></a>

### Function `check_package_txn`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a>;
</code></pre>




<a name="0x1_PackageTxnManager_CheckPackageTxnAbortsIf"></a>


<pre><code><b>schema</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a> {
    sender: address;
    package_address: address;
    package_hash: vector&lt;u8&gt;;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_spec_get_module_maintainer">spec_get_module_maintainer</a>(package_address) != sender;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 3;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="Option.md#0x1_Option_spec_is_none">Option::spec_is_none</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_upgrade_plan">sepc_get_upgrade_plan</a>(package_address));
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="Option.md#0x1_Option_spec_get">Option::spec_get</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_upgrade_plan">sepc_get_upgrade_plan</a>(package_address)).package_hash != package_hash;
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 1 && !<b>exists</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">Block::BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>());
    <b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 1 && <a href="Option.md#0x1_Option_spec_get">Option::spec_get</a>(<a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_upgrade_plan">sepc_get_upgrade_plan</a>(package_address)).active_after_number &gt; <b>global</b>&lt;<a href="Block.md#0x1_Block_BlockMetadata">Block::BlockMetadata</a>&gt;(<a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ADDRESS">CoreAddresses::GENESIS_ADDRESS</a>()).number;
}
</code></pre>



<a name="@Specification_1_finish_upgrade_plan"></a>

### Function `finish_upgrade_plan`


<pre><code><b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: address)
</code></pre>




<pre><code><b>aborts_if</b> !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
</code></pre>



<a name="@Specification_1_package_txn_prologue"></a>

### Function `package_txn_prologue`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, txn_sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>include</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_CheckPackageTxnAbortsIf">CheckPackageTxnAbortsIf</a>{sender: txn_sender};
</code></pre>



<a name="@Specification_1_package_txn_epilogue"></a>

### Function `package_txn_epilogue`


<pre><code><b>public</b> <b>fun</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: address, package_address: address, success: bool)
</code></pre>




<pre><code><b>aborts_if</b> <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) != <a href="CoreAddresses.md#0x1_CoreAddresses_SPEC_GENESIS_ADDRESS">CoreAddresses::SPEC_GENESIS_ADDRESS</a>();
<b>aborts_if</b> <a href="PackageTxnManager.md#0x1_PackageTxnManager_sepc_get_module_upgrade_strategy">sepc_get_module_upgrade_strategy</a>(package_address) == 1
        && success && !<b>exists</b>&lt;<a href="PackageTxnManager.md#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
</code></pre>
