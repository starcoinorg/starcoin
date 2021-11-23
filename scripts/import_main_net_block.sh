#!/bin/bash

STEP=500000
main_cnt=$(ps -ef | grep starcoin | grep main | | grep -v export | grep -v grep | wc -l)
echo $main_cnt
if [ $main_cnt -ne 0 ]; then
  echo "main_net running, exit"
  exit 1
fi
for((i = 0; i < 5; i++));
do
  start=$((i * STEP + 1))
  end=$((i  * STEP + STEP))
  name=block_"$start"_"$end".csv
  url=https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/$name
  wget $url
  ./starcoin_db_exporter apply-block -i $name -n main -o ~/.starcoin/main
done
wget https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/block_2500001_2890710.csv
./starcoin_db_exporter apply-block -i block_2500001_2890710.csv -n main -o ~/.starcoin/main/

