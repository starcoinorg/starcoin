Feature: cmd integration test
  Background:
    Given a storage
#
#  1. node：
  Scenario: cmd
    Given cmd context
    Given [cmd] node info

