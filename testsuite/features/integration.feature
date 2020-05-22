Feature: starcoin integration
  Background:
    Given a storage

#
#  1. node：
  Scenario Outline: Node start and execute transfer
    Given ipc file config "<conf>"
    And a rpc client
    Then get node info
    Then get node status
    Then get node peers
    Given an account
    And default account
    Then charge money to account
    Then execute transfer transaction
    Then state proof

    Examples:
      | conf |
      | /Volumes/jiayi/project/rust/starcoin/conf/dev/starcoin.ipc |
#
#  2. sync:
#
#  - [ ] basic
#  - [ ] no data
#  - [ ] partial data
#  - [ ] full node
#  - [ ] fast sync

  Scenario Outline: sync status
    Given sync network config "<conf>" "<seed>"
    Given a node config
    And node handle
    And local rpc client
    And remote rpc client
    Then basic check
    Then node stop

    Examples:
      | conf | seed |
      | /Volumes/jiayi/project/rust/starcoin/conf/dev/starcoin.ipc | /ip4/127.0.0.1/tcp/59753/p2p/12D3KooWMLGtRBKR31BpSdAxNHe8Qwv2rGiQUJpVuFFFoNTejq79 |

#  4.  genesis:
#
#  - [ ] generate
#  - [ ] check


#  5. VM:
#
#  - [ ] script
#  - [ ] module

