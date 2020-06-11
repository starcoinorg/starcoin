#!/bin/bash
set -x
cfg_path="/.starcoin"
net="$1"
shift;
rm -f $cfg_path/$net/starcoin.ipc &>/dev/null
rm -f $cfg_path/$net/peers.json &>/dev/null
./starcoin -d /.starcoin -n $net $@
ret=$?
if [ $ret -eq 120 ] | [ $ret -eq 134 ]; then
    echo "Start failed with gensis mismatch code 120 or 134, clean data..."
    rm -rf $cfg_path/$net/ &>/dev/null
fi
exit $ret
