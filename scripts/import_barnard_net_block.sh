#!/bin/bash
function download() {
  name=$1
  if [ -f "$name" ]; then
      echo -e "$name exists. use it"
      return 0
  fi
  compress_name=$name.tar.gz
  url=https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/barnard/$compress_name
  for((i = 0; i < 3; i++));
  do
    rm -f $compress_name
    wget $url
    case_status=$?
    if [ $case_status -eq 0 ]; then
        echo -e "download $name succ"
        break
    fi
  done
  case_status=$?
  if [ $case_status -ne 0 ]; then
        return $case_status
  fi
  tar xzvf $compress_name
  case_status=$?
  if [ $case_status -ne 0 ]; then
        echo -e "tar $compress_name fail"
        return $case_status
  fi
  return 0
}


echo "usage: import_main_get_block.sh to_dir. to_dir like ~/.starcoin/barnard"
to_dir=$1
STEP=500000
for((i = 0; i < 4; i++));
do
  start=$((i * STEP + 1))
  end=$((i  * STEP + STEP))
  name=block_"$start"_"$end".csv
  download $name
  case_status=$?
  if [ $case_status -ne 0 ]; then
      echo -e "download $name fail"
      exit $case_status
  fi
  ./starcoin_db_exporter apply-block -i $name -n main -o $to_dir
  case_status=$?
  if [ $case_status -ne 0 ]; then
    echo -e "apply-block $name fail"
    exit $case_status
  fi
done
name="block_2000001_2034454.csv"
download $name
case_status=$?
if [ $case_status -ne 0 ]; then
  echo -e "download $name fail"
  exit $case_status
fi

./starcoin_db_exporter apply-block -i $name -n main -o ~/.starcoin/main/
if [ $case_status -ne 0 ]; then
  echo -e "apply-block $name fail"
  exit $case_status
fi
echo -e "apply-block succ"
