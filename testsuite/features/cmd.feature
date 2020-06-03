Feature: cmd integration test
  Background:
    Given a storage
#
#  1. node：
  Scenario Outline: cmd
    Given cmd context
    Then [cmd] node info
    Then [cmd] wallet list
    Then [cmd] wallet show
    Then dev get_coin "<amount>"
    Then wallet create "<password>"
    Then wallet unlock password:"<password>"

    Examples:
      | amount | password|
      | 20000000 | sfsd333 |
