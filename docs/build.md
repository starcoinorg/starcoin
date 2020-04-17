# Build from source

1. Clone source code

     ```shell
    git clone https://github.com/starcoinorg/starcoin.git
    cd starcoin
    ```
2. Setup build environment

    ```shell
    ./scripts/dev_setup.sh
    ```
3. Run debug build

    ```shell
   cargo build
    ```
4. Run release build

    ```
   cargo build --release
    ```
   
The debug version starcoin at target/debug/starcoin, and release version at target/release/starcoin. Next to read: [How to start dev network](./dev_network.md).