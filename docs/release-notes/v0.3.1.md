## Main feature and update

1. Epoch and Uncle Block mechanism are introduced, the difficulty of PoW can be dynamically adjusted according to the Uncle Block rate.
2. The `Package` transaction type is introduced, which supports batch deployment of multiple Modules with initialization scripts.
3. Stabilization of Token module and issuance mechanism, the value of Token is changed from u64 to u128, which can support larger total amount and higher accuracy.
4. Implementation of Transaction fee distribution contracts.
5. Stdlib added SortedLinkedList, Math, BitOperators modules. 
6. The BlockReword contract was refactored to accommodate Epoch and Uncle Block mechanisms.
7. Module upgrade mechanism is provided, and developers can customize the strategy of contract upgrade. Module upgrade compatibility check is implemented to ensure compatibility with the old version when upgrading.
8. Refactor Genesis to implement Genesis transaction via Package transaction. Simplify Genesis Account, retaining only 0x1 Genesis account.
9. Introduced the network rpc framework to simplify the implementation of the rpc interface on p2p networks.
10. Introduce Move's coverage tool to count stdlib's test coverage.
11. Simplify Node configuration and unify command line parameter format.

1. 引入 Epoch 以及叔块机制，PoW 出块难度可以根据叔块率来动态调整。
2. 引入 `Package` 交易类型，支持批量部署多个 Module 以及附带初始化脚本。
3. Token 模块以及发行机制的稳定化，Token 的值从 u64 改为 u128，可以支持更大的总量以及更高的精度。
4. 实现了 Transaction fee 的分发合约。
5. Stdlib 增加了 SortedLinkedList，Math，BitOperators 模块。 
6. 重构了 BlockReword 合约，以适应 Epoch 以及叔块机制。
7. 提供了 Module 升级机制，开发者可以自定义合约升级的策略。实现了升级 Module 的兼容性检查，保证升级时和旧的版本兼容。
8. 重构 Genesis，通过 Package 交易实现 Genesis 交易。简化 Genesis Account，只保留 0x1 一个 Genesis account。
9. 引入 network rpc 框架，简化 p2p 网络上的 rpc 接口实现。
10. 引入 Move 的覆盖率工具，统计 stdlib 的测试覆盖率。
11. 简化 Node 配置以及统一命令行参数格式。

## Main dependency bump

1. move-vm bump to 821ac69a5e3ff3e323601c355d8de42f957d9c26 (July 14) .
2. libp2p bump to 0.22.
3. rust tool chain bump to 1.44.1.

For a full rundown of the changes please consult the Starcoin 0.3.0 [release milestone](https://github.com/starcoinorg/starcoin/milestone/8)