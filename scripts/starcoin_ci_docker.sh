#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cfg_root=/mnt/volume_01/starcoin_cfg

check_errs() {
  # Function. Parameter 1 is the return code
  # Para. 2 is text to display on failure.
  if [ "${1}" -ne "0" ]; then
    echo "ERROR # ${1} : ${2}"
    # as a bonus, make our script exit with the right error code.
    exit ${1}
  fi
}

function docker_rebuild() {
  echo -e "*\n"'!'"starcoin""\n!txfactory\n"'!'"start.sh" >$DIR/../target/debug/.dockerignore
  cp $DIR/start.sh $DIR/../target/debug/
  docker build -f $DIR/DockerfileCi -t starcoin:latest $DIR/../target/debug/
  check_errs $? "Docker build error"
  docker image prune -f
}

function start_starcoin() {
  local host_name=$1
  local name=$2
  local port=$3
  local m_port=$4
  local net=$5
  shift 5
  eval $(docker-machine env $host_name)
  docker_rebuild
  docker rm -f $name 1>/dev/null
  docker run -td --log-opt mode=non-blocking --log-opt max-size=1m --restart=on-failure:10 --network=host -p $port:9840 -p $m_port:9101 -p 9850:9850 -p 9851:9851 -p 9852:9852 -v $cfg_root/$name:/.starcoin --name $name starcoin $net $@
  check_errs $? "Docker run starcoin error"
}

function start_txfactory() {
  local host_name=$1
  local starcoin_name=$2
  local name=$3
  local net=$4
  shift 4
  eval $(docker-machine env $host_name)
  docker_rebuild
  docker rm -f $name 1>/dev/null
  docker run -td --restart=on-failure:10 -v $cfg_root/$starcoin_name:/.starcoin --name $name --entrypoint "/starcoin/txfactory" starcoin:latest --ipc-path /.starcoin/$net/starcoin.ipc $@
  check_errs $? "Docker run txfactory error"
}

function start_cluster() {
  local number=$1
  local cluster_name=$2
  local net=$3
  shift 3
  if [ -z "$NODE_KEYS" ]; then
    exit -1
  fi
  IFS=', ' read -r -a node_keys <<<$NODE_KEYS
  seed_host=$(docker-machine ip $cluster_name-0)
  rpc_address=$(docker-machine ssh $cluster_name-0 ifconfig eth1 | awk -F ' *|:' '/inet /{print $3}' | tr -d '\n')
  if [ -z "$rpc_address" ]; then
    rpc_address="127.0.0.1"
  fi

  start_starcoin $cluster_name-0 starcoin-0 9840 9101 $net --node-key ${node_keys[0]} -s full --rpc-address $rpc_address --disable-seed
  sleep 5
  seed_peer_id=$(docker-machine ssh $cluster_name-0 grep 'Local\ node\ identity\ is:\ ' $cfg_root/starcoin-0/$net/starcoin.log | awk '{print $8}' | tac | head -n 1)
  seed=/ip4/$seed_host/tcp/9840/p2p/$seed_peer_id
  for ((c = 1; c < $number; c++)); do
    rpc_address=$(docker-machine ssh $cluster_name-$c ifconfig eth1 | awk -F ' *|:' '/inet /{print $3}' | tr -d '\n')
    if [ -z "$rpc_address" ]; then
      rpc_address="127.0.0.1"
    fi
    start_starcoin $cluster_name-$c starcoin-$c 9840 9101 $net --seed $seed -s full --node-key ${node_keys[$c]} --rpc-address $rpc_address

  done
  start_txfactory $cluster_name-0 starcoin-0 txfactory-0 $net
}

usage() {
  echo "Usage $(basename $0)  [number, cluster_name, network]"
  exit -1
}

if [ $# -lt 3 ]; then
  usage
fi

start_cluster $1 $2 $3
