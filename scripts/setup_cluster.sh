#!/bin/bash
# use for init cluster

cfg_root=/mnt/volume_01/starcoin_cfg

create_node(){
    local node=$1
    local access_token=$2
    docker-machine  create --driver digitalocean --digitalocean-region sgp1 --digitalocean-image "ubuntu-18-04-x64" --digitalocean-size "c-4" --digitalocean-access-token  $access_token $node
}

create_nodes(){
    local cluster_name=$1
    local num=$2
    local token=$3
    for((c=0; c<$num; c++));do
	create_node $cluster_name-$c $token
    done
}

stop_nodes(){
    local cluster_name=$1
    local num=$2
    for((c=0; c<$num;c++));do
	docker-machine stop $cluster_name-$c
    done
}

remove_nodes(){
    local cluster_name=$1
    local num=$2
    for((c=0; c<$num;c++));do
	docker-machine rm $cluster_name-$c -f -y	
    done
}

clean_data(){
    local cluster_name=$1
    local num=$2
    for((c=0; c<$num;c++));do
	docker-machine ssh $cluster_name-$c rm -rf $cfg_root	
    done
}

clean_cfg(){
    local cluster_name=$1
    local num=$2
    for((c=0; c<$num;c++));do
	docker-machine ssh $cluster_name-$c rm -rf $cfg_root/starcoin-$c/*/config.toml
    done
    
}

usage(){
    echo "Usage $(basename $0)  [stop, start, clean_node, clean_cfg] cluster_name nodes_number [access_token]"
    exit -1
}


if [ $# -lt 2 ]; then
    usage;
fi

case $1 in
    start)
	shift;
	create_nodes $@
	;;
    stop)
	shift;
	remove_nodes $@
	;;
    clean_data)
	shift;
	clean_data $@
	;;
    clean_cfg)
	shift;
	clean_cfg $@
esac
