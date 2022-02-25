// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

address StarcoinFramework {
module YieldFarming {
    use StarcoinFramework::Token;
    use StarcoinFramework::Errors;

    const EDEPRECATED_FUNCTION: u64 = 19;
    const ERR_FARMING_INIT_REPEATE: u64 = 101;
    const ERR_FARMING_NOT_STILL_FREEZE: u64 = 102;
    const ERR_FARMING_STAKE_EXISTS: u64 = 103;
    const ERR_FARMING_STAKE_NOT_EXISTS: u64 = 104;
    const ERR_FARMING_HAVERST_NO_GAIN: u64 = 105;
    const ERR_FARMING_TOTAL_WEIGHT_IS_ZERO: u64 = 106;
    const ERR_EXP_DIVIDE_BY_ZERO: u64 = 107;
    const ERR_FARMING_BALANCE_EXCEEDED: u64 = 108;
    const ERR_FARMING_NOT_ENOUGH_ASSET: u64 = 109;
    const ERR_FARMING_TIMESTAMP_INVALID: u64 = 110;
    
    spec module {
        pragma verify = false;
    }

    /// The object of yield farming
    /// RewardTokenT meaning token of yield farming
    struct Farming<phantom PoolType, phantom RewardTokenT> has key, store {
        treasury_token: Token::Token<RewardTokenT>,
    }

    struct FarmingAsset<phantom PoolType, phantom AssetT> has key, store {
        asset_total_weight: u128,
        harvest_index: u128,
        last_update_timestamp: u64,
        // Release count per seconds
        release_per_second: u128,
        // Start time, by seconds, user can operate stake only after this timestamp
        start_time: u64,
    }

    /// Capability to modify parameter such as period and release amount
    struct ParameterModifyCapability<phantom PoolType, phantom AssetT> has key, store {}

    /// To store user's asset token
    struct Stake<phantom PoolType, AssetT> has key, store {
        asset: AssetT,
        asset_weight: u128,
        last_harvest_index: u128,
        gain: u128,
    }

    //////////////////////////////////////////////////////////////////////
    // Exponential functions

    const EXP_SCALE: u128 = 1000000000000000000;// e18

    struct Exp has copy, store, drop {
        mantissa: u128
    }

    fun exp(num: u128, denom: u128): Exp {
        // if overflow move will abort
        let scaledNumerator = mul_u128(num, EXP_SCALE);
        let rational = div_u128(scaledNumerator, denom);
        Exp {
            mantissa: rational
        }
    }

    fun mul_u128(a: u128, b: u128): u128 {
        if (a == 0 || b == 0) {
            return 0
        };

        a * b
    }

    fun div_u128(a: u128, b: u128): u128 {
        if ( b == 0) {
            abort Errors::invalid_argument(ERR_EXP_DIVIDE_BY_ZERO)
        };
        if (a == 0) {
            return 0
        };
        a / b
    }

    fun truncate(exp: Exp): u128 {
        return exp.mantissa / EXP_SCALE
    }

    /// Called by token issuer
    /// this will declare a yield farming pool
    public fun initialize<
        PoolType: store,
        RewardTokenT: store>(_account: &signer,
                             _treasury_token: Token::Token<RewardTokenT>) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    // Initialize asset pools
    public fun initialize_asset<PoolType: store, AssetT: store>(
        _account: &signer,
        _release_per_second: u128,
        _delay: u64): ParameterModifyCapability<PoolType, AssetT> {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    public fun modify_parameter<PoolType: store, RewardTokenT: store, AssetT: store>(
        _cap: &ParameterModifyCapability<PoolType, AssetT>,
        _broker: address,
        _release_per_second: u128) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Call by stake user, staking amount of asset in order to get yield farming token
    public fun stake<PoolType: store, RewardTokenT: store, AssetT: store>(
        _account: &signer,
        _broker: address,
        _asset: AssetT,
        _asset_weight: u128) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Unstake asset from farming pool
    public fun unstake<PoolType: store, RewardTokenT: store, AssetT: store>(_account: &signer, _broker: address)
    : (AssetT, Token::Token<RewardTokenT>) {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// Harvest yield farming token from stake
    public fun harvest<PoolType: store,
                       RewardTokenT: store,
                       AssetT: store>(
        _account: &signer,
        _broker: address,
        _amount: u128): Token::Token<RewardTokenT> {
        abort Errors::deprecated(EDEPRECATED_FUNCTION)
    }

    /// The user can quering all yield farming amount in any time and scene
    public fun query_gov_token_amount<PoolType: store,
                                      RewardTokenT: store,
                                      AssetT: store>(_account: &signer, _broker: address): u128 {
        0
    }

    /// Query total stake count from yield farming resource
    public fun query_total_stake<PoolType: store,
                                 AssetT: store>(_broker: address): u128 {
        0
    }

    /// Query stake weight from user staking objects.
    public fun query_stake<PoolType: store,
                           AssetT: store>(_account: &signer): u128 {
        0
    }

    /// Update farming asset
    fun calculate_harvest_index_with_asset<PoolType, AssetT>(_farming_asset: &FarmingAsset<PoolType, AssetT>, _now_seconds: u64): u128 {
        0
    }

    /// There is calculating from harvest index and global parameters without asset_total_weight
    public fun calculate_harvest_index_weight_zero(_harvest_index: u128,
                                                   _last_update_timestamp: u64,
                                                   _now_seconds: u64,
                                                   _release_per_second: u128): u128 {
        0
    }

    /// There is calculating from harvest index and global parameters
    public fun calculate_harvest_index(_harvest_index: u128,
                                       _asset_total_weight: u128,
                                       _last_update_timestamp: u64,
                                       _now_seconds: u64,
                                       _release_per_second: u128): u128 {
        0
    }

    /// This function will return a gain index
    public fun calculate_withdraw_amount(_harvest_index: u128,
                                         _last_harvest_index: u128,
                                         _asset_weight: u128): u128 {
        0
    }

    /// Check the Farming of TokenT is exists.
    public fun exists_at<PoolType: store, RewardTokenT: store>(broker: address): bool {
        exists<Farming<PoolType, RewardTokenT>>(broker)
    }

    /// Check the Farming of AsssetT is exists.
    public fun exists_asset_at<PoolType: store, AssetT: store>(broker: address): bool {
        exists<FarmingAsset<PoolType, AssetT>>(broker)
    }

    /// Check stake at address exists.
    public fun exists_stake_at_address<PoolType: store, AssetT: store>(account: address): bool {
        exists<Stake<PoolType, AssetT>>(account)
    }
}
}