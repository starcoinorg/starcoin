#!/bin/bash
PORT=31440
nodes=$(kubectl get pod  -l app=starcoin --field-selector=status.phase=Running  --no-headers=true|awk '{print $1}')
addresses=$(for node in $nodes;do
	      kubectl exec  $node --stdin --tty -- /starcoin/starcoin -n proxima -d /data/ -o json node info  |grep -v INFO |jq -r .ok.self_address
	    done)
f1=$(mktemp)
f2=$(mktemp)
kubectl get node -o wide --no-headers=true | awk '{print $1,$7}'| sort >f1
kubectl get pod -o wide --no-headers=true | awk '{print $7,$1}' | sort >f2
exips=($(join f1 f2 |awk '{print $2}'));rm f1 f2

green=`tput setaf 2`
echo -e "${green} Starcoin nodes in this cluster:"

counter=0
for add in $addresses;do
    ip=${exips[$((counter++))]};
    echo "$add"|sed -e  "s/127.0.0.1/$ip/"|sed -e  "s/9840/$PORT/"
done


