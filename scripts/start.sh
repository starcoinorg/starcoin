#!/bin/bash

cfg_path="/.starcoin"
net="$1"
shift;
rm -f $cfg_path/$net/starcoin.ipc &>/dev/null
rm -f $cfg_path/$net/peers.json &>/dev/null
./starcoin -d /.starcoin -n $net $@
ret=$?
if [ $ret -ne 120 ]; then
    echo "Start failed with gensis mismatch code 120,clean data..."
    rm -rf $cfg_path/$net/ &>/dev/null
    exit $ret
fi
