
<a name="0x1_PackageTxnManager"></a>

# Module `0x1::PackageTxnManager`

### Table of Contents

-  [Struct `UpgradePlan`](#0x1_PackageTxnManager_UpgradePlan)
-  [Resource `UpgradePlanCapability`](#0x1_PackageTxnManager_UpgradePlanCapability)
-  [Resource `ModuleUpgradeStrategy`](#0x1_PackageTxnManager_ModuleUpgradeStrategy)
-  [Resource `ModuleMaintainer`](#0x1_PackageTxnManager_ModuleMaintainer)
-  [Resource `TwoPhaseUpgrade`](#0x1_PackageTxnManager_TwoPhaseUpgrade)
-  [Function `STRATEGY_ARBITRARY`](#0x1_PackageTxnManager_STRATEGY_ARBITRARY)
-  [Function `STRATEGY_TWO_PHASE`](#0x1_PackageTxnManager_STRATEGY_TWO_PHASE)
-  [Function `STRATEGY_NEW_MODULE`](#0x1_PackageTxnManager_STRATEGY_NEW_MODULE)
-  [Function `STRATEGY_FREEZE`](#0x1_PackageTxnManager_STRATEGY_FREEZE)
-  [Function `grant_maintainer`](#0x1_PackageTxnManager_grant_maintainer)
-  [Function `update_module_upgrade_strategy`](#0x1_PackageTxnManager_update_module_upgrade_strategy)
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



<a name="0x1_PackageTxnManager_UpgradePlan"></a>

## Struct `UpgradePlan`



<pre><code><b>struct</b> <a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>
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

<code>active_after_height: u64</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_UpgradePlanCapability"></a>

## Resource `UpgradePlanCapability`



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>
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



<pre><code><b>resource</b> <b>struct</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>
</code></pre>



<details>
<summary>Fields</summary>


<dl>
<dt>

<code>plan: <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;</code>
</dt>
<dd>

</dd>
</dl>


</details>

<a name="0x1_PackageTxnManager_STRATEGY_ARBITRARY"></a>

## Function `STRATEGY_ARBITRARY`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>() : u8{0}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_TWO_PHASE"></a>

## Function `STRATEGY_TWO_PHASE`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>() : u8{1}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_NEW_MODULE"></a>

## Function `STRATEGY_NEW_MODULE`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>(): u8{2}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_STRATEGY_FREEZE"></a>

## Function `STRATEGY_FREEZE`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(): u8{3}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_grant_maintainer"></a>

## Function `grant_maintainer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_grant_maintainer">grant_maintainer</a>(account: &signer, maintainer: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_grant_maintainer">grant_maintainer</a>(account: &signer, maintainer: address) <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{
   <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
   <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(account_address)) {
     borrow_global_mut&lt;<a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(account_address).account_address = maintainer;
   }<b>else</b>{
     move_to(account, <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{ account_address: maintainer});
   };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_update_module_upgrade_strategy"></a>

## Function `update_module_upgrade_strategy`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_update_module_upgrade_strategy">update_module_upgrade_strategy</a>(account: &signer, strategy: u8) <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>assert</b>(strategy == <a href="#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>() || strategy == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>() || strategy == <a href="#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>() || strategy == <a href="#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>(), 1004);
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> previous_strategy = <a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address);
    <b>assert</b>(strategy &gt; previous_strategy, 1005);
    <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address)) {
        borrow_global_mut&lt;<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(account_address).strategy = strategy;
    }<b>else</b>{
        move_to(account, <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{ strategy: strategy});
    };
    <b>if</b> (strategy == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        move_to(account, <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{ account_address: account_address});
        move_to(account, <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{plan: <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()});
    };
    //clean two phase upgrade <b>resource</b>
    <b>if</b> (previous_strategy == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>let</b> tpu = move_from&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
        <b>let</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>{plan:_} = tpu;
        // <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> may be extracted
        <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)){
            <b>let</b> cap = move_from&lt;<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
            <a href="#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap);
        };
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_destroy_upgrade_plan_cap"></a>

## Function `destroy_upgrade_plan_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_destroy_upgrade_plan_cap">destroy_upgrade_plan_cap</a>(cap: <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>){
    <b>let</b> <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{account_address:_} = cap;
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_extract_submit_upgrade_plan_cap"></a>

## Function `extract_submit_upgrade_plan_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_extract_submit_upgrade_plan_cap">extract_submit_upgrade_plan_cap</a>(account: &signer): <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a> <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>, <a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>assert</b>(<a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), 1006);
    move_from&lt;<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address)
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan"></a>

## Function `submit_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_submit_upgrade_plan">submit_upgrade_plan</a>(account: &signer, package_hash: vector&lt;u8&gt;, active_after_height: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_submit_upgrade_plan">submit_upgrade_plan</a>(account: &signer, package_hash: vector&lt;u8&gt;, active_after_height: u64) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = borrow_global&lt;<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap, package_hash, active_after_height);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_submit_upgrade_plan_with_cap"></a>

## Function `submit_upgrade_plan_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap: &<a href="#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, active_after_height: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_submit_upgrade_plan_with_cap">submit_upgrade_plan_with_cap</a>(cap: &<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>, package_hash: vector&lt;u8&gt;, active_after_height: u64) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    //FIXME
    //<b>assert</b>(active_after_height &gt;= <a href="Block.md#0x1_Block_get_current_block_height">Block::get_current_block_height</a>(), 1005);
    <b>let</b> account_address = cap.account_address;
    <b>assert</b>(<a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), 1006);
    <b>let</b> tpu = borrow_global_mut&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_none">Option::is_none</a>(&tpu.plan), 1007);
    tpu.plan = <a href="Option.md#0x1_Option_some">Option::some</a>(<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>{ package_hash, active_after_height});
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan"></a>

## Function `cancel_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_cancel_upgrade_plan">cancel_upgrade_plan</a>(account: &signer) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>,<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = <a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account);
    <b>let</b> cap = borrow_global&lt;<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>&gt;(account_address);
    <a href="#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_cancel_upgrade_plan_with_cap"></a>

## Function `cancel_upgrade_plan_with_cap`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="#0x1_PackageTxnManager_UpgradePlanCapability">PackageTxnManager::UpgradePlanCapability</a>)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_cancel_upgrade_plan_with_cap">cancel_upgrade_plan_with_cap</a>(cap: &<a href="#0x1_PackageTxnManager_UpgradePlanCapability">UpgradePlanCapability</a>) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>,<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> account_address = cap.account_address;
    <b>assert</b>(<a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(account_address) == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>(), 1006);
    <b>let</b> tpu = borrow_global_mut&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(account_address);
    <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&tpu.plan), 1007);
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_module_maintainer"></a>

## Function `get_module_maintainer`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(addr: address): address
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(addr: address): address <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>{
    <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr)) {
        borrow_global&lt;<a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>&gt;(addr).account_address
    }<b>else</b>{
        addr
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_module_upgrade_strategy"></a>

## Function `get_module_upgrade_strategy`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: address): u8
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(module_address: address): u8 <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address)) {
        borrow_global&lt;<a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>&gt;(module_address).strategy
    }<b>else</b>{
        0
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_get_upgrade_plan"></a>

## Function `get_upgrade_plan`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option_Option">Option::Option</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">PackageTxnManager::UpgradePlan</a>&gt;
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(module_address: address): <a href="Option.md#0x1_Option">Option</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt; <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> {
    <b>if</b> (exists&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address)) {
        *&borrow_global&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(module_address).plan
    }<b>else</b>{
        <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;()
    }
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_check_package_txn"></a>

## Function `check_package_txn`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(sender: address, package_address: address, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>, <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a>{
    <b>let</b> module_maintainer = <a href="#0x1_PackageTxnManager_get_module_maintainer">get_module_maintainer</a>(package_address);
    //TODO <b>define</b> error code.
    <b>assert</b>(module_maintainer == sender, 1000);
    <b>let</b> strategy = <a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b> (strategy == <a href="#0x1_PackageTxnManager_STRATEGY_ARBITRARY">STRATEGY_ARBITRARY</a>()){
        //do nothing
    }<b>else</b> <b>if</b>(strategy == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>let</b> plan_opt = <a href="#0x1_PackageTxnManager_get_upgrade_plan">get_upgrade_plan</a>(package_address);
        <b>assert</b>(<a href="Option.md#0x1_Option_is_some">Option::is_some</a>(&plan_opt), 1001);
        <b>let</b> plan = <a href="Option.md#0x1_Option_borrow">Option::borrow</a>(&plan_opt);
        <b>assert</b>(*&plan.package_hash == package_hash, 1002);
        <b>assert</b>(plan.active_after_height &lt;= <a href="Block.md#0x1_Block_get_current_block_height">Block::get_current_block_height</a>(), 1003);
    }<b>else</b> <b>if</b>(strategy == <a href="#0x1_PackageTxnManager_STRATEGY_NEW_MODULE">STRATEGY_NEW_MODULE</a>()){
        //do check at VM runtime.
    }<b>else</b> <b>if</b>(strategy == <a href="#0x1_PackageTxnManager_STRATEGY_FREEZE">STRATEGY_FREEZE</a>()){
        <b>abort</b>(1004)
    };
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_finish_upgrade_plan"></a>

## Function `finish_upgrade_plan`



<pre><code><b>fun</b> <a href="#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address: address) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a> {
    <b>let</b> tpu = borrow_global_mut&lt;<a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>&gt;(package_address);
    tpu.plan = <a href="Option.md#0x1_Option_none">Option::none</a>&lt;<a href="#0x1_PackageTxnManager_UpgradePlan">UpgradePlan</a>&gt;();
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_prologue"></a>

## Function `package_txn_prologue`



<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, txn_sender: address, package_address: address, package_hash: vector&lt;u8&gt;)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_package_txn_prologue">package_txn_prologue</a>(account: &signer, txn_sender: address, package_address: address, package_hash: vector&lt;u8&gt;) <b>acquires</b> <a href="#0x1_PackageTxnManager_ModuleMaintainer">ModuleMaintainer</a>, <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <a href="#0x1_PackageTxnManager_check_package_txn">check_package_txn</a>(txn_sender, package_address, package_hash);
}
</code></pre>



</details>

<a name="0x1_PackageTxnManager_package_txn_epilogue"></a>

## Function `package_txn_epilogue`

Package txn finished, and clean UpgradePlan


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: address, package_address: address, success: bool)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>public</b> <b>fun</b> <a href="#0x1_PackageTxnManager_package_txn_epilogue">package_txn_epilogue</a>(account: &signer, _txn_sender: address, package_address: address, success: bool) <b>acquires</b> <a href="#0x1_PackageTxnManager_TwoPhaseUpgrade">TwoPhaseUpgrade</a>, <a href="#0x1_PackageTxnManager_ModuleUpgradeStrategy">ModuleUpgradeStrategy</a> {
    // Can only be invoked by genesis account
    <b>assert</b>(<a href="Signer.md#0x1_Signer_address_of">Signer::address_of</a>(account) == <a href="CoreAddresses.md#0x1_CoreAddresses_GENESIS_ACCOUNT">CoreAddresses::GENESIS_ACCOUNT</a>(), <a href="ErrorCode.md#0x1_ErrorCode_ENOT_GENESIS_ACCOUNT">ErrorCode::ENOT_GENESIS_ACCOUNT</a>());
    <b>let</b> strategy = <a href="#0x1_PackageTxnManager_get_module_upgrade_strategy">get_module_upgrade_strategy</a>(package_address);
    <b>if</b>(strategy == <a href="#0x1_PackageTxnManager_STRATEGY_TWO_PHASE">STRATEGY_TWO_PHASE</a>()){
        <b>if</b> (success) {
            <a href="#0x1_PackageTxnManager_finish_upgrade_plan">finish_upgrade_plan</a>(package_address)
            //TODO fire event.
        };
    };
}
</code></pre>



</details>
