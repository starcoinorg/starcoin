# Starcoin Stdlib Release Notes

## Version 1

- release for Starcoin 1.0

## Version 2

- SIP、Hash、UpgradeModuleV2
- init_script: "cargo run -- -v 2 -m PackageTxnManager -f convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2 -a 0x1"
- need update native gas table since native function keccak_256() is added

## Version 3

- Collection2、Treasury、TransferScripts
- init_script: "cargo run -- -v 3 -m StdlibUpgradeScripts -f upgrade_from_v2_to_v3 -a 3185136000000000000u128"
- need update Consensus Config 'base_reward_per_block'
    
## Version 4
    
- remove deprecated methods

## Version 5
    
- Add max amount limit to treasury withdraw propose.
- New authentication_key check strategy, create account do not need authentication_key. Provider Account::create_account_with_address, and TransactionManager::epilogue_v2

## Version 6
    
- init_script: "cargo run -- -v 6 -m StdlibUpgradeScripts -f upgrade_from_v5_to_v6"
- Update specs to use move 1.3 syntax. (#2603)
- Support contract account by using `SignerCapability` abstraction. (#2673)
- `Account::deposit` support deposit zero token. (#2745)
- `Account` supports auto-accept-token feature. (#2745)
- Implement Oracle protocol, support general data oracle and price oracles. (#2732)
- Implement NFT protocol which has builtin IdentifierNFT, GenesisNFT. (#2688, #2763, #2760, #2767, #2769, #2771, #2772)
- Add many script functions. (#2745, #2781)
- Fix `Math.mul_div`. (#2775 by xiangfeihan<xiangfeihan@bixin.com)