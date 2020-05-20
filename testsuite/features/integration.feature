Feature: starcoin integration
  Background:
    Given a storage

  Scenario: Node start
    Given a node config
    Given a storage
    Given a node handle
    Then node handle stop

