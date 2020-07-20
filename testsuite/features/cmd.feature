Feature: cmd integration test
  Background:
#    Given a dev node config
#    And node dev handle
#    And dev rpc client
    Given remote rpc client

    #  1. node info
  Scenario: [cmd] node info
    Then [cmd] node info
#    Then [cmd] node peers
#    Then node handle stop
    #  2. wallet list
  Scenario: [cmd] wallet list
    Then [cmd] wallet list
    Then [cmd] wallet show
#    Then node handle stop

  #  3. dev get coin
  Scenario Outline: [cmd] dev get coin
    Then dev get_coin "<amount>"
#    Then node handle stop

    Examples:
      | amount |
      | 200000 |

  #  4. wallet create
  Scenario Outline: [cmd] wallet create
    Then wallet create "<password>"
    Then [cmd] wallet show
#    Then node handle stop

    Examples:
      | password |
      | ssyuan |

  #  7. wallet unlock
#  Scenario Outline: [cmd] wallet unlock
#    Then wallet unlock password:"<password>"
#
#    Examples:
#      | password |
#      |   |

  #  8. common cmd cli
  Scenario Outline: [cmd] cli
    Then cmd cli: "<cmd>"
#    Then node handle stop

    Examples:
      | cmd |
      | wallet create -p dssss |
      | wallet show |

  Scenario Outline: [cmd]  cli continuous 1
    Then cmd: "wallet create -p dssss $.address"
    Then cmd: "wallet show $.account.address"
    Then cmd: "wallet unlock -p dssss $.account.address"

    Examples:
      |  |

  Scenario Outline: [cmd]  cli continuous 2
    Then cmd: "chain show $.head_block"
    Then cmd: "chain get_block $.author"
    Then cmd: "wallet show $.account.address"

    Examples:
      |  |

  Scenario Outline: [cmd]  cli continuous 3
    Then cmd: "chain show $.head_block"
    Then cmd: "chain get_block $.author"
    Then cmd: "wallet show $.account.address"

    Examples:
      |  |

  Scenario Outline: [cmd]  cli continuous 4
    Then cmd: "chain branches $[0].head_block"
    Then cmd: "chain get_txn_by_block  $[0].transaction_hash"
    Then cmd: "chain get_txn $.txn_info_id"

    Examples:
      |  |

  Scenario Outline: [cmd]  cli continuous 5
#    Then cmd: "wallet list "
    Then cmd: "wallet unlock $.None"
    Then cmd: "dev compile ../examples/my_counter/module/MyCounter.move -o ../examples $.result"
    Then cmd: "dev execute $.txn_hash"
#    Then cmd: "chain get_txn $.major_status"

    Examples:
      |  |
