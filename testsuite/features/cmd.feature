Feature: cmd integration test
  Background:
    Given a dev node config
    And node dev handle
    And dev rpc client
    Given cli state
#
    #  1. node info
  Scenario: [cmd] node info
    Then [cmd] node info
    Then node handle stop
    #  2. node peers
  Scenario: [cmd] node peers
    Then [cmd] node peers
    Then node handle stop
    #  3. wallet list
  Scenario: [cmd] wallet list
    Then [cmd] wallet list
    Then node handle stop
    #  4. wallet show
  Scenario: [cmd] wallet show
    Then [cmd] wallet show
    Then node handle stop
  #  5. dev get coin
  Scenario Outline: [cmd] dev get coin
    Then dev get_coin "<amount>"
    Then node handle stop

    Examples:
      | amount |
      | 200000 |

  #  6. wallet create
  Scenario Outline: [cmd] wallet create
    Then wallet create "<password>"
    Then node handle stop

    Examples:
      | password |
      | ssyuan |
  #  7. wallet unlock
#  Scenario Outline: [cmd] wallet unlock
#    Then wallet unlock password:"<password>"
#
#    Examples:
#      | password |
#      |   |

  #  8. common cmd cli
  Scenario Outline: [cmd] cli
    Then cmd cli: "<cmd>"
    Then node handle stop

    Examples:
      | cmd |
      | wallet create -p dssss |
      | wallet show |


