#!/bin/bash

# rand function
function rand(){
    min=$1
    max=$(($2-$min+1))
    num=$(cat /dev/urandom | head -n 10 | cksum | awk -F ' ' '{print $1}')
    echo $(($num%$max+$min))
}
# get last header
#current_number=`~/kubectl --kubeconfig ~/.kube/starcoin_config -n starcoin-centauri exec starcoin-stress-2 -c starcoin --stdin --tty -- /starcoin/starcoin -c /sc-data/centauri/starcoin.ipc -o json chain info |grep -v INFO |jq .ok.head.number | sed 's/"//g'`
current_number=`target/debug/starcoin -c data/centauri/starcoin.ipc -ojson chain info |grep -v INFO |jq .ok.head.number | sed 's/"//g'`
echo "current number: $current_number"
rnd_block_num=$(rand 1 current_number)

echo "chain verify node..."
target/debug/starcoin -c data/centauri/starcoin.ipc -ojson chain verify node
echo "verify node ok!"

echo "chain verify block..."
target/debug/starcoin -c data/centauri/starcoin.ipc -ojson chain verify block -n $rnd_block_num
echo "verify block ok!"

echo "chain verify epoch..."
target/debug/starcoin -c data/centauri/starcoin.ipc -ojson chain verify epoch -n $rnd_block_num
echo "verify epoch ok!"

#while [ `echo ${TEMP} | awk -v tem=$block_num '{print(current_number>tem)? "1":"0"}'` -eq "0" ]
#do
#    echo $block_num
##    ~/kubectl --kubeconfig ~/.kube/starcoin_config -n starcoin-centauri exec starcoin-stress-2 -c starcoin --stdin --tty -- /starcoin/starcoin -c /sc-data/centauri/starcoin.ipc -o json chain verify epoch -n $block_num
#    target/debug/starcoin -c data/centauri/starcoin.ipc -ojson chain verify epoch -n $block_num
#    block_num=$((block_num+240))
#done





