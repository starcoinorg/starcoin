Feature: compat cmd integration test
  Background:
    Given compat node1 rpc client
    Given compat node2 rpc client

#    Given remote rpc client

  # node
  Scenario Outline: [compat] basic test
    Then compat basic check


    Examples:
      |  |

