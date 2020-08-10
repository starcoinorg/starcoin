Feature: compat cmd integration test
  Background:
    Given compat node1 rpc client
    Given compat node2 rpc client

#    Given remote rpc client

  # node
  Scenario Outline: [compat] transfer test
    Then cmd: "account show"
    Then cmd: "account unlock -d 30000 0000000000000000000000000a550c18"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -s 0000000000000000000000000a550c18 -r @$.address@ -k @$.public_key@"
#    Then cmd: "chain get_block @$.block_id@"
    Then transfer txn block check

    Examples:
      |  |

