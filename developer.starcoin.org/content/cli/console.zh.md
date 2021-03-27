---
title: 使用 starcoin 控制台 
weight: 1
---

使用 starcoin 的控制台有两种方式，一种是节点启动时同时进入控制台。

<!--more-->

以下命令会启动一个 dev 节点并进入控制台:

```shell
starcoin -n dev console
```

以下命令会启动一个 barnard 节点并进入控制台：

```shell
starcoin -n barnard console
```

如果这种方式启动，控制台和节点在同一个进程内，控制台退出，节点也会自动退出。

另外一种方式是通过控制台连接到一个已经启动的节点上。


### 启动 cli console

假设你的节点目录是默认目录，如果不是请通过 -d 参数指定。

执行以下命令，进入 starcoin console。

- 通过本地的 IPC 进行连接：

这个命令和启动节点同时进入控制台的命令一样，命令会自动检测目录下是否有 ipc 文件，如果有则会自动连接，不再启动新的节点。

``` shell
starcoin -n barnard console
```

或者明确指定 ipc 文件。 

``` shell
starcoin --connect ~/.starcoin/barnard/starcoin.ipc console
```

注: Windows 下的 ipc 文件路径不一样

``` shell
starcoin.exe --connect \\.\pipe\starcoin.ipc console
```

- 通过 websocket 连接：


然后执行以下命令进入 console。

```shell
starcoin --connect ws://127.0.0.1:9870 console
```

`9870` 是 starcoin 的默认 websocket 端口，如果你修改了它，请替换成自己修改后的值。 

可以通过节点 config 文件查看 websocket 端口，默认的 config 文件在 `~/.starcoin/barnard/config.yml`

更多命令和参数请通过  starcoin help 查看。
