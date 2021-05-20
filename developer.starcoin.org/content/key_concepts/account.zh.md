---
title: 账号
weight: 1
---

账户，地址、收款识别码、认证密钥和加密密钥

<!--more-->

* 账号：账户代表了 Starcoin 上的一个可以发送交易的资源。一个账户是一组 Move 资源的集合。其由 16 字节的地址唯一标识。
* 认证密钥(authentication_key)：每个账户都会在链上存储了一个认证密钥, 用于认证交易的签名。
* 地址：一个账户的地址来自于它的**初始认证密钥**。Starcoin 支持在不改变其地址的情况下修改账户的认证密钥。
* 收款识别码：一种编码后的地址，包含的校验机制，避免复制的时候被篡改，也包含了初始认证密钥，如果该账号在链上不存在，可以转账时创建。


### 生成认证密钥和加密密钥

* 生成一个新的密钥对 (public_key, private_key). starcoin 密钥使用 Ed25519 curve 及 PureEdDSA scheme 生成，见 RFC 8032
* 生成认证密钥 authentication_key = sha3-256(public_key | 0x00)，其中 | 为连接。0x00 是一个 1bytes 的标识符，表示单签。
* 生成帐户地址 account_address = authentication_key[-16:] 即 authentication_key 的后16字节。
* 收款识别码 receipt_identifier = "stc" + 1 + bech32(account_address + authentication_key?), authentication_key 是可选的。详情参看 sip-21。

因此，对比, Ethereum/Bitcoin, Starcoin 地址不仅更短，为 16 bytes, 同时，在不改变地址的情况下，还支持用户修改密钥对。更加灵活和安全。

任何创建账号的交易，都同时需要 account_address 以及 authentication_key， 但与现有账户交互的交易，则只需要地址。使用收款识别码(receipt_identifier)，则可以同时兼顾这两种情况，对用户屏蔽差异。

1. [sip-21 receipt_identifier](https://developer.starcoin.org/zh/sips/sip-21/)
2. [rotate key 例子](https://github.com/starcoinorg/starcoin-sdk-python/blob/master/examples/rotate_auth_key.py)