#!/bin/bash

DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

SEED_NODE_KEY=a52cb7fe64b154d192cebd35a0b129c80481f89dd94f2aa2978b71417304a858
SEED_PORT=8840
SEED_HOST=206.189.89.104
SEED=/ip4/$SEED_HOST/tcp/$SEED_PORT/p2p/QmcejhDq4ubxLnx7sNENECJroAuepMiL6Zkjp63LMmwVaT

cfg_root=/root/workspaces/starcoin_cfg

function docker_rebuild(){
    echo  -e "*\n"'!'"starcoin"> $DIR/../target/debug/.dockerignore
    docker build -f $DIR/DockerfileCi -t starcoin:latest  $DIR/../target/debug/
}

function start_starcoin(){
    local name=$1
    docker rm -f $name 1>/dev/null
    local port=$2
    shift 2;
    rm $cfg_root/$name/*/starcoin.ipc &>/dev/null
    docker run -td --restart=on-failure:10 -p $port:9840 -v $cfg_root/$name:/.starcoin --name $name starcoin -d /.starcoin $@
}

function start_halley_seed(){
    start_starcoin $1 $2 -n halley -s full --node-key $SEED_NODE_KEY
}

function start_halley_node(){
    start_starcoin $1 $2 -n halley -s full --seed $SEED
}

function clean_all(){
    rm -rf /root/workspaces/starcoin_cfg
}

docker_rebuild
#TODO: start failed, clean all env and restart
start_halley_seed starcoin-0 $SEED_PORT
start_halley_node starcoin-1 8841
start_halley_node starcoin-2 8842
