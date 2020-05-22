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

  Scenario: sync status
    Given a node config
    And node handle
    And a rpc client
    Then basic check
    Then node stop


#  4.  genesis:
#
#  - [ ] generate
#  - [ ] check


#  5. VM:
#
#  - [ ] script
#  - [ ] module

