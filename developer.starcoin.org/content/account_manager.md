---
title: Account management
weight: 20
---

The Starcoin node has a built-in decentralized wallet that allows users to manage their accounts through account api and commands.

When the node starts, a default account is automatically created with an empty password. The default can be changed via account commands. The following commands require a connection to the console, see [Using the starcoin console](./console). 1.



1. The account create command allows you to create an account

```bash
starcoin% account create -p my-pass
+------------+--------------------------------------------------------------------+
| address | 0x8d885d806c14654832aa371c3c980153 |
+------------+--------------------------------------------------------------------+
| is_default | false |
+------------+--------------------------------------------------------------------+
| public_key | 0xf0a2cee9d7c85a40f3f217782b449fab9ba73fa11ab210f11d12305fdf57b908 |
+------------+--------------------------------------------------------------------+

```

2. The account show command allows you to view the account status

```bash
starcoin% account show 0x8d885d806c14654832aa371c3c980153
+--------------------+--------------------------------------------------------------------+
| account.address | 0x8d885d806c14654832aa371c3c980153 |
+--------------------+--------------------------------------------------------------------+
| account.is_default | false |
+--------------------+--------------------------------------------------------------------+
| account.public_key | 0xf0a2cee9d7c85a40f3f217782b449fab9ba73fa11ab210f11d12305fdf57b908 |
+--------------------+--------------------------------------------------------------------+
| auth_key | 0xbc0b37f099741399c30dcd09cfd8a6118d885d806c14654832aa371c3c980153 |
+--------------------+--------------------------------------------------------------------+
| sequence_number | |
+--------------------+--------------------------------------------------------------------+
```

- address is the address of the account.
- public_key is the public key corresponding to the address of the account.
- auth_key is the authentication key.

> Note that creating an account only creates a pair of keys in the starcoin node, and does not update the state of the chain. So balance and sequence_number are still empty at this point. All the above information is public information. 


3. You can see the list of accounts through the account list

```bash
starcoin% account list
+------------------------------------+------------+--------------------------------------------------------------------------------------------------------------------------------------+
| address | is_default | public_key |
+------------------------------------+------------+--------------------------------------------------------------------------------------------------------------------------------------+
| 0xddf5d370b6aae8251dacc99d1ff6fe94 | true | 0xaaf0b46c8a6bb88322e047aebdc90b0be7415583230d2dccff7b3fbe1fcfbfec |
+------------------------------------+------------+--------------------------------------------------------------------------------------------------------------------------------------+
| address | is_default | public_key |
+------------------------------------+------------+--------------------------------------------------------------------------------------------------------------------------------------+
| 0x8d885d806c14654832aa371c3c980153 | false | 0xf0a2cee9d7c85a40f3f217782b449fab9ba73fa11ab210f11d12305fdf57b908 |
+------------------------------------+------------+--------------------------------------------------------------------------------------------------------------------------------------+
```

- is_default: Indicates whether the account is the default account. Many commands that require an account address parameter, if user not passed it, the command will use the default account address. If the node has enable the miner client, the default account will also be used for miner client.

4. View or change the default account via the account default command

To view the default account address.

```bash
account default
```
Set 0x8d885d806c14654832aa371c3c980153 to the default address.
```bash
account default 0x8d885d806c14654832aa371c3c980153
```

> Note: After changing the default account, some services will not automatically use the new default account, it is better to restart the node.

5. Export and import accounts via the account export/import command

In order to avoid losing your assets due to disk corruption and other reasons, it is important to backup your private key.

Execute the following command: 
```bash
account export 0x8d885d806c14654832aa371c3c980153 -p my-pass
```
to export the private key of 0x8d885d806c14654832aa371c3c980153.

Execute the following command:

```bash
account import -i <private-key> -p my-pass 0x8d885d806c14654832aa371c3c980153
```

This will import the 0x8d885d806c14654832aa371c3c980153 account. This command can also be used to import the account to a different node and used to do node migration.
