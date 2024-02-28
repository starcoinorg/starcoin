Feature: cmd integration test

  Background:
    Given a dev node config
    And node handle
    And ipc rpc client

  # chain
  Scenario Outline: [cmd] cli chain test
    Then cmd: "chain epoch-info"
    Then cmd: "chain list-block"
    Then cmd: "chain get-block {{$.chain[1].ok[0].block_hash}}"
    Then cmd: "chain get-block {{$.chain[1].ok[0].number}}"
    Then cmd: "chain get-txn-infos {{$.chain[1].ok[0].block_hash}}"
    Then cmd: "chain get-txn {{$.chain[4].ok[0].transaction_hash}}"
    Then cmd: "chain get-events {{$.chain[4].ok[0].transaction_hash}}"
    Then cmd: "chain get-txn-info-list -s 0 -c 5"
    Then cmd: "chain get-block-info {{$.chain[1].ok[0].number}}"
    Then cmd: "chain get-txn-proof --block-hash {{$.chain[1].ok[0].block_hash}} --transaction-global-index 0"
    Then cmd: "chain get-txn-proof --block-hash {{$.chain[1].ok[0].block_hash}} --transaction-global-index 0 --raw"
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
    Then assert: "{{$.node[-1].ok}} == Stopped"
    Then cmd: "node service start starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then cmd: "node service check starcoin_miner::generate_block_event_pacemaker::GenerateBlockEventPacemaker"
    Then assert: "{{$.node[-1].ok}} == Started"
    #ensure some service start successful.
    Then cmd: "node service check starcoin_rpc_server::service::RpcService"
    Then assert: "{{$.node[-1].ok}} == Started"
    Then cmd: "node service check starcoin_node::metrics::MetricsServerActorService"
    Then assert: "{{$.node[-1].ok}} == Started"
    Then stop

    Examples:
      |  |

  # multisig account
  Scenario Outline: [cmd] multisig account
    Then cmd: "account unlock"
    Then cmd: "dev get-coin"
    Then cmd: "account generate-keypair -c 3"
    Then cmd: "account derive-address -t 2 -p {{$.account[1].ok[0].public_key}} -p {{$.account[1].ok[1].public_key}}  -p {{$.account[1].ok[2].public_key}} "
    Then cmd: "account transfer --blocking -r {{$.account[-1].ok.receipt_identifier}} -t 0x1::STC::STC -v 10000000 --gas-token 0x1::STC::STC"
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
    Then cmd: "account submit-txn {{$.account[-1].ok}} -b"
    Then stop

    Examples:
      |  |
      |  |

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
    Then cmd: "state get-proof {{$.account[0].ok.account.address}}/1/0x1::Account::Account"
    Then cmd: "state get-proof {{$.account[0].ok.account.address}}/1/0x1::Account::Account --raw"
    Then cmd: "state get resource {{$.account[0].ok.account.address}} 0x1::Account::Account"
    Then assert: "{{$.state[-1].ok.json.sequence_number}} == 0"
    Then cmd: "account show"
    Then cmd: "state list resource {{$.account[0].ok.account.address}}"
    Then cmd: "state list resource -t 0x1::Account::Balance {{$.account[0].ok.account.address}}"
    Then cmd: "state list resource -s 5 -i 0 {{$.account[0].ok.account.address}}"
    Then cmd: "account show"
    Then cmd: "state list code {{$.account[0].ok.account.address}}"
    Then stop

    Examples:
      |  |

  #account
  Scenario Outline: [cmd] account test
    Then cmd: "account show"
    Then cmd: "account unlock"
    Then cmd: "dev get-coin"
    Then cmd: "account create -p transfer"
    Then cmd: "account transfer --blocking -v 10000 -r {{$.account[2].ok.address}}"
    Then cmd: "chain get-txn {{$.account[3].ok.execute_output.txn_hash}}"
    Then cmd: "chain get-events {{$.account[3].ok.execute_output.txn_hash}}"
    Then cmd: "account create -p compat"
    Then cmd: "account unlock -p compat {{$.account[4].ok.address}}"
    Then cmd: "account export -p compat {{$.account[4].ok.address}}"
    Then cmd: "account change-password {{$.account[4].ok.address}} -p hello"
    Then cmd: "account remove {{$.account[4].ok.address}} -p hello"
    Then cmd: "account generate-keypair"
    Then cmd: "account import-readonly -i {{$.account[-1].ok[0].public_key}}"
    Then cmd: "account remove {{$.account[-1].ok.address}}"
    Then cmd: "account generate-keypair"
    Then cmd: "account import -i {{$.account[-1].ok[0].private_key}}"
    Then cmd: "account remove {{$.account[-1].ok.address}}"
    Then cmd: "account import -i 0x6afe92b85f5fd61b099d1bd805aa54b9737ad73522f490c47872cd028ea338f3"
    Then cmd: "account transfer --blocking -v 100000000 -r 0x809c795045105a7b1efbcca4510d2034"
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    # using a temporal private key as import
    Then cmd: "account rotate-authentication-key 0x809c795045105a7b1efbcca4510d2034 -i 0x3885e7dde8381046849d64d28b675f1c668dc36eaa9be11cbcaadb24c3917554 --gas-token 0x1::STC::STC"
    # rotate-authentication-key twice for:
    # 1. auth key will be verified on chain, so do it again for checking last rotation.
    # 2. ensuring it's idempotent
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    Then cmd: "account rotate-authentication-key 0x809c795045105a7b1efbcca4510d2034 -i 0x3885e7dde8381046849d64d28b675f1c668dc36eaa9be11cbcaadb24c3917554"
    Then cmd: "account unlock 0x809c795045105a7b1efbcca4510d2034"
    # transfer after rotation
    Then cmd: "account transfer --blocking -v 10000000 -s 0x809c795045105a7b1efbcca4510d2034 -r {{$.account[2].ok.address}}"
    Then cmd: "chain get-txn {{$.account[-1].ok.execute_output.txn_hash}}"
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
    Then cmd: "account verify-sign-message -m {{$.account[-1].ok.hex}}"
    Then assert: "{{$.account[-1].ok.ok}} == true"
    # create the account on chain
    Then cmd: "dev get-coin"
    Then cmd: "account sign-message  -m helloworld"
    Then cmd: "account verify-sign-message -m {{$.account[-1].ok.hex}}"
    Then assert: "{{$.account[-1].ok.ok}} == true"
    # init the auth key on chain by send the first transaction, test authkey is not dummy key.
    Then cmd: "account transfer -v 1000 -r 0xA550C18 -b"
    Then cmd: "account sign-message -m helloworld"
    Then cmd: "account verify-sign-message -m {{$.account[-1].ok.hex}}"
    Then assert: "{{$.account[-1].ok.ok}} == true"
    # test multi sign account
    Then cmd: "account sign-message -s 0xA550C18 -m helloworld"
    Then cmd: "account verify-sign-message -m {{$.account[-1].ok.hex}}"
    Then assert: "{{$.account[-1].ok.ok}} == true"

    Examples:
      |  |

  #StarcoinFramework checkpoint
  Scenario Outline: [ignore] starcoin-framework checkpoint
    Then cmd: "dev get-coin"
    Then cmd: "account unlock"
    Then cmd: "account execute-function --function 0x1::Block::checkpoint_entry -b"
    Then cmd: "dev call-api chain.get_block_by_number [1,{\"raw\":true}]"
    Then cmd: "account execute-function --function 0x1::Block::update_state_root_entry --arg {{$.dev[1].ok.raw.header}} -b"
    Then cmd: "dev call --function 0x1::Block::latest_state_root"
    Then assert: "{{$.dev[2].ok[1]}} == {{$.dev[1].ok.header.state_root}}"

    Examples:
      |  |

  #flexidagconfig dao testing
  Scenario Outline: [cmd] starcoin flexidagconfig dao
    # 1. deposit to default account which is a proposer
    Then cmd: "dev get-coin -v 1000000"
    Then cmd: "account unlock"
    # 2. create FlexiDagConfig proposal with proposer account
    Then cmd: "account execute-function --function 0x1::OnChainConfigScripts::propose_update_flexi_dag_effective_height -s {{$.account[0].ok.address}} --arg 10000u64 --arg 0u64 -b"
    Then cmd: "dev sleep -t 60000"
    # 3. make sure proposal has been ACTIVE for voting
    Then cmd: "dev gen-block"
    Then cmd: "dev call --function 0x1::Dao::proposal_state -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0"
    Then assert: "{{$.dev[-1].ok[0]}} == 2"
    # 4. create a new account to vote, deposit enough tokens
    Then cmd: "account create -p 1234"
    Then cmd: "dev get-coin -v 10000000 {{$.account[2].ok.address}}"
    Then cmd: "dev get-coin -v 10000000 {{$.account[2].ok.address}}"
    Then cmd: "account unlock {{$.account[2].ok.address}} -p 1234"
    # 5. stake and cast vote with new account
    Then cmd: "account execute-function --function 0x1::DaoVoteScripts::cast_vote -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> -s {{$.account[2].ok.address}} --arg {{$.account[0].ok.address}} --arg 0 --arg true --arg 12740545600000000u128 -b"
    Then cmd: "dev sleep -t 3600000"
    # 6. switch to proposer account, make sure proposal has been AGREED
    Then cmd: "account unlock"
    Then cmd: "dev gen-block"
    Then cmd: "dev call --function 0x1::Dao::proposal_state -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0"
    Then assert: "{{$.dev[-1].ok[0]}} == 4"
    # 7. add proposal to execution queue with proposer account
    Then cmd: "account execute-function -s {{$.account[0].ok.address}} --function 0x1::Dao::queue_proposal_action -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0 -b"
    Then cmd: "dev sleep -t 3600000"
    # 8. make sure proposal is EXECUTABLE
    Then cmd: "dev gen-block"
    Then cmd: "dev call --function 0x1::Dao::proposal_state -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0"
    Then assert: "{{$.dev[-1].ok[0]}} == 6"
    # 9. execute proposal with proposer account
    Then cmd: "account execute-function -s {{$.account[0].ok.address}} --function 0x1::OnChainConfigScripts::execute_on_chain_config_proposal -t 0x1::FlexiDagConfig::FlexiDagConfig --arg 0 -b"
    # 10. make sure the proposal is EXTRACTED
    Then cmd: "dev gen-block"
    Then cmd: "dev call --function 0x1::Dao::proposal_state -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0"
    Then assert: "{{$.dev[-1].ok[0]}} == 7"
    # 11. clean up proposal
    Then cmd: "account execute-function --function 0x1::Dao::destroy_terminated_proposal -t 0x1::STC::STC -t 0x1::OnChainConfigDao::OnChainConfigUpdate<0x1::FlexiDagConfig::FlexiDagConfig> --arg {{$.account[0].ok.address}} --arg 0u64"
    # 12. check the latest flexidagconfig
    Then cmd: "state get resource 0x1 0x1::Config::Config<0x01::FlexiDagConfig::FlexiDagConfig>"
    Then assert: "{{$.state[0].ok.json.payload.effective_height}} == 10000"

    Examples:
      |  |  |

  #easy gas testing
  Scenario Outline: starcoin easy gas test
    Then cmd: "dev get-coin -v 1000000"
    Then cmd: "account show"
    Then cmd: "account unlock"
    # stake to sbt
    Then cmd: "account execute-function --function 0x1::DummyTokenScripts::mint --arg 9000u128 -b"
    Then cmd: "dev call-api chain.info"
    # check point and update state root
    Then cmd: "account execute-function --function 0x1::Block::checkpoint_entry -b"
    Then cmd: "dev call-api chain.get_block_by_number [{{$.dev[1].ok.head.number}},{\"raw\":true}]"
    Then cmd: "account execute-function --function 0x1::Block::update_state_root_entry --arg {{$.dev[2].ok.raw.header}} -b"
    Then cmd: "dev call --function 0x1::Block::latest_state_root"
    Then assert: "{{$.dev[3].ok[1]}} == {{$.dev[2].ok.header.state_root}}"
    # register oracle
    Then cmd: "account execute-function --function 0x1::EasyGas::register_oracle -t 0x1::DummyToken::DummyToken --arg 15u8 -b"
    Then cmd: "account execute-function --function 0x1::EasyGas::init_data_source -t 0x1::DummyToken::DummyToken --arg 43793u128 -b"
    Then cmd: "account execute-function --function 0x1::EasyGas::update -t 0x1::DummyToken::DummyToken --arg 43794u128 -b"
    Then cmd: "dev call --function 0x1::EasyGas::gas_oracle_read -t 0x1::DummyToken::DummyToken"
    Then assert: "{{$.dev[15].ok[0]}} == 43794"
    # transfer stc to 0x1
    Then cmd: "account transfer --blocking -r 0x1 -v 10000000000"
    # transfer use gas fee by DummyToken
    Then cmd: "account transfer --blocking -r 0x1 -v 1 --gas-token 0x1::DummyToken::DummyToken"
    Then cmd: "state get resource {{$.account[0].ok.account.address}} 0x1::Account::Balance<0x1::DummyToken::DummyToken>"
    Then assert: "{{$.state[0].ok.json.token.value}} == 8999"
    Then stop

    Examples:
      |  |


#mytoken
#  Scenario Outline: [cmd] my_token test
#    Then cmd: "account unlock 0x0000000000000000000000000a550c18"
#    Then cmd: "dev compile ../examples/my_token/MyToken.move -o ../examples -s 0x0000000000000000000000000a550c18"
#    Then cmd: "dev deploy --blocking @$[0]@"
#    Then cmd: "account execute-function -s 0x0000000000000000000000000a550c18 --function 0x0000000000000000000000000a550c18::MyToken::init --gas-token 0x1::STC::STC --blocking"
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

