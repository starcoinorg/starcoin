---
title: Run/Join Network
weight: 6
---


`starcoin` is used to start a local network or join a starcoin network. Running a local network or join test network makes it easier to test and debug your code changes. You can use the CLI command dev to compile, publish, and execute Move programs on your local network or test network. 

<!--more-->

## Usage

`starcoin` [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
- --disable-file-log Disable std error log output
- --disable-seed Disable seed for seed node


OPTIONS:
- --seed config seed node address manually
- -n, --net network name ,it should be one of dev/halley/proxima/barnard/main

SUBCOMMAND:
- console Run node background, after node started ,start cli console
- help  Prints this message or the help of the given subcommand(s)

## Run Local Network

the following command could start new dev node:

```shell
starcoin -n dev
```

after this command,you cound find node address in log or std output ,it could like:

```shell
Self address is: /ip4/127.0.0.1/tcp/59476/p2p/12D3KooWPePRG6BDdjgtEYmPDxNyJfMWpQ1Rwgefuz9eqksLfxJb
```

then you could setup another node by this command:

```shell
starcoin -n dev --seed /ip4/127.0.0.1/tcp/59476/p2p/12D3KooWPePRG6BDdjgtEYmPDxNyJfMWpQ1Rwgefuz9eqksLfxJb

```

You could use subcommand console to start cli console:

```shell
starcoin -n dev console
```

repeat these steps , you cloud get multi node local dev network.

## Join Halley network

**Halley** is first starcoin test network. The data on the chain will be cleaned up periodically。

You could use such command to join Halley network:

```shell
starcoin -n halley
```

Inspiration of the name "Halley" comes from the [Comet Halley](https://en.wikipedia.org/wiki/Halley%27s_Comet), officially designated 1P/Halley, is a short-period comet visible from Earth every 75–76 years.


## Join Proxima network

**Proxima** is starcoin long-running test network, released at the third quater of 2020

You could use such command to join Barnard network:

```shell
starcoin -n proxima
```

Inspiration of the name "Proxima" comes from the [Proxima Centauri](https://en.wikipedia.org/wiki/Proxima_Centauri), it is a small, low-mass star located 4.244 light-years (1.301 pc) away from the Sun in the southern constellation of Centaurus. 


## Join Barnard network

**Barnard** is starcoin permanent test network, barnard is the successor of proxima.

You could use such command to join Barnard network:

```shell
starcoin -n barnard
```

Inspiration of the name "Barnard" comes from the [Barnard's Star](https://en.wikipedia.org/wiki/Barnard%27s_Star), it is a red dwarf about six light-years away from Earth in the constellation of Ophiuchus.


## Join main network

```shell
starcoin -n main
```

The default network is main, so also just run:

```shell
starcoin
```