#!/bin/bash
# use for init cluster
ACCESS_TOKEN=EMPTY_TOKEN
create_node(){
    local node=$1
    #Todo: remove the token
    docker-machine  create --driver digitalocean --digitalocean-region sgp1 --digitalocean-image "ubuntu-18-04-x64" --digitalocean-size "c-2" --digitalocean-access-token  $ACCESS_TOKEN $node
}

create_nodes(){
    for((c=0; c<$1; c++));do
	create_node starcoin-$c
    done
}

stop_nodes(){
    for((c=0; c<$1;c++));do
	docker-machine stop starcoin-$c
    done
}

remove_nodes(){
    for((c=0; c<$1;c++));do
	docker-machine rm starcoin-$c -f -y
    done
}

usage(){
    echo "Usage $(basename $0) [stop, start] nodes_number"
}


if [ $# -lt 2 ]; then
    usage;
fi

case $"$1" in
    start)
	shift;
	remove_nodes $@
	create_nodes $@
	;;
    stop)
	shift;
	remove_nodes $@
	;;
esac
