Feature: compat cmd integration test
  Background:
    Given a dev node config
    And node dev handle
    And dev rpc client
#    Given remote rpc client


  Scenario Outline: [COMPAT] cli chain test
    Then cmd: "chain branches $.None"
    Then cmd: "chain epoch_info $.None"
    Then cmd: "chain get_block_by_number $.None"
    Then cmd: "chain list_block $[0].id"
    Then cmd: "chain get_block $.id"
    Then cmd: "chain get_txn_by_block $[0].transaction_hash"
    Then cmd: "chain get_txn $.txn_info_id"
    Then cmd: "chain get_events $.None"

    Examples:
      |  |

# debug
  Scenario Outline: [COMPAT] debug test
    Then cmd: "chain show $.head_block"
    Then cmd: "debug gen_dev_block -p $.None"
    Then cmd: "wallet unlock $.None"
    Then cmd: "dev get_coin $.None"
    Then cmd: "debug gen_txn -r -v 10 $.None"
    Then cmd: "debug log level Debug $.None"

    Examples:
      |  |



# node
  Scenario Outline: [COMPAT] node test
    Then cmd: "node metrics $.None"
    Then cmd: "node info $.None"
    Then cmd: "node peers $.None"

    Examples:
      |  |

#state
  Scenario Outline: [COMPAT] state test
    Then cmd: "state get_root $.None"
    Then cmd: "dev get_coin $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "state get_proof $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "state get_account $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "state get $.None"

    Examples:
      |  |

#wallet
  Scenario Outline: [COMPAT] wallet test
    Then cmd: "wallet show $.None"
    Then cmd: "wallet unlock $.None"
    Then cmd: "dev get_coin $.None"
    Then cmd: "wallet create -p transfer $-r$.address $-k$.public_key"
    Then cmd: "wallet transfer -v 10000 $.address"
    Then cmd: "wallet create -p compat $.address"
    Then cmd: "wallet unlock -p compat $.result"
    Then cmd: "wallet show $.account.address"
    Then cmd: "wallet export -p compat $.None"
    Then cmd: "wallet list $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "wallet execute-builtin --blocking --script empty_script -s $.None"
    Then cmd: "wallet accept_token 0x1::STC::STC $.None"


    Examples:
      |  |


#mytoken
  Scenario Outline: [COMPAT] my_token test
    Then cmd: "wallet show $.account.address"
    Then cmd: "wallet unlock $.None"
    Then cmd: "dev get_coin $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "dev compile ../examples/my_token/module/MyToken.move -o ../examples -s $.result"
    Then cmd: "dev deploy --blocking $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "dev compile ../examples/my_token/scripts/init.move -d ../examples/my_token/module/MyToken.move -o ../examples -s $.result"
    Then cmd: "dev execute --blocking $.txn_hash"
    Then cmd: "chain get_txn $.None"
    Then cmd: "wallet show $.account.address"
    Then cmd: "dev compile ../examples/my_token/scripts/mint.move -d ../examples/my_token/module/MyToken.move -o ../examples -s $.result"
    Then cmd: "dev execute --arg 1000000u128 --blocking $.txn_hash"
    Then cmd: "chain get_txn $.None"

    Examples:
      |  |
