Feature: cmd integration test
  Background:
    Given a test node config
    And node handle
    And ipc rpc client

# chain
  Scenario Outline: [cmd] cli chain test
    Then cmd: "chain branches"
    Then cmd: "chain epoch_info"
    Then cmd: "chain get_block_by_number"
    Then cmd: "chain list_block"
    Then cmd: "chain get_block @$[0].id@"
    Then cmd: "chain get_txn_by_block @$.id@"
    Then cmd: "chain get_txn @$[0].transaction_hash@"
    Then cmd: "chain get_events @$.txn_info_id@"
    Then stop

    Examples:
      |  |

# debug
  Scenario Outline: [cmd] debug test
    Then cmd: "chain show"
#    Then cmd: "debug gen_dev_block -p $.head_block"
    Then cmd: "account unlock"
    Then cmd: "dev get_coin"
    Then cmd: "debug gen_txn -r -v 10"
#    Then cmd: "debug log level Debug"
    Then stop

    Examples:
      |  |

# node
  Scenario Outline: [cmd] node test
    Then cmd: "node metrics"
    Then cmd: "node info"
    Then cmd: "node peers"
    Then stop

    Examples:
      |  |

# multisig account
  Scenario Outline: [cmd] multisig account
    Then cmd: "account unlock"
    Then cmd: "dev get_coin"
    Then cmd: "account create -p 111"
    Then cmd: "account create -p 222"
    Then cmd: "account list"
    Then cmd: "dev derive-address -t 2 -p @$[0].public_key@ -p @$[1].public_key@ -p @$[2].public_key@"
    Then cmd: "account execute-builtin --blocking --script create_account --type_tag 0x01::STC::STC --arg 0x@$.address@ --arg <para> --arg 10000000u128"
    Then stop

    Examples:
      | para |
      | x@$.auth_key_prefix@  |

 #dev
  Scenario Outline: [cmd] dev test
    Then cmd: "account unlock -d 30000 0000000000000000000000000a550c18"
    Then cmd: "dev upgrade_stdlib --blocking"
    Then stop

    Examples:
      |  |

#state
  Scenario Outline: [cmd] state test
    Then cmd: "state get_root"
    Then cmd: "dev get_coin"
    Then assert: "$.gas_unit_price 1 $.sequence_number 0 $.sender 0000000000000000000000000a550c18"
    Then cmd: "account show"
    Then assert: "$.account.is_default true $.sequence_number 0"
    Then cmd: "state get_proof @$.account.address@"
    Then cmd: "account show"
    Then cmd: "state get -a @$.account.address@"
    Then cmd: "account show"
    Then cmd: "state get -a @$.account.address@ 0x1::Account::Account"
    Then stop

    Examples:
      |  |

#account
  Scenario Outline: [cmd] account test
    Then cmd: "account show"
    Then cmd: "account unlock"
    Then cmd: "dev get_coin"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -r @$.address@ -k @$.public_key@"
    Then cmd: "chain get_txn @$.txn_hash@"
    Then cmd: "chain get_events @$.transaction_hash@"
    Then cmd: "account create -p compat"
    Then cmd: "account unlock -p compat @$.address@"
    Then cmd: "account show @$.result@"
    Then cmd: "account export -p compat @$.account.address@"
    Then cmd: "account list"
    Then cmd: "account show"
    Then cmd: "account execute-builtin --blocking --script empty_script -s @$.account.address@"
    Then cmd: "account accept_token 0x1::DummyToken::DummyToken"
    Then stop


    Examples:
      |  |

#mytoken
  Scenario Outline: [cmd] my_token test
    Then cmd: "account show"
    Then cmd: "account unlock @$.account.address@"
    Then cmd: "dev get_coin"
    Then cmd: "account show"
    Then cmd: "dev compile ../examples/my_token/module/MyToken.move -o ../examples -s @$.account.address@"
    Then cmd: "dev deploy --blocking @$.result@"
    Then cmd: "account show"
    Then cmd: "dev compile ../examples/my_token/scripts/init.move -d ../examples/my_token/module/MyToken.move -o ../examples -s @$.account.address@"
    Then cmd: "dev execute --blocking @$.result@"
    Then cmd: "chain get_txn @$.txn_hash@"
    Then cmd: "account show"
    Then cmd: "dev compile ../examples/my_token/scripts/mint.move -d ../examples/my_token/module/MyToken.move -o ../examples -s @$.account.address@"
    Then cmd: "dev execute @$.result@ --blocking --arg 1000000u128"
#    Then assert: "$.status Executed"
    Then cmd: "chain get_txn @$.txn_hash@"
    Then stop

    Examples:
      |  |
