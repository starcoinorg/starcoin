Feature: cmd integration test
  Background:
    Given a dev node config
    And node handle
    And dev rpc client
    Given cli state
#
    #  1. node info
  Scenario: [cmd] node info
    Then [cmd] node info
    #  2. node peers
  Scenario: [cmd] node peers
    Then [cmd] node peers
    #  3. wallet list
  Scenario: [cmd] wallet list
    Then [cmd] wallet list
    #  4. wallet show
  Scenario: [cmd] wallet show
    Then [cmd] wallet show
  #  5. dev get coin
  Scenario Outline: [cmd] dev get coin
    Then dev get_coin "<amount>"

    Examples:
      | amount |
      | 200000 |

  #  6. wallet create
  Scenario Outline: [cmd] wallet create
    Then wallet create "<password>"

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

    Examples:
      | cmd |
      | wallet create -p dssss |
      | wallet show |


