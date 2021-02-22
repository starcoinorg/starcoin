---
title: 形式化验证
weight: 10
---

介绍 Move 的形式化验证工具 Move Prover的基本用法。

<!--more-->

# 形式化验证


Move 的形式化验证工具 Move Prover 可以对 Move 程序进行形式化验证。它可以自动证明 Move 智能合约是否符合其形式化规范，同时提供类似于类型检查器的用户体验。Move Prover的设计目标是使智能合约更具“可信赖性”：

- 保护由Starcoin区块链管理的大量资产免受智能合约漏洞的影响
- 有利于监管机构审查和合规要求
- 允许具有数学背景但不一定具有软件工程背景的领域专家了解智能合约的作用
  
  
## 安装

如果开发者曾经使用scripts/dev_setup.sh安装了Starcoin开发环境，那么他实际上已经安装了Move Prover。
如果之前没有安装过，则可以运行（在Starcoin根目录中）：

```shell script
bash scripts/move_prover.sh
```

目前仅支持在MacOS和Linux版本（如Ubuntu或CentOS）上运行。
注意，安装完成后必须在shell中添加以下环境变量。例如在MacOS上，添加到`〜/.bashrc`（或其他shell配置）：

```
export BOOGIE_EXE=/Users/$(whoami)/.dotnet/tools/boogie
export Z3_EXE=/usr/local/bin/z3
```

## 运行

通常是在 Starcoin源码环境中使用`cargo run`运行 Move Prover。
例如在当前目录中对`arithm.move`做形式化验证，需要告诉Move Prover在哪里
查找源文件的依赖（本例中 arithm.move 没有依赖）：
```shell script
> cargo run --package move-prover -- --dependency . arithm.move
```

如果验证成功，Prover将打印一些统计信息，否则将打印错误诊断信息。
下面我们使用`arithm.move`来看一下如何对其进行形式化验证。首先，需要编写形式化规范。

```move
/// arithm.move
module TestArithmetic {

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    fun arithmetic(x: u64, y: u64): u64 {
        (x + y) / x
    }
}
```

Move的形式化规范通常直接添加到源码里面。 `pragma verify = true` 用来告诉 Prover 需要验证 Module 中的所有方法。
`pragma aborts_if_is_strict = true` 告诉 Prover 必须严格检查所有退出条件。然后我们运行上面的命令就会输出如下错误信息：

```abort happened here with execution failure```

这说明有一些退出路径没有被规范覆盖到。可以通过添加下面的 aborts_if 退出条件来完善规范。规范添加完整后 Porver 将不再报错。

```move
module TestArithmetic {

    spec module {
        pragma verify = true;
        pragma aborts_if_is_strict = true;
    }

    fun arithmetic(x: u64, y: u64): u64 {
        (x + y) / x
    }

    spec fun arithmetic {
        aborts_if x + y > max_u64();
        aborts_if x == 0;
    }
}
```

更多信息请参考下面的文档：

-  [Move智能合约的形式化验证工具](http://westar.io/blog/move_prover/)
-  [Move Prover 用户指南](https://github.com/starcoinorg/starcoin/tree/master/vm/move-prover/docs/prover-guide.md)
-  [Move 形式化规范语言](https://github.com/starcoinorg/starcoin/tree/master/vm/move-prover/docs/spec-lang.md)

