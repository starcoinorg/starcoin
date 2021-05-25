---
id: multisig-account
title: 多签账户以及多签交易
weight: 5

---

本节介绍多签账户的使用，包括如何创建通过 cli 在链上创建多签账户，以及如何发起多签交易。

<!--more-->

## 前置准备

多签交易涉及到多个参与者。这里我们用 alice, bob, tom 三个参与者来说明多签交易的流程。

1. 首先你需要在本地启动三个 starcoin dev 节点，分别对应到 alice,bob,tom。并同时连接到 console 中。
```
% starcoin -n dev -d alice -o json console
% starcoin -n dev -d bob -o json console
% starcoin -n dev -d tom -o json console
```

2. 通过以下命令各自生成一个公私钥对
```shell
%starcoin account generate-keypair
```

这里假设他们三者生成对私钥信息（只用于举例，请勿使用在正式网络中）分别是：

- alice:
  - address: 0x662275a33c99a1e3f4d1dd3bf712470f
  - pubkey: 0x6cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c03034    
  - prikey: 0x48d097cb7dafea94c0310c6c02bba075955913bf2d356faffbf9adc6fc6d8e1a
- bob:
  - address: 0x51888ec9961c09db913bfe2bfacd8ec1
  - authkey: 0x951d6a3008eec3b0add413a622514d5c51888ec9961c09db913bfe2bfacd8ec1
  - pubkey: 0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b584    
  - prikey: 0x744dcfb091bce98d5aeec1e28471251a526d1b4a6240db1ec50655a285703bac
  
- tom: 
  - address: 0x49ea9eb68253cde31bf4cda26d640a21
  - pubkey: 0x3db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da7064066
  - prikey: 0x74f6c2f05a7b369351a21af3afd05d94aed5b254269c5f149a23a4a600a202c0
  

3. 最后在 tom 的 starcoin console 用 `dev get_coin` 给 tom 账户充钱。

```
starcoin% dev get_coin
```


做完上述准备后，下面开始我们的多签交易流程。主要步骤如下：

1. 首先我们在本地创建一个多签账户。
2. 然后从 alice 向这个多签账户转一笔钱。
3. 最后以这个多签账户的名义发起一个多签交易：从多签账户给 bob 转账。

## 生成多签账户

这里假设读者了解多签账户的基本概念。

这一小节里，我们会创建由 **三个** 参与者共同维护的多签账户，交易只需要其中**两个**参与者的签名即可（ `threshold=2` ）。
然后我们从这三个公钥以及 `threshold=2`  来生成多签账户的地址信息。

首先分别在 alice，bob，tom 的节点中生成三人共同维护的多签账户。

在 alice 的 console 中执行：

```bash

starcoin% account import-multisig --pubkey 0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b584 --pubkey 0x3db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da7064066 --prikey 0x48d097cb7dafea94c0310c6c02bba075955913bf2d356faffbf9adc6fc6d8e1a -t 2
```

在 bob 的 console 中执行：

```bash

starcoin% account import-multisig --pubkey 0x6cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c03034 --pubkey 0x3db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da7064066 --prikey 0x744dcfb091bce98d5aeec1e28471251a526d1b4a6240db1ec50655a285703bac -t 2
```

在 tom 的 console 中执行：

```bash

starcoin% account import-multisig --pubkey 0x6cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c03034 --pubkey 0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b584 --prikey 0x74f6c2f05a7b369351a21af3afd05d94aed5b254269c5f149a23a4a600a202c0 -t 2
```

你会发现，三个命令会生成相同的多签地址信息。

```bash
starcoin% account show 0xdec266f6749fa0b193f3a7f89d3cd9f2
{
  "ok": {
    "account": {
      "address": "0xdec266f6749fa0b193f3a7f89d3cd9f2",
      "is_default": false,
      "public_key": "0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b5843db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da70640666cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c0303402"
    },
    "auth_key": "0x0ed57ae832f34fc5b1a744c7c7f65e5fdec266f6749fa0b193f3a7f89d3cd9f2",
    "receipt_identifier": "stc1pmmpxdan5n7stryln5luf60xe7g8d27hgxte5l3d35azv03lkte0aasnx7e6flg93j0e607ya8nvly44eynp",
    "sequence_number": null,
    "balances": {}
  }
}
```



## 给多签账户打钱

这一小节，我们从 tom 账户给这个多签账户转 1000 个 STC。

在 tom 的 starcoin console 中执行：

```bash
starcoin% account execute-function -b --function 0x1::TransferScripts::peer_to_peer  -t 0x1::STC::STC --arg 0xdec266f6749fa0b193f3a7f89d3cd9f2 --arg x"0ed57ae832f34fc5b1a744c7c7f65e5fdec266f6749fa0b193f3a7f89d3cd9f2" --arg 1000000000000u128
```

再查看多签账户的信息：

