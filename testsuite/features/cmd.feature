Feature: cmd integration test
  Background:
    Given a dev node config
    And node handle
    And ipc rpc client

# chain
  Scenario Outline: [cmd] cli chain test
    Then cmd: "chain epoch-info"
    Then cmd: "chain list-block"
    Then cmd: "chain get-block @$[0].block_hash@"
    Then cmd: "chain list-block"
    Then cmd: "chain get-block @$[0].number@"
    Then cmd: "chain get-txn-infos @$.header.block_hash@"
    Then cmd: "chain get-txn @$[0].transaction_hash@"
    Then cmd: "chain get-events @$.transaction_hash@"
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

# node service
  Scenario Outline: [cmd] node service test
    Then cmd: "node service list"
    Then cmd: "node service stop starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then cmd: "node service start starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then stop

    Examples:
      |  |

# multisig account
  Scenario Outline: [cmd] multisig account
    Then cmd: "account unlock"
    Then cmd: "dev get-coin"
    Then cmd: "account generate-keypair -c 3"
    Then cmd: "account derive-address -t 2 -p @$[0].public_key@ -p @$[1].public_key@ -p @$[2].public_key@"
    Then cmd: "account transfer --blocking -r @$.receipt_identifier@ -t 0x1::STC::STC -v 10000000"
    Then stop

    Examples:
      | para |
      | x@$.auth_key@  |

 #dev
  Scenario Outline: [cmd] dev test
    Then cmd: "account unlock -d 30000 0x0000000000000000000000000A550C18"
    Then stop

    Examples:
      |  |

#state
  Scenario Outline: [cmd] state test
    Then cmd: "state get-root"
    Then cmd: "dev get-coin"
    Then cmd: "account show"
    Then cmd: "state get-proof @$.account.address@/1/0x1::Account::Account"
    Then cmd: "account show"
    Then cmd: "state get resource @$.account.address@ 0x1::Account::Account"
    Then assert: "$.json.sequence_number 0 "
    Then cmd: "account show"
    Then cmd: "state list resource @$.account.address@"
    Then cmd: "account show"
    Then cmd: "state list code @$.account.address@"
    Then stop

    Examples:
      |  |

#account
  Scenario Outline: [cmd] account test
    Then cmd: "account show"
    Then cmd: "account unlock"
    Then cmd: "dev get-coin"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -r @$.address@ -k @$.public_key@"
    Then cmd: "chain get-txn @$.txn_hash@"
    Then cmd: "chain get-events @$.transaction_hash@"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -r @$.receipt_identifier@"
    Then cmd: "chain get-txn @$.txn_hash@"
    Then cmd: "chain get-events @$.transaction_hash@"
    Then cmd: "account create -p compat"
    Then cmd: "account unlock -p compat @$.address@"
    Then cmd: "account show @$.address@"
    Then cmd: "account export -p compat @$.account.address@"
    Then cmd: "account create -p test"
    Then cmd: "account unlock -p test @$.address@"
    Then cmd: "account change-password @$.address@ -p hello"
    Then cmd: "account remove @$.address@ -p hello"
    Then cmd: "account generate-keypair"
    Then cmd: "account import-readonly -i @$[0].public_key@"
    Then cmd: "account remove @$.address@"
    Then cmd: "account list"
    Then cmd: "account show"
    Then cmd: "account accept-token 0x1::DummyToken::DummyToken"
    Then stop


    Examples:
      |  |

#account sign message
  Scenario Outline: [cmd] account sign message
    # test the account do not exist on chain
    Then cmd: "account unlock"
    Then cmd: "account sign-message -m helloworld"
    Then cmd: "account verify-sign-message -m @$.result@"
    Then assert: "$.result true"
    # create the account on chain
    Then cmd: "dev get-coin"
    Then cmd: "account sign-message  -m helloworld"
    Then cmd: "account verify-sign-message -m @$.result@"
    Then assert: "$.result true"
    # init the auth key on chain by send the first transaction, test authkey is not dummy key.
    Then cmd: "account transfer -v 1000 -r 0xA550C18 -b"
    Then cmd: "account sign-message -m helloworld"
    Then cmd: "account verify-sign-message -m @$.result@"
    Then assert: "$.result true"
    # test multi sign account
    Then cmd: "account sign-message -s 0xA550C18 -m helloworld"
    Then cmd: "account verify-sign-message -m @$.result@"
    Then assert: "$.result true"

    Examples:
      |  |

#mytoken
  Scenario Outline: [cmd] my_token test
    Then cmd: "account show"
    Then cmd: "account unlock @$.account.address@"
    Then cmd: "dev get-coin"
    Then cmd: "account show"
    Then cmd: "dev compile ../examples/my_token/MyToken.move -o ../examples -s @$.account.address@"
    Then cmd: "dev deploy --blocking @$[0]@"
    Then cmd: "account show"
    Then cmd: "account execute-function --function @$.account.address@::MyToken::init --blocking"
    Then cmd: "chain get-txn @$.txn_hash@"
    Then cmd: "account show"
    Then cmd: "account execute-function --function @$.account.address@::MyToken::mint --blocking --arg 1000000u128"
#    Then assert: "$.status Executed"
    Then cmd: "chain get-txn @$.txn_hash@"
# TODO check MyToken balance.
#    Then cmd: "account show"
#    Then assert: "$.account.balances.MyToken 1000000"
    Then stop

    Examples:
      |  |
