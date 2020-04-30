#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

SEED_NODE_KEY=a52cb7fe64b154d192cebd35a0b129c80481f89dd94f2aa2978b71417304a858
SEED_PORT=9840
SEED_HOST=$(docker-machine ip starcoin-0)
SEED=/ip4/$SEED_HOST/tcp/$SEED_PORT/p2p/QmcejhDq4ubxLnx7sNENECJroAuepMiL6Zkjp63LMmwVaT

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
  echo -e "*\n"'!'"starcoin""\n!txfactory" >$DIR/../target/debug/.dockerignore
  docker build -f $DIR/DockerfileCi -t starcoin:latest $DIR/../target/debug/
  check_errs $? "Docker build error"
}

function start_starcoin() {
  local host_name=$1
  local name=$2
  local port=$3
  local m_port=$4
  shift 4
  eval $(docker-machine env $host_name)
  docker_rebuild
  docker rm -f $name 1>/dev/null
  docker-machine ssh $host_name rm -f $cfg_root/$name/*/starcoin.ipc
  docker run -td --restart=on-failure:1 -p $port:9840 -p $m_port:9101 -v $cfg_root/$name:/.starcoin --name $name starcoin -d /.starcoin $@
  check_errs $? "Docker run starcoin error"
}

function start_txfactory() {
  local starcoin_name=$1
  local name=$2
  shift 2
  docker_rebuild
  docker rm -f $name 1>/dev/null
  # docker-machine ssh $host_name rm -f $cfg_root/$name/*/starcoin.ipc
  docker run -td --restart=on-failure:1 -v $cfg_root/$starcoin_name:/.starcoin --name $name --entrypoint "/starcoin/txfactory" starcoin:latest --ipc-path /.starcoin/halley/starcoin.ipc $@
  check_errs $? "Docker run txfactory error"
}

function start_halley_seed() {
  start_starcoin $1 $2 $3 $4 -n halley -s full --node-key $SEED_NODE_KEY
}

function start_halley_node() {
  start_starcoin $1 $2 $3 $4 -n halley -s full --seed $SEED
}

#TODO: start failed, clean all env and restart
start_halley_seed starcoin-0 starcoin-0 $SEED_PORT 9101
start_halley_node starcoin-0 starcoin-1 9841 9102
start_halley_node starcoin-0 starcoin-2 9842 9103
start_txfactory starcoin-0 txfactory-0
