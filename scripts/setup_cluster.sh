#!/bin/bash
# use for init cluster

cfg_root=/mnt/volume_01/starcoin_cfg

create_node(){
    local node=$1
    local access_token=$2
    docker-machine  create --driver digitalocean --digitalocean-region sgp1 --digitalocean-image "ubuntu-18-04-x64" --digitalocean-size "c-4" --digitalocean-access-token  $access_token $node
}

create_nodes(){
    for((c=0; c<$1; c++));do
	create_node starcoin-node-$c $2
    done
}

stop_nodes(){
    for((c=0; c<$1;c++));do
	docker-machine stop starcoin-node-$c
    done
}

remove_nodes(){
    for((c=0; c<$1;c++));do
	docker-machine rm starcoin-node-$c -f -y
    done
}

clean_data(){
    for((c=0; c<$1;c++));do
	docker-machine ssh starcoin-node-$c rm -rf $cfg_root
    done
}

clean_cfg(){
    for((c=0; c<$1;c++));do
	docker-machine ssh starcoin-node-$c rm -rf $cfg_root/starcoin-$c/halley/config.toml
    done
    
}


usage(){
    echo "Usage $(basename $0)  [stop, start, clean_node, clean_cfg] nodes_number [access_token]"
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
