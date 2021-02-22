---
title: Run by docker
weight: 5
---

1. Pull docker image

```shell
docker pull starcoin/starcoin:latest
```

the starcoin binary at /starcoin dir in docker image.

If you want to run a special version, just pull by tag, such as  v0.9.3

```shell
docker pull starcoin/starcoin:v0.9.3
```

2. Run starcoin node

Run proxima network node

```shell
docker run --name starcoin -d --network host -v ~/.starcoin/:/root/.starcoin/ starcoin/starcoin:latest /starcoin/starcoin -n proxima
``` 

3. Attach to console

```shell
docker run --rm -it -v  ~/.starcoin/:/root/.starcoin/ starcoin/starcoin:latest /starcoin/starcoin --connect ~/.starcoin/proxima/starcoin.ipc console
```

More detail about run a network node see [Run/Join Network](./runcustomnetwork).