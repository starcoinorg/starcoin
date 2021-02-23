
---
title: 从源码构建
weight: 5
---

从源码构建 Starcoin。

<!--more-->

1. Clone 源代码

     ```shell
    git clone https://github.com/starcoinorg/starcoin.git
    cd starcoin
    ```
2. Setup build environment

    ```shell
    ./scripts/dev_setup.sh
    ```

    如果操作系统是CentOS ,则需要使用如下命令，单独安装相关开发工具

    ```shell
    yum install -y openssl-devel					# 安装openssl
	yum install -y centos-release-scl 				# 安装centos-release-scl
	yum install -y devtoolset-7						# 安装开发工具
	scl enable devtoolset-7 bash					# 激活开发工具

	# 下面的这两步，会删除错误链接的 llvm-private 包，但同样会导致GUI登录不了，如果使用命令行，则不影响
	# 慎重，如果要使用 GUI 系统，则这步操作很危险
	rpm -qa | grep "llvm-private" 					# 查找包含 llvm-private 的包
	rpm -e --nodeps llvm-private-6.0.1-2.el7.x86_64 # 卸载查找到的包，实际找到的可能和示例不同

    ```shell

3. Run debug build

    ```shell
   cargo build
    ```
4. Run release build

    ```shell
   cargo build --release
    ```
   
debug 版本的 starcoin 在 target/debug/starcoin, release 版本的在 target/release/starcoin. 

注意：如果要正式使用，请使用 release 版本，debug 版本和 release 版本之间的性能有数量级差异。


5. 常见问题排查

	* 若出现 Could not find directory of OpenSSL installation 字样的错误，则需要安装 OpenSSL 库。
	* 若出现 Unable to find libclang: "the `libclang` shared library at /usr/lib64/clang-private/libclang.so.6.0 字样错误，则可能是 llvm-private 的原因，解决方法是卸载它
		rpm -qa | grep "llvm-private" # 查找包含 llvm-private 的包
		rpm -e --nodeps llvm-private-6.0.1-2.el7.x86_64 # 卸载查找到的包
	* 每次编译出错后，解决后，需要cargo clean，清除之前已编译的目标文件，再重新编译