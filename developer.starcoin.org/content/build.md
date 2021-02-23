
---
title: Build from Source
weight: 5
---

Build starcoin from source.

<!--more-->

1. Clone source code

     ```shell
    git clone https://github.com/starcoinorg/starcoin.git
    cd starcoin
    ```
2. Setup build environment

    ```shell
    ./scripts/dev_setup.sh
    ```

    If your operating system is CentOS 6.x , then use such commands to install some toolset.
    
    ```shell
    yum install centos-release-scl
    yum install devtoolset-7
    . /opt/rh/devtoolset-7/enable
    ```shell

3. Run debug build

    ```shell
   cargo build
    ```
4. Run release build

    ```shell
   cargo build --release
    ```
   
The debug version starcoin at target/debug/starcoin, and release version at target/release/starcoin.