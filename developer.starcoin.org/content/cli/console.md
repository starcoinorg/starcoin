---
title: Using the starcoin console
weight: 1
---

There are two ways to use starcoin's console, one is to enter the console at the same time as the node starts.

<!--more-->

The following command will start a dev node and enter the console:

```shell
starcoin -n dev console
```

The following command will start a barnard node and enter the console.

```shell
starcoin -n barnard console
```

If started this way, the console and the node are in the same process, and if the console exits, the node will also exit automatically.

Another way is to connect to a started node via the console.


### Start cli console

It is assumed that your node directory is the default directory, if not please specify it with the -d parameter.

Execute the following command to access the starcoin console.

- To connect via the local IPC.

This command is the same as the one that starts the node while entering the console. The command will automatically detect if there is an ipc file in the directory, and if so, it will automatically connect the node.

``` shell
starcoin -n barnard console
```

Or specify the ipc file explicitly. 

``` shell
starcoin --connect ~/.starcoin/barnard/starcoin.ipc console
```

Note: The path to the ipc file is different on Windows

``` shell
starcoin.exe --connect \\. \pipe\starcoin.ipc console
```

- To connect via websocket.


Then execute the following command to access console.

```shell
starcoin --connect ws://127.0.0.1:9870 console
```

``9870`` is the default websocket port for starcoin, if you have modified it, please replace it with your own modified value. 

You can view the websocket port through the node config file, the default config file is `~/.starcoin/barnard/config.yml`.

More commands and parameters can be found in starcoin help.
