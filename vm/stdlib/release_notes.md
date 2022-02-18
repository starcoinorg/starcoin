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

## Version 7

- init_script: "cargo run -- -v 7 -m StdlibUpgradeScripts -f upgrade_from_v6_to_v7"
- package_hash: 0x9f153064ee8800f831cca006da269d4387fd6b243f8ca9c7009f6160680b76c3
- Do not trigger Withdraw event when the amount is zero. (#2857)
- NFT improvements, resolve #2842 . (#2856)
- Implement yield farming module (#2832) (#2852)
- Support language version OnChainConfig (#2845)

## Version 8

- Account: `Account::remove_signer_capability` now is available to every user.  (#2926)
- Dao: If the user revokes all vote, should destroy the user's vote resource. Add change_vote script to DaoVoteScripts, and add more scripts to DaoVoteScripts. Support auto change_vote in DaoVoteScripts::cast_vote. (#2925, #2947)
- NFT: Support update NFTTypeInfo metadata. (#2952, #2887)
- And all stdlib modules now use v3 bytecode version. (#2956)

## Version 9

- YieldFarmingV2: fix #2989

### Version 10

- Add `0x1::Signature::ecrecover` , `0x1::Hash::ripemd160` and `0x1::EVMAddress`. (#3020)
- Store `VMConfig` info in Module, instead of Resource. (#3019)
- Make `append`, `remove`, `reverse` native in Vector. (#3055)
- Add `U256` implementation. (#3032)
- improve account balance function.(#3058)

### Version 11

- Upgrade bytecode to v4.(#3109)
- Use move pacakge system to organize stdlib source files.(#3109)
- Add `phantom` modifier to NFT and Token Module. (#3109)