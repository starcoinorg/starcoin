---
title: 通过Dao升级Module
weight: 14
---
Starcoin 支持通过 DAO 治理机制升级 Module。

<!--more-->
1. 编译好要更新的Module；
2. 解锁账号：

```bash
starcoin% account unlock <account address>
```

3. 提议更新Module：

```bash
starcoin% dev module_proposal -s <account address> -m <module path> -v <version>
```
如果stdlib当前版本 >= 2, 改为下面命令：

```bash
starcoin% dev module_proposal_v2 -s <account address> -m <module path> -v <version> -e false
```
其中，参数 -m 表示升级包的路径；-v 表示新的版本号；参数 -e 表示是否跳过 module 兼容性检查强制升级，false 表示不跳过兼容性检查。

4. 查询提议状态：

```bash
starcoin% dev call --module-address <module address> --module-name Dao --func-name proposal_state -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModule --arg <proposal address> --arg <proposal number>
```
如果stdlib当前版本 >= 2, 改为下面命令：

```bash
starcoin% dev call --module-address <module address> --module-name Dao --func-name proposal_state -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModuleV2 --arg <proposal address> --arg <proposal number>
```

5. 任何人都可以给状态为 ACTIVE 的提议投赞成或者反对票：
```bash
starcoin% dev execute -s <account address> -b --script cast_vote -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModule --arg <proposal address> --arg <proposal number> --arg <agree> --arg <votes>u128
```
如果stdlib当前版本 >= 2, 改为下面命令：
```bash
starcoin% dev execute -s <account address> -b --script cast_vote -t 0x1::STC::STC -t 0x1::UpgradeModuleDaoProposal::UpgradeModuleV2 --arg <proposal address> --arg <proposal number> --arg <agree> --arg <votes>u128
```

6. 任何人都可以将状态为 AGREED 的提议放入更新队列：
```bash
starcoin% dev module_queue -s <account address> -a <proposal address> -m <proposal number>
```
如果stdlib当前版本 >= 2, 改为下面命令：
```bash
starcoin% dev module_queue_v2 -s <account address> -a <proposal address> -m <proposal number>
```

执行公示期满后，状态从 QUEUED 变为 EXECUTABLE。

7. 任何人都可以为状态为 EXECUTABLE 的提议提交更新计划：
```bash
starcoin% dev module_plan -s <account address> -a <proposal address> -m <proposal number>
```

8. 如果提议的状态为 EXTRACTED，任何人都可以更新对应的Module：
```bash
starcoin% dev module_exe -s <account address> -m <module path>
```

9. 最后不要忘记取回押金、终结提案。具体可参考上一节 onchain config 的修改。