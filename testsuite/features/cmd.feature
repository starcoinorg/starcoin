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
    Then cmd: "chain get-txn-info-list -s 0 -c 5"
    Then cmd: "chain list-block"
    Then cmd: "chain get-block-info @$[0].number@"
    Then cmd: "chain list-block"
    Then cmd: "chain get-txn-proof --block-hash @$[0].block_hash@ --transaction-global-index 0"
    Then cmd: "chain list-block"
    Then cmd: "chain get-txn-proof --block-hash @$[0].block_hash@ --transaction-global-index 0 --raw"
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
    Then cmd: "node service check starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then assert: "$ Stopped"
    Then cmd: "node service start starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then cmd: "node service check starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then assert: "$ Started"
    #ensure some service start successful.
    Then cmd: "node service check starcoin_rpc_server::service::RpcService"
    Then assert: "$ Started"
    Then cmd: "node service check starcoin_node::metrics::MetricsServerActorService"
    Then assert: "$ Started"
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
    # account for testing only
    Then cmd: "account import-multisig --pubkey 0x934e8a5a557229f90be7c95ec17fab84e64dcc3cf2dc934ff83ffc0915fad13e --pubkey 0x28358c05692e6758ba1398835525687c16d65abc9e1dc89023b46298ed2c575a --prikey 0x0c84a983ff0bfab39570c2ceed3e1c1feb645e84eccf9fd6baf4f49351a52329 --prikey 0x3695d6e08e3ad41962cba8c55ebb0552827807ae4cd6236d35195c769437272e -t 2"
    Then cmd: "account unlock"
    Then cmd: "account transfer --blocking -r 0x056d9bed868f8e8c74cf19109a2b375a -v 200000000"
    Then cmd: "account unlock 0x056d9bed868f8e8c74cf19109a2b375a"
    # enough signatures, submit directly
    Then cmd: "account transfer -s 0x056d9bed868f8e8c74cf19109a2b375a -r 0x056d9bed868f8e8c74cf19109a2b375b -v 10000000 -b"
    Then cmd: "account unlock 0x056d9bed868f8e8c74cf19109a2b375a"
    # sign to file first
    Then cmd: "account sign-multisig-txn -s 0x056d9bed868f8e8c74cf19109a2b375a --function 0x1::TransferScripts::peer_to_peer_v2 -t 0x1::STC::STC --arg 0x991c2f856a1e32985d9793b449c0f9d3 --arg 1000000u128 --output-dir /tmp"
    Then cmd: "account submit-txn @$@ -b"
    Then stop

    Examples:
      | para          |
      | x@$.auth_key@ |

  #dev
  Scenario Outline: [cmd] dev resolve test
    Then cmd: "dev resolve function 0x1::TransferScripts::peer_to_peer_v2"
    Then cmd: "dev resolve struct 0x1::Account::Account"
    Then cmd: "dev resolve module 0x1::Account"
    Then stop

    Examples:
      |  |

  Scenario Outline: [cmd] dev sleep test
    Then cmd: "dev get-coin"
    Then cmd: "dev sleep -t 864000000"
    #TODO support wait and add an assert for result.
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
    Then cmd: "state get-proof @$.account.address@/1/0x1::Account::Account --raw"
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
    Then cmd: "chain get-txn @$.execute_output.txn_hash@"
    Then cmd: "chain get-events @$.transaction_hash@"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -r @$.receipt_identifier@"
    Then cmd: "chain get-txn @$.execute_output.txn_hash@"
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
    Then cmd: "account generate-keypair"
    Then cmd: "account import -i @$[0].private_key@"
    Then cmd: "account remove @$.address@"
    Then cmd: "account list"
    Then cmd: "account unlock @$[0].address@"
    Then cmd: "account import -i 0x6afe92b85f5fd61b099d1bd805aa54b9737ad73522f490c47872cd028ea338f3"
    Then cmd: "account transfer --blocking -v 100000000 -r 0x809c795045105a7b1efbcca4510d2034"
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    # using a temporal private key as import
    Then cmd: "account rotate-authentication-key 0x809c795045105a7b1efbcca4510d2034 -i 0x3885e7dde8381046849d64d28b675f1c668dc36eaa9be11cbcaadb24c3917554"
    # rotate-authentication-key twice for:
    # 1. auth key will be verified on chain, so do it again for checking last rotation.
    # 2. ensuring it's idempotent
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    Then cmd: "account rotate-authentication-key 0x809c795045105a7b1efbcca4510d2034 -i 0x3885e7dde8381046849d64d28b675f1c668dc36eaa9be11cbcaadb24c3917554"
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    Then cmd: "account create -p rotate-transfer"
    # transfer after rotation
    Then cmd: "account transfer --blocking -v 10000000 -s 0x809c795045105a7b1efbcca4510d2034 -r @$.address@"
    Then cmd: "chain get-txn @$.execute_output.txn_hash@"
    Then cmd: "chain get-events @$.transaction_hash@"
    Then cmd: "account remove 0x809c795045105a7b1efbcca4510d2034"
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
    Then cmd: "account verify-sign-message -m @$.hex@"
    Then assert: "$.ok true"
    # create the account on chain
    Then cmd: "dev get-coin"
    Then cmd: "account sign-message  -m helloworld"
    Then cmd: "account verify-sign-message -m @$.hex@"
    Then assert: "$.ok true"
    # init the auth key on chain by send the first transaction, test authkey is not dummy key.
    Then cmd: "account transfer -v 1000 -r 0xA550C18 -b"
    Then cmd: "account sign-message -m helloworld"
    Then cmd: "account verify-sign-message -m @$.hex@"
    Then assert: "$.ok true"
    # test multi sign account
    Then cmd: "account sign-message -s 0xA550C18 -m helloworld"
    Then cmd: "account verify-sign-message -m @$.hex@"
    Then assert: "$.ok true"

    Examples:
      |  |

