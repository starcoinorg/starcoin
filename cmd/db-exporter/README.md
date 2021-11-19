## Database Exporter

A tool to export starcoin database record. 

### Usage

starcoin_db_exporter exporter
```shell
USAGE:
    starcoin_db_exporter exporter [OPTIONS] --db-path <db-path> --schema <schema>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --db-path <db-path>    starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
    -o, --output <output>      output file, like accounts.csv, default is stdout
    -s, --schema <schema>      the table of database which to export, block,block_header
```
starcoin_db_exporter checkkey
```shell
USAGE:
    starcoin_db_exporter checkkey --block-hash <block-hash> --cf-name <cf-name> --db-path <db-path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b, --block-hash <block-hash>    
    -n, --cf-name <cf-name>           [possible values: block, block_header]
    -i, --db-path <db-path>          starcoin node db path. like ~/.starcoin/barnard/starcoindb/db/starcoindb
```
starcoin_db_exporter export-block-range
```shell
./starcoin_db_exporter export-block-range --db-path ~/.starcoin/main -s 1 -e 10000 -n main -o ~/bak/
USAGE:
    starcoin_db_exporter export-block-range --db-path <db-path> --end <end> --net <net> --output <output> --start <start>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --db-path <db-path>    starcoin node db path. like ~/.starcoin/main
    -e, --end <end>            
    -n, --net <net>            Chain Network, like main, proxima
    -o, --output <output>      output dir, like ~/, output filename ~/block_start_end.csv
    -s, --start <start> 
```
starcoin_db_exporter apply-block
```shell
./starcoin_db_exporter apply-block -i ~/block_1_10000.csv -n main -o ~/.starcoin/main
USAGE:
    starcoin_db_exporter apply-block [FLAGS] --input-path <input-path> --net <net> --to-path <to-path> [verifier]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -w, --watch      Watch metrics logs

OPTIONS:
    -i, --input-path <input-path>    input file, like accounts.csv
    -n, --net <net>                  Chain Network
    -o, --to-path <to-path>          starcoin node db path. like ~/.starcoin/main

ARGS:
    <verifier>    Verify type:  Basic, Consensus, Full, None, eg [possible values: Basic, Consensus, Full, None]
```
