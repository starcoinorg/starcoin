#!/bin/bash

cfg_path="/.starcoin"
net="halley"
rm -f $cfg_path/$net/starcoin.ipc &>/dev/null
rm -f $cfg_path/$net/peers.json &>/dev/null
./starcoin -d /.starcoin -n halley -s full $@