#mytoken
#  Scenario Outline: [cmd] my_token test
#    Then cmd: "account unlock 0x0000000000000000000000000a550c18"
#    Then cmd: "dev compile ../examples/my_token/MyToken.move -o ../examples -s 0x0000000000000000000000000a550c18"
#    Then cmd: "dev deploy --blocking @$[0]@"
#    Then cmd: "account execute-function -s 0x0000000000000000000000000a550c18 --function 0x0000000000000000000000000a550c18::MyToken::init --blocking"
#    Then cmd: "chain get-txn @$.execute_output.txn_hash@"
#    Then cmd: "account show"
#    Then cmd: "account execute-function -s 0x0000000000000000000000000a550c18 --function 0x0000000000000000000000000a550c18::MyToken::mint --blocking --arg 1000000u128"
#    Then assert: "$.execute_output.txn_info.status Executed"
#    Then cmd: "chain get-txn @$.execute_output.txn_hash@"
#    Then cmd: "account show 0x0000000000000000000000000a550c18"
#    # TODO match token balance
#    #Then assert: "$.account.balances.'0x0000000000000000000000000a550c18::MyToken::MyToken' 1000000"
#    Then stop
#
#    Examples:
#      |  |


#simplenft
#  Scenario Outline: [cmd] simple_nft test
#    Then cmd: "account unlock 0x0000000000000000000000000a550c18"
#    Then cmd: "dev compile ../examples/simple_nft/module/SimpleNFT.move -o ../examples/simple_nft/build -s 0x0000000000000000000000000a550c18"
#    Then cmd: "dev package ../examples/simple_nft/build -o ../examples/simple_nft/package/ -n simple_nft --function 0x0000000000000000000000000a550c18::SimpleNFTScripts::initialize"
#    Then cmd: "dev deploy --blocking ../examples/simple_nft/package/simple_nft.blob"
#    # use default account to mint nft
#    Then cmd: "dev get-coin"
#    Then cmd: "account unlock"
#    Then cmd: "account execute-function --function 0x0000000000000000000000000a550c18::SimpleNFTScripts::test_mint_with_image -b"
#    Then assert: "$.execute_output.txn_info.status Executed"
#    Then cmd: "account execute-function --function 0x0000000000000000000000000a550c18::SimpleNFTScripts::test_mint_with_image_data -b"
#    Then assert: "$.execute_output.txn_info.status Executed"
#    # transfer to a550c18
#    Then cmd: "account execute-function -s 0x0000000000000000000000000a550c18 --function 0x0000000000000000000000000a550c18::SimpleNFTScripts::accept -b"
#    Then assert: "$.execute_output.txn_info.status Executed"
#    Then cmd: "account nft list"
#    Then cmd: "account nft transfer --uuid @$.list[0].uuid@ -r 0x0000000000000000000000000a550c18 -b"
#    Then cmd: "account nft list 0x0000000000000000000000000a550c18"
#    Then assert: "$.list[0].nft_type 0x0000000000000000000000000a550c18::SimpleNFT::SimpleNFT/0x0000000000000000000000000a550c18::SimpleNFT::SimpleNFTBody"
#    Then stop
#
#    Examples:
#      |  |

