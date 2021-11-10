## Database Exporter

A tool to export starcoin database record. 

### Usage

```shell
USAGE:
    db-exporter exporter [OPTIONS] --db-path <db-path> --schema <schema>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --db-path <db-path>    starcoin node db path. like ~/.starcoin/barnard/starcoindb/db
    -o, --output <output>      output file, like accounts.csv, default is stdout
    -s, --schema <schema>      the table of database which to export, block,block_header
    
USAGE:
    db-exporter checkkey --block-hash <block-hash> --cf-name <cf-name> --db-path <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --block-hash <block-hash>    
    -n, --cf-name <cf-name>           [possible values: block, block_header]
    -i, --db-path <db-path>          starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
USAGE:
    starcoin_db_exporter export-block-range --db-path <db-path> --end <end> --net <net> --output <output> --start <start>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --db-path <db-path>    starcoin node db path. like ~/.starcoin/main
    -e, --end <end>            
    -n, --net <net>            Chain Network, like main, proxima
    -o, --output <output>      output file, like block.csv
    -s, --start <start>
    
USAGE:
    starcoin_db_exporter apply-block [FLAGS] --input-path <input-path> --net <net> --to-path <to-path> [verifier]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --watch      Watch metrics logs

OPTIONS:
    -o, --input-path <input-path>    input file, like accounts.csv
    -n, --net <net>                  Chain Network
    -i, --to-path <to-path>          starcoin node db path. like ~/.starcoin/main

ARGS:
    <verifier>    Verify type:  Basic, Consensus, Full, None, eg [possible values: Basic, Consensus, Full, None]
```
