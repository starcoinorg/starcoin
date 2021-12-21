#!/bin/bash
function download() {
  net=$1
  name=$2
  if [ -f "$name" ]; then
    echo -e "$net $name exists. use it"
    return 0
  fi
  compress_name=$name.tar.gz
  url=https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/$net/$compress_name
  for ((i = 0; i < 3; i++)); do
    rm -f "$compress_name"
    wget $url
    case_status=$?
    if [ $case_status -eq 0 ]; then
      echo -e "download $net $name succ"
      break
    fi
  done
  case_status=$?
  if [ $case_status -ne 0 ]; then
    return $case_status
  fi
  tar xzvf "$compress_name"
  case_status=$?
  if [ $case_status -ne 0 ]; then
    echo -e "tar $net $compress_name fail"
    return $case_status
  fi
  return 0
}

function usage() {
  echo -e "usage: import_net_block.sh net to_dir"
  echo -e "net is main, barnard, proxima, halley"
  echo -e "to_dir like ~/.starcoin/main，~/.starcoin/barnard"
}

function import_net_block() {
  net=$1
  block_list=$2
  to_dir=$3

  download "$net" "$block_list"

  while read line; do
    name=$line
    download "$net" "$name"
    case_status=$?
    if [ $case_status -ne 0 ]; then
      echo -e "download $net $name fail"
      exit $case_status
    fi

    ./starcoin_db_exporter apply-block -i "$name" -n "$net" -o "$to_dir"
    case_status=$?
    if [ $case_status -ne 0 ]; then
      echo -e "apply-block $net $name fail"
      exit $case_status
    fi
  done <"$block_list"
  echo -e "$net apply-block succ"
}

if [ $# != 2 ]; then
  usage
  exit 1
fi
net=$1
to_dir=$2
case $net in
"main" | "barnard" | "proxima" |"halley")
  import_net_block "$net" block_list.csv "$to_dir"
  ;;
*)
  echo "$net not supported"
  usage
  ;;
esac
