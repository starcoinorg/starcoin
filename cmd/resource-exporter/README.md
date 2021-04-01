## Resource Exporter

A tool to export some kind of resource of under all accounts.
It save the data in csv.

### Usage

```shell
USAGE:
    resource-exporter [OPTIONS] <fields>... --block-id <block-id> --db-path <db-path> --output <output>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --block-id <block-id>    block id which we want do snapshot on
    -i, --db-path <db-path>      starcoin node db path. like ~/.starcoin/proxima/starcoindb/db
    -o, --output <output>        output file, like accounts.csv
    -r <resource-type>           resource struct tag [default: 0x1::Account::Balance<0x1::STC::STC>]

ARGS:
    <fields>...    fields of the struct to output. it use pointer syntax of serde_json. like: /authentication_key
                   /sequence_number /deposit_events/counter
```

examples:

```shell
resource-exporter -i ~/.starcoin/proxima/starcoindb/db \
-o balance.csv \
--block-id 0xeca4128459f455ed25e8c7c72830e45bbb6232bc259604a50118a2af32f1c5ed \
-r '0x1::Account::Balance<0x1::STC::STC>' \
/token/value
```
