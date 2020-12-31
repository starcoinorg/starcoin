## TxFactory

A binary to gen real user txn for dev/testing env.

### Usage



```bash
$ ./target/debug/starcoin_txfactory --ipc-path node/dev/starcoin.ipc
```


```
USAGE:
    starcoin_txfactory [FLAGS] [OPTIONS] --ipc-path <ipc-path>

FLAGS:
    -h, --help       Prints help information
    -s, --stress     is stress test or not
    -V, --version    Prints version information

OPTIONS:
    -a, --account-address <account-address>            account used to send txn, use default account if not specified
    -n, --account-num <account-num>                    numbers of account will be created [default: 30]
    -p, --account-password <account-password>           [default: ]
    -i, --interval <interval>                          interval(in ms) of txn gen [default: 1000]
        --ipc-path <ipc-path>                          
    -r, --receiver-address <receiver-address>          address to receive balance, default faucet address
    -k, --receiver-public-key <receiver-public-key>    public key(hex encoded) of address to receive balance
    -t, --round-num <round-num>                        count of round number [default: 20]
    -w, --watch-timeout <watch-timeout>                watch_timeout [default: 60]
```