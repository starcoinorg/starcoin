# Starcoin Stdlib Release Notes

Version 1

    - release for Starcoin 1.0

Version 2

    - SIP、Hash、UpgradeModuleV2
    - init_script: "cargo run -- -v 2 -m PackageTxnManager -f convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2 -a 0x1"
    - need update native gas table since native function keccak_256() is added

Version 3

    - Collection2、Treasury、TransferScripts
    - init_script: "cargo run -- -v 3 -m StdlibUpgradeScripts -f upgrade_from_v2_to_v3 -a 3185136000000000000u128"
    - need update Consensus Config 'base_reward_per_block'
    
Version 4
    
    - remove deprecated methods

Version 5
    
    - Add max amount limit to treasury withdraw propose.
    - New authentication_key check strategy, create account do not need authentication_key. Provider Account::create_account_with_address, and TransactionManager::epilogue_v2