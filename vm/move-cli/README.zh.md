# Move CLI

move-cli 是一个用于开发 Move 合约的工具， 用户可以
- 在本地编译 Move 合约（本地的合约可以依赖链上的其他合约）。
- 将合约字节码按照 Move 的存储模型，存放到本地磁盘中。
- 读取链上数据，在本地执行 Move 脚本。
- 将脚本执行结果持久化在本地。


## 安装
```shell
$ cargo install --path starcoin/vm/move-cli
```
或者
```shell
$ cargo install --git https://github.com/starcoinorg/starcoin move-cli --branch master
```
这将在你的 cargo 命令目录中安装`move`命令。在 macOS 和 Linux 上，这通常是`~/.cargo/bin`。你要确保这个位置在 `PATH` 环境变量中。

现在你应该可以运行 Move CLI 了。

```shell
$ move
Move 0.1.0
CLI frontend for Move compiler and VM

USAGE:
    move [FLAGS] [OPTIONS] <SUBCOMMAND>
  ...
```

我们将在这里介绍最常见的Move CLI命令和参数，你也可以通过调用 `move --help` 获取完整参数列表，
或者 `move <command> --help`  获取某个具体命令的参数列表。


## 目录结构
每个 move 项目都应该具有以下结构：
```
name/
└── src
    ├── modules # Directory containing all Move source modules
    │   ├ ...
    │   └── Module.move
    └── scripts # Directory containing all Move scripts
        ├ ...
        └── script.move
```

现在让我们创建一个Move项目，并`cd`到这个目录中。

```shell
$ move scaffold readme
```

## 编译、运行脚本

让我们先从一个简单的脚本开始，打印出它的`signer'。

```rust
script {
use 0x1::Debug;
use 0x1::Signer;
fun main(account: signer) {
    Debug::print(&Signer::address_of(&account));
}
}
```

将这个文件放在`src/scripts`下的`debug_script.move`文件中，然后试试：

```shell
$ move run src/scripts/debug_script.move --signers 0xf
[debug] (&) { 0000000000000000000000000000000F }
```

参数 `---signers 0xf` 表示哪些账户地址在该脚本上签过名。 省略`---signers`或传递多个签名者会触发一个类型错误。

## 传递参数
CLI支持通过`--args`向`move run`传递非`signer`参数，支持以下参数类型。

* `bool' 类型 (`true`, `false`)
* `u64`类型 (例如，`10`, `58`)
* `address` 类型 (例如, `0x12', `0x0000000000000000000f')
* 十六进制字符串（例如，`x"0012"`将被解析为 `vector<u8>`值: `[00, 12]`)
* ASCII字符串 (例如, `b"hi"`将被解析为`vector<u8>`值: `[68, 69]`)



## 发布新模块

编写脚本时，通常需要调用其他 Move 模块，比如上面的例子中的`Debug`模块（`Debug` 模块是一个预定义模块）。
自定义的模块可以添加到项目目录下的`src/modules`目录中。
在运行脚本之前，`move run`命令会编译并发布该目录中的所有模块源文件。
当然你也可以单独编译和发布模块。

试着在`src/modules/Test.move`中保存这段代码。

```rust
address 0x2 {
module Test {
    use 0x1::Signer;

    struct Resource  has key { i: u64 }

    public fun publish(account: &signer) {
        move_to(account, Resource { i: 10 })
    }

    public fun write(account: &signer, i: u64) acquires Resource {
        borrow_global_mut<Resource>(Signer::address_of(account)).i = i;
    }

    public fun unpublish(account: &signer) acquires Resource {
        let Resource { i: _ } = move_from(Signer::address_of(account));
  }
}
}
```

现在，试试：

```shell
$ move check
```

这将导致 move-cli 编译并检查模块的类型，但它不会在`storage`下发布模块字节码。
你可以通过运行 `move publish` 命令来编译和发布模块（这里我们传递 `-v` 标志，以便更好地了解正在发生的事情。)

```shell
$ move publish -v
Compiling Move modules...
Found and compiled 1 modules
```

现在，查看 `storage` 目录下的内容，你会发现 `Test`  模块的字节码文件。

```shell
$ ls storage/0x00000000000000000000000000000002/modules
Test.mv
```

我们还可以通过使用 `move view` 命令查看这个字节码文件。

