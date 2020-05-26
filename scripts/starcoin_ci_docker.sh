#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

NODE_KEYS=("a52cb7fe64b154d192cebd35a0b129c80481f89dd94f2aa2978b71417304a858" "6a5d25da2a86dab3b46639aae7db33bc7d1fe2006c7c8a9fdf93e20775b8bc8d" "17f42931c760ce8d4611a8907e0e8325ce59569aa3e3eb207cb0eef596f44af6")

SEED_PORT=9840
SEED_METRICS_PORT=9101

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
  docker run -td --log-opt mode=non-blocking --log-opt max-size=1m --restart=on-failure:10 -p $port:9840 -p $m_port:9101 -v $cfg_root/$name:/.starcoin --name $name starcoin $net $@
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

function start_cluster(){
    local number=$1
    local cluster_name=$2;
    local net=$3
    shift 3;
    SEED_HOST=$(docker-machine ip $cluster_name-0)
    SEED=/ip4/$SEED_HOST/tcp/$SEED_PORT/p2p/QmcejhDq4ubxLnx7sNENECJroAuepMiL6Zkjp63LMmwVaT
    
    start_starcoin $cluster_name-0 starcoin-0 $SEED_PORT $SEED_METRICS_PORT $net --node-key ${NODE_KEYS[0]} -s full
    for((c=1; c<$number;c++));do
	start_starcoin $cluster_name-$c starcoin-$c $SEED_PORT $SEED_METRICS_PORT $net --seed $SEED -s full --node-key ${NODE_KEYS[$c]}
    done
    start_txfactory $cluster_name-0 starcoin-0 txfactory-0 $net
}

usage(){
    echo "Usage $(basename $0)  [number, cluster_name, network]"
    exit -1
}

if [ $# -lt 3 ]; then
    usage;
fi

start_cluster $1 $2 $3
