Feature: integration
  Background:
    Given a storage

#
#  1. node：
  Scenario: [cmd] node start and execute transfer
    Given a test node config
    And node handle
    And ipc rpc client
    Then get node info
    Then get node status
    Then get node peers
    Given an account
    And default account
    Then charge money to account
    Then state proof
    Then stop

#  Scenario: Node start and execute transfer
#    Given remote rpc client
#    Then get node info
#    Then get node status
#    Then get node peers
#    Given an account
#    And default account
#    Then charge money to account
#    Then execute transfer transaction
#    Then state proof

#
#  2. sync:
#
#  - [ ] basic
#  - [ ] no data
#  - [ ] partial data
#  - [ ] full node
#  - [ ] fast sync

#  Scenario: sync status
#    Given a node config
#    And node handle
#    And sync rpc client
#    And remote rpc client
#    Then basic check
#    Then node stop

#  4.  genesis:
#
#  - [ ] generate
#  - [ ] check


#  5. VM:
#
#  - [ ] script
#  - [ ] module

