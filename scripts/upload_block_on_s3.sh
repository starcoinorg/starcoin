#!/bin/bash
net=$1
start=$2
end=$3
./starcoin_db_exporter export-block-range -i /sc-data/$net -s $start -e $end -n $net -o ./
filename=block_"$start"_"$end".csv
compress_name=$filename".tar.gz"
tar czvf $compress_name $filename
aws s3api put-object --bucket main.starcoin.org --key "$net"/"$compress_name" --body $compress_name
