#!/bin/bash

cfg_path="/.starcoin"
net="$1"
shift;
rm -f $cfg_path/$net/starcoin.ipc &>/dev/null
rm -f $cfg_path/$net/peers.json &>/dev/null
./starcoin -d /.starcoin -n $net $@
