## TxFactory

A binary to gen real user txn for dev/testing env.

### Usage



```bash
$ ./target/debug/starcoin_txfactory --ipc-path node/dev/starcoin.ipc
```


```
USAGE:
    starcoin_txfactory [OPTIONS] --ipc-path <ipc-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account-address <account-address>            account used to send txn, use default account if not specified
    -p, --account-password <account-password>           [default: ]
    -i, --interval <interval>                          interval(in ms) of txn gen [default: 3000]
        --ipc-path <ipc-path>
    -r, --receiver-address <receiver-address>          address to receive balance, default faucet address
    -k, --receiver-public-key <receiver-public-key>
            public key(hex encoded) of address to receive balance, default to none```