# Starcoin Stratum Protocol User Guide

## Introduction
This document explains how to use the Stratum protocol integration in Starcoin nodes for mining STC with `starcoin_miner` or `stcbox`.

### Stratum Protocol Implementation
Starcoin nodes feature a Stratum protocol implementation that includes a basic difficulty manager,which calculates suitable difficulties and dispatches mining sub-jobs to miners based on their hash rate.

### Mining Configurations
### Solo Mining
- **Mode**: Enables miners to independently connect and mine directly on the Starcoin network. This mode suits miners who prefer managing their own operations without the involvement of an intermediary pool.

### Public Mining Pool
- **Setup Requirement**: While the system provides the tools necessary for job distribution and difficulty management, it does not include a reward distribution system.
  - **Authentication Module**: Required for establishing a secure public mining pool.
  - **Reward Distribution Implementation**: Operators need to integrate their own system to distribute rewards. Common methods are:
    - **PPS (Pay-per-Share)**: Miners are paid a fixed rate for each share they submit.
    - **PPLNS (Pay-per-Last-N-Shares)**: Rewards are based on the shares contributed in the last N shares window.

### Conclusion
The integration of the Stratum protocol in Starcoin nodes supports efficient mining through dynamic difficulty adjustments. However, those wishing to run a public mining pool will need to implement additional features for complete functionality.

## Operation guide
### Enable stratum server in starcoin

Starcoin Node Command Settings for Enabling Stratum
+ --disable-stratum

	This option disables the stratum server. default is false.

+ --stratum-address <ADDRESS> 

	This option sets the IP address or hostname where the stratum server should listen for connections. default is 0.0.0.0

+ --stratum-port <stratum-port>

	This option sets the port number on which the stratum server will listen for incoming connections.  default is 9880

### Mining with stcbox

Configure the miner pool in stcbox:
```
<user-name>@stratum+tcp://<stratum-address>:<stratum-port>
```
### Mining with cpu by starcoin_miner (for testing)

```
USAGE:
    starcoin_miner [OPTIONS] --user <USER>
OPTIONS:
    -a, --server <SERVER>              [default: 127.0.0.1:9880]
    -n, --thread-num <THREAD_NUM>      [default: 1]
    -u, --user <USER>
```