```bash
starcoin% account show 0xdec266f6749fa0b193f3a7f89d3cd9f2
{
  "ok": {
    "account": {
      "address": "0xdec266f6749fa0b193f3a7f89d3cd9f2",
      "is_default": false,
      "public_key": "0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b5843db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da70640666cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c0303402"
    },
    "auth_key": "0x0ed57ae832f34fc5b1a744c7c7f65e5fdec266f6749fa0b193f3a7f89d3cd9f2",
    "receipt_identifier": "stc1pmmpxdan5n7stryln5luf60xe7g8d27hgxte5l3d35azv03lkte0aasnx7e6flg93j0e607ya8nvly44eynp",
    "sequence_number": 0,
    "balances": {
      "STC": 1000000000000
    }
  }
}
```

## 发起多签交易

现在多签账户有了 1000 STC。

我们来发起一个多签交易：从多签账户往 bob 转账 1 个 STC。

在 tom 的 starcoin console 中执行：

```bash
starcoin% account sign-multisig-txn -s 0xdec266f6749fa0b193f3a7f89d3cd9f2 --function 0x1::TransferScripts::peer_to_peer -t 0x1::STC::STC --arg 0x51888ec9961c09db913bfe2bfacd8ec1 --arg x"951d6a3008eec3b0add413a622514d5c51888ec9961c09db913bfe2bfacd8ec1" --arg 1000000000u128

mutlisig txn(address: 0xdec266f6749fa0b193f3a7f89d3cd9f2, threshold: 2): 1 signatures collected
still require 1 signatures
{
  "ok": "/Users/caojiafeng/projects/starcoinorg/starcoin/5e764f83.multisig-txn"
}
```

其中 `peer_to_peer` 脚本参数：
- `0x51888ec9961c09db913bfe2bfacd8ec1` 是 bob 地址。
- `x"951d6a3008eec3b0add413a622514d5c51888ec9961c09db913bfe2bfacd8ec1"` 是 bob 的 auth_key。
- `1000000000u128` 是要发送的 token 数量。 

该命令会生成原始交易，并用 alice 的私钥签名，生成的 txn 会以文件形式保存在当前目录下，文件名是 txn 的 short hash。

命令行提示：该多签交易还需要一个签名。
那么需要将生成的 txn 文件分发给该多签账户的其他参与者去签名。

## ALICE 签名

alice 拿到上述的交易文件后，在自己的 starcoin cosole 中签名：


```bash
starcoin% account sign-multisig-txn /Users/caojiafeng/projects/starcoinorg/starcoin/5e764f83.multisig-txn
mutlisig txn(address: 0xdec266f6749fa0b193f3a7f89d3cd9f2, threshold: 2): 2 signatures collected
enough signatures collected for the multisig txn, txn can be submitted now
{
  "ok": "/Users/caojiafeng/projects/starcoinorg/starcoin/194d547f.multisig-txn"
}
```

该命令会生成另一个交易文件，包含有 tom 和 alice 的签名。
返回信息提示用户，该多签交易已经收集到足够多的签名，可以提交到链上执行了。


## 提交多签交易

多签交易完整生成后，任何人都可以将其提交到链上。
这里我们从 alice 的 starcoin console 中提交该多签交易。

```bash
starcoin% account submit-multisig-txn /Users/caojiafeng/projects/starcoinorg/starcoin/194d547f.multisig-txn
{
  "ok": "0x194d547f06018c0bad6312db0dae75ce4dd26afd302410a9647e5720e395878a"
}
```

等待10秒，再查看多签账户的信息，会发现多签账户的余额已经减少了（gas 费用和转出去的 1 stc）， sequence number 也变成了 1，说明交易已经执行成功了。

```bash
starcoin% account show 0xdec266f6749fa0b193f3a7f89d3cd9f2
{
  "ok": {
    "account": {
      "address": "0xdec266f6749fa0b193f3a7f89d3cd9f2",
      "is_default": false,
      "public_key": "0x2ffe29f29064bf839c3194bf852b00f35fd9351afb602832ae64a754e8c2b5843db1b2a0172f8fb857afc5abebd98ecf969a2fcf2ba90e4b759ebc3da70640666cbe3fb3639a98fc5b8637b8280a8d5bb927b50fa2e2cdfa53a5de9395c0303402"
    },
    "auth_key": "0x0ed57ae832f34fc5b1a744c7c7f65e5fdec266f6749fa0b193f3a7f89d3cd9f2",
    "receipt_identifier": "stc1pmmpxdan5n7stryln5luf60xe7g8d27hgxte5l3d35azv03lkte0aasnx7e6flg93j0e607ya8nvly44eynp",
    "sequence_number": 1,
    "balances": {
      "STC": 998998928221
    }
  }
}
```

## 结语

至此，你已经完成了一个多签账户的创建以及多签交易的生成和链上执行。
