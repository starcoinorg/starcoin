Feature: cmd integration test
  Background:
    Given a storage
#
#  1. node：
  Scenario: cmd
    Given cmd context
    Then [cmd] node info
    Then [cmd] wallet list
    Then [cmd] wallet show
    Then [cmd] dev get_coin
    Then [cmd] wallet create