```shell
$ move view storage/0x00000000000000000000000000000002/modules/Test.mv
module 00000000.Test {
resource Resource {
	i: u64
}

public publish() {
	0: MoveLoc[0](Arg0: &signer)
	1: LdU64(10)
	2: Pack[0](Resource)
	3: MoveTo[0](Resource)
	4: Ret
}
public unpublish() {
	0: MoveLoc[0](Arg0: &signer)
	1: Call[0](address_of(&signer): address)
	2: MoveFrom[0](Resource)
	3: Unpack[0](Resource)
	4: Pop
	5: Ret
}
public write() {
	0: CopyLoc[1](Arg1: u64)
	1: MoveLoc[0](Arg0: &signer)
	2: Call[0](address_of(&signer): address)
	3: MutBorrowGlobal[0](Resource)
	4: MutBorrowField[0](Resource.i: u64)
	5: WriteRef
	6: Ret
}
}
```
你也可以依赖某些预定义的模块，比如说上面提到的 `Debug` 模块。
我们会在本文后面 [_模式_](#初始状态和开发模式) 一节介绍这个事情。

## 更新状态

现在让我们来运行以下脚本，拿 `Test` 模块做做练习。

```rust
script {
use 0x2::Test;
fun main(account: signer) {
    Test::publish(&account)
}
}
```
这个脚本调用了我们的`Test`模块的`publish`函数，它将在签名者的账户下发布一个`Test::Resource`类型的资源。
让我们先看看这个脚本将改变什么（先不提交这些变化）。我们可以通过传递`--dry-run`标志来做到这一点。


```shell
$ move run src/scripts/test_script.move --signers 0xf -v --dry-run
Compiling transaction script...
Changed resource(s) under 1 address(es):
  Changed 1 resource(s) under address 0x0000000000000000000000000000000f:
    Added type 0x00000000000000000000000000000002::Test::Resource: [10, 0, 0, 0, 0, 0, 0, 0] (wrote 40 bytes)
Wrote 40 bytes of resource ID's and data
Discarding changes; re-run without --dry-run if you would like to keep them.
```

一切看起来都很好，所以我们再运行一次，但这次要提交修改的内容。可以通过删除"--dry-run" 标志来提交修改。


```shell
$ move run src/scripts/test_script.move --signers 0xf -v
Compiling transaction script...
Changed resource(s) under 1 address(es):
  Changed 1 resource(s) under address 0x0000000000000000000000000000000f:
    Added type 0x00000000000000000000000000000002::Test::Resource: [10, 0, 0, 0, 0, 0, 0, 0] (wrote 40 bytes)
Wrote 40 bytes of resource ID's and data
```
现在我们使用 `move view` 命令来检查这个新发布的资源。

```shell
$ move view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
resource 0x2::Test::Resource {
        i: 10
}
```

### 清理状态

调用 `move run` 或者 `move publish`，会将状态更新持久化在本地。

但有些时候，我们需要从一个干净的状态重新开始。 可以使用`move clean`命令来删除`storage`目录。

```shell
$ move view storage/0x0000000000000000000000000000000f/resources/0x00000000000000000000000000000002::Test::Resource.bcs
resource 0x2::Test::Resource {
        i: 10
}
$ move clean
$ move view storage/0x0000000000000000000000000000000F/resources/0x00000000000000000000000000000002::Test::Resource.bcs
Error: `move view <file>` must point to a valid file under storage
```


## 初始状态和开发模式

move-cli 提供了几种不同的开发模式。 不同的开发模式，拥有不同的初始状态，以及预定义的模块。

可以使用的模式有以下几种。

* **bare:** 在编译和执行脚本或模块时，没有预定义的模块可以使用。
  例如，使用上面的`debug_script.move`例子。
  
	```shell
	$ move run src/scripts/debug_script.move --signers 0xf --mode bare
	error:

	   ┌── debug_script.move:2:5 ───
	   │
	 2 │ use 0x1::Debug;
	   │     ^^^^^^^^^^ Invalid 'use'. Unbound module: '0x1::Debug'
	   │
	```

* **stdlib:** 这包括了 Starcoin 链初始化时的 [预定义模块](https://github.com/starcoinorg/starcoin/blob/master/vm/stdlib/modules/doc)。
  再次执行命令，会执行成功。
  ```shell
  $ move run src/scripts/debug_script.move --signers 0xf --mode stdlib
  [debug] 0x0000000000000000000000000000000f
  ```
* **starcoin:** 在这种模式下，你可以使用 starcoin 网络上已经存在的模块和状态。
  由`--starcoin-rpc` 和 `--block-number` 参数指定网络和状态，默认为主网络的最新块。
  比如查看当前链的最新高度。
  
  ```move
  script {
  use 0x1::Block;
  use 0x1::Debug;
  fun main() {
    Debug::print(&Block::get_current_block_number());
  }
  }
  ```
  执行这个脚本会打印初链的高度。
  ```
  $ move run src/scripts/test_script.move --mode starcoin
  [debug] 278342
  ```

## 检测不兼容更新

`move publish` 命令会自动检测模块升级是否是兼容性的。
有两种兼容性检查：
* 链接兼容性（例如，删除或改变一个被其他模块调用的公共函数的签名，删除一个被其他模块使用的结构或资源。)
* 数据布局兼容性（例如，添加/删除一个结构字段）。

`move publish`执行的的兼容性分析是保守的。假设我们发布了以下模块。

```
address 0x2 {
module M {
    struct S has key { f: u64, g: u64 }
}
}
```

然后，希望升级成以下模块：

```
address 0x2 {
module M {
    struct S has key { f: u64 } 
}
}
```

再次使用 `move publish` 命令发布这个模块，会得到如下错误:

```
Breaking change detected--publishing aborted. Re-run with --ignore-breaking-changes to publish anyway.
Error: Layout API for structs of module 00000000000000000000000000000002::M has changed. Need to do a data migration of published structs
```

这种情况下，我们没有在全局存储中发布任何 `S` 的实例，但是 `move publish` 仍然报出兼容性问题。
重新运行`move publish --ignore-breaking-changes` 可以强制重新 publish 模块。

