#!/bin/bash
set -e
PORT=31440
nodes=$(kubectl get pod  -l app=starcoin --field-selector=status.phase=Running  --no-headers=true|awk '{print $1}')
addresses=$(for node in $nodes;do
	      kubectl exec  $node --stdin --tty -- /starcoin/starcoin -n proxima -d /sc-data/ -o json node info  |grep -v INFO |jq -r .ok.self_address
	    done)
exips=($(kubectl get service -o wide|grep LoadBalance|sort -k 1|awk '{print $4}'))

green=`tput setaf 2`
echo -e "${green} Starcoin nodes in this cluster:"

counter=0
for add in $addresses;do
    ip=${exips[$((counter++))]};
    echo "$add"|sed -e  "s/127.0.0.1/$ip/"
done
