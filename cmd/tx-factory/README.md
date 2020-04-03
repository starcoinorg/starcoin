## TxFactory

A binary to gen real user txn for dev/testing env.

### Usage



```bash
$ ./target/debug/txfactory --ipc-path node/dev/starcoin.ipc
```


```
USAGE:
    txfactory [OPTIONS] --ipc-path <ipc-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --faucet-address <faucet-address>     [default: 0xa550c18]
    -i, --interval <interval>                interval(in ms) of txn gen [default: 3000]
    -p, --ipc-path <ipc-path>
```