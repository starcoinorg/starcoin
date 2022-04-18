#!/bin/bash
function download() {
  net=$1
  from_dir=$2
  compress_name=snapshot.tar.gz
  url=https://s3.ap-northeast-1.amazonaws.com/main.starcoin.org/$net/$compress_name
  for ((i = 0; i < 3; i++)); do
    rm -f "$compress_name"
    wget $url  -P $from_dir
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
  cd "$from_dir"
  tar xzvf "$compress_name" -C $from_dir
  case_status=$?
  if [ $case_status -ne 0 ]; then
    echo -e "tar $net $compress_name fail"
    return $case_status
  fi
  cd -
  return 0
}

function usage() {
  echo -e "usage: import_snapshot.sh net from_dir to_dir"
  echo -e "net is main, barnard, proxima, halley"
  echo -e "from_dir like ~/snapshot"
  echo -e "to_dir like ~/.starcoin/main，~/.starcoin/barnard"
}

function import_snapshot() {
  net=$1
  from_dir=$2
  to_dir=$3

  download "$net" "$from_dir"

    ./starcoin_db_exporter apply-snapshot -i "$from_dir" -n "$net" -o "$to_dir"
    case_status=$?
    if [ $case_status -ne 0 ]; then
      echo -e "apply-snapshot $net $from_dir fail"
      exit $case_status
    fi
  echo -e "$net apply-snapshot succ"
}

if [ $# != 3 ]; then
  usage
  exit 1
fi
net=$1
from_dir=$2
to_dir=$3
case $net in
"main" | "barnard" | "proxima" |"halley")
  import_snapshot "$net" "$from_dir" "$to_dir"
  ;;
*)
  echo "$net not supported"
  usage
  ;;
esac