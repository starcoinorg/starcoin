Feature: starcoin integration
  Background:
    Given a storage
    Given a node config

#
#  1. node：
#
#  - [ ] single startup
#  - [ ] Multiple startup
#  - [ ] restart
#  - [ ] stop
  Scenario: Node start
    Given a node handle
    Given a rpc client
    Then get node info
    Then get node status
    Then get node peers
    Then node handle stop

#
#  2. sync:
#
#  - [ ] basic
#  - [ ] no data
#  - [ ] partial data
#  - [ ] full node
#  - [ ] fast sync



#  3. transaction:
#
#  - [ ] user transaction
#  - [ ] block meterdata
#  - [ ] mint
#  - [ ] transfer



#  4.  genesis:
#
#  - [ ] generate
#  - [ ] check


#  5. VM:
#
#  - [ ] script
#  - [ ] module

