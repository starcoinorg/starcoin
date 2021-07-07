## Database Exporter

A tool to export starcoin database record. 

### Usage

```shell
USAGE:
    db-exporter [OPTIONS] --db-path <db-path> --schema <schema>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --db-path <db-path>    starcoin node db path. like ~/.starcoin/barnard/starcoindb/db
    -o, --output <output>      output file, like accounts.csv, default is stdout
    -s, --schema <schema>      the table of database which to export, block,block_header
```
