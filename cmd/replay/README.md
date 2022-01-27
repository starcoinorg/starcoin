## Replay

A tools for replay data from a network to a new chain.

### Usage



```bash
$ .target/release/starcoin-replay  -n proxima -f $source -t $target -c 10000
```


```

USAGE:
    starcoin-replay [OPTIONS] --from <from> --to <to>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --watch      Watch metrics logs

OPTIONS:
    -c, --block-num <block-num>    Number of block [default: 20000]
    -f, --from <from>              Replay data dir
    -n, --net <net>                Chain Network to replay
    -t, --to <to>                  Target dir
ARGS:
    <verifier>    Verify type:  Basic, Consensus, Full, None, eg [possible values: Basic, Consensus, Full, None]

```