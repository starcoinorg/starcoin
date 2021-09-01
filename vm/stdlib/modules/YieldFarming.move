// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

address 0x1 {
module YieldFarming {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::Timestamp;
    // use 0x1::Debug;
    use 0x1::YieldFarmingTreasury;

    const ERR_FARMING_INIT_REPEATE: u64 = 101;
    const ERR_FARMING_OBJECT_NONE_EXISTS: u64 = 102;
    const ERR_FARMING_WITHDRAW_OVERFLOW: u64 = 103;
    const ERR_FARMING_WEIGHT_DECREASE_OVERLIMIT: u64 = 104;
    const ERR_FARMING_NOT_STILL_FREEZE: u64 = 105;
    const ERR_FARMING_STAKE_EXISTS: u64 = 106;
    const ERR_FARMING_STAKE_NOT_EXISTS: u64 = 107;
    const ERR_FARMING_HAVERST_NO_GAIN: u64 = 108;
    const ERR_FARMING_TOTAL_WEIGHT_IS_ZERO: u64 = 109;

    /// The object of yield farming
    /// GovTokenT meaning token of yield farming
    /// AssetT meaning asset which has been staked in yield farming
    struct Farming<PoolType, GovTokenT> has key, store {
        withdraw_cap: YieldFarmingTreasury::WithdrawCapability<PoolType, GovTokenT>,
    }

    struct FarmingAsset<PoolType, AssetT> has key, store {
        asset_total_weight: u128,
        harvest_index: u128,
        last_update_timestamp: u64,
        // Release count per seconds
        release_per_second: u128,
        // Start time, by seconds, user can operate stake only after this timestamp
        start_time: u64,
    }

    /// Capability to modify parameter such as period and release amount
    struct ParameterModifyCapability<PoolType, AssetT> has key, store {}

    /// Asset wrapper
    struct AssetWrapper<PoolType, AssetT> has key {
        asset: AssetT,
        asset_weight: u128,
        asset_origin_weight: u128,
    }

    /// To store user's asset token
    struct Stake<PoolType, AssetT> has key, store {
        asset: Option::Option<AssetT>,
        asset_weight: u128,
        last_harvest_index: u128,
        gain: u128,
    }

    /// Called by token issuer
    /// this will declare a yield farming pool
    public fun initialize<
        PoolType: store,
        GovTokenT: store>(account: &signer,
                          treasury: Token::Token<GovTokenT>) {
        assert(!exists_at<PoolType, GovTokenT>(Signer::address_of(account)), ERR_FARMING_INIT_REPEATE);

        let withdraw_cap = YieldFarmingTreasury::initialize<PoolType, GovTokenT>(account, treasury);
        move_to(account, Farming<PoolType, GovTokenT> {
            withdraw_cap,
        });
    }

    // Initialize asset pools
    public fun initialize_asset<PoolType: store, AssetT: store>(
        account: &signer,
        release_per_second: u128,
        delay: u64): ParameterModifyCapability<PoolType, AssetT> {

        assert(!exists_asset_at<PoolType, AssetT>(Signer::address_of(account)), ERR_FARMING_INIT_REPEATE);

        let now_seconds = Timestamp::now_seconds();
        move_to(account, FarmingAsset<PoolType, AssetT> {
            asset_total_weight: 0,
            harvest_index: 0,
            last_update_timestamp: now_seconds,
            release_per_second,
            start_time: now_seconds + delay,
        });
        ParameterModifyCapability<PoolType, AssetT> {}
    }

    public fun modify_parameter<PoolType: store,
                                GovTokenT: store,
                                AssetT: store>(
        _cap: &ParameterModifyCapability<PoolType, AssetT>,
        broker: address,
        release_per_second: u128) acquires FarmingAsset {
        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);

        let now_seconds = Timestamp::now_seconds();

        // Recalculate harvest index
        if (gov_asset.asset_total_weight <= 0) {
            let time_period = now_seconds - gov_asset.last_update_timestamp;
            gov_asset.harvest_index = gov_asset.harvest_index + (release_per_second * (time_period as u128));
        } else {
            gov_asset.harvest_index = calculate_harvest_index(
                gov_asset.harvest_index,
                gov_asset.asset_total_weight,
                gov_asset.last_update_timestamp,
                gov_asset.release_per_second);
        };
        gov_asset.last_update_timestamp = now_seconds;
        gov_asset.release_per_second = release_per_second;
    }

    /// Borrow from `Stake` object, calling `stake` function to pay back which is `AssetWrapper`
    public fun borrow_asset<PoolType: store, AssetT: store>(
        account: address): AssetWrapper<PoolType, AssetT> acquires Stake {
        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(account);
        let asset = Option::extract(&mut stake.asset);
        AssetWrapper<PoolType, AssetT> {
            asset,
            asset_weight: stake.asset_weight,
            asset_origin_weight: stake.asset_weight,
        }
    }

    public fun borrow<PoolType, AssetT>(a: &mut AssetWrapper<PoolType, AssetT>): (&mut AssetT, u128) {
        (&mut a.asset, a.asset_weight)
    }

    public fun modify<PoolType, AssetT>(a: &mut AssetWrapper<PoolType, AssetT>, amount: u128) {
        a.asset_weight = amount;
    }

    /// Claim from user
    public fun claim<PoolType: store,
                     GovTokenT: store,
                     AssetT: store>(
        account: &signer,
        broker: address,
        asset: AssetT) acquires FarmingAsset {
        assert(!exists_stake_at_address<PoolType, AssetT>(Signer::address_of(account)), ERR_FARMING_STAKE_EXISTS);

        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);

        move_to(account, Stake<PoolType, AssetT> {
            asset: Option::some(asset),
            asset_weight: 0,
            last_harvest_index : gov_asset.harvest_index,
            gain: 0,
        });
    }

    /// Call by stake user, staking amount of asset in order to get yield farming token
    public fun stake<PoolType: store,
                     GovTokenT: store,
                     AssetT: store>(
        account: &signer,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>) acquires FarmingAsset, Stake {
        assert(exists_stake_at_address<PoolType, AssetT>(Signer::address_of(account)), ERR_FARMING_STAKE_NOT_EXISTS);
        inner_stake<PoolType, GovTokenT, AssetT>(Signer::address_of(account), broker, asset_wrapper);
    }

    public fun stake_with_cap<PoolType: store,
                              GovTokenT: store,
                              AssetT: store>(
        account: address,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>,
        _cap: &ParameterModifyCapability<PoolType, AssetT>
    ) acquires FarmingAsset, Stake {
        inner_stake<PoolType, GovTokenT, AssetT>(account, broker, asset_wrapper);
    }

    /// This function called by user for staking users yield farming authority in this pool
    fun inner_stake<PoolType: store, GovTokenT: store, AssetT: store>(
        account: address,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>) acquires Stake, FarmingAsset {
        let AssetWrapper<PoolType, AssetT> { 
            asset, 
            asset_weight, 
            asset_origin_weight,
        } = asset_wrapper;
        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);

        // Check locking time
        assert(gov_asset.start_time <= Timestamp::now_seconds(), ERR_FARMING_NOT_STILL_FREEZE);

        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(account);

        // Perform settlement before add weight
        settle_with_param<PoolType, GovTokenT, AssetT>(gov_asset, stake);

        stake.asset_weight = asset_weight;

        // update stake total weight from asset wrapper
        if (asset_weight > asset_origin_weight) {
            gov_asset.asset_total_weight = gov_asset.asset_total_weight + (asset_weight - asset_origin_weight);
        } else if (asset_weight < asset_origin_weight) {
            gov_asset.asset_total_weight = gov_asset.asset_total_weight - (asset_origin_weight - asset_weight);
        };

        Option::fill(&mut stake.asset, asset);
    }

    /// Harvest all token from stake asset
    public fun harvest_all<PoolType: store,
                           GovTokenT: store,
                           AssetT: store>(
        account: &signer,
        broker: address) : Token::Token<GovTokenT> acquires Farming, FarmingAsset, Stake {
        let zero: u128 = 0;
        harvest<PoolType, GovTokenT, AssetT>(account, broker,zero)
    }

    /// Harvest yield farming token from stake
    public fun harvest<PoolType: store,
                       GovTokenT: store,
                       AssetT: store>(
        account: &signer,
        broker: address,
        amount: u128) : Token::Token<GovTokenT> acquires Farming, FarmingAsset, Stake {
        let gov = borrow_global_mut<Farming<PoolType, GovTokenT>>(broker);
        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);
        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(Signer::address_of(account));

        // Perform settlement
        settle_with_param<PoolType, GovTokenT, AssetT>(gov_asset, stake);

        assert(stake.gain > 0, ERR_FARMING_HAVERST_NO_GAIN);
        assert(stake.gain - amount > 0, ERR_FARMING_WITHDRAW_OVERFLOW);

        // Withdraw goverment token
        if (amount > 0) {
            stake.gain = stake.gain - amount;
            YieldFarmingTreasury::withdraw_with_capability<PoolType, GovTokenT>(&mut gov.withdraw_cap, amount)
        } else {
            stake.gain = 0;
            YieldFarmingTreasury::withdraw_with_capability<PoolType, GovTokenT>(&mut gov.withdraw_cap, stake.gain)
        }
    }

    /// The user can quering all yield farming amount in any time and scene
    public fun query_gov_token_amount<PoolType: store,
                                      GovTokenT: store,
                                      AssetT : store>(account: &signer, broker: address): u128 acquires FarmingAsset, Stake {
        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);
        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(Signer::address_of(account));

        // Perform settlement
        settle_with_param<PoolType, GovTokenT, AssetT>(gov_asset, stake);

        stake.gain
    }
    
    /// Query total stake count from yield farming resource
    public fun query_total_stake<PoolType: store,
                                 AssetT: store>(broker: address): u128 acquires FarmingAsset {
        let gov_asset = borrow_global_mut<FarmingAsset<PoolType, AssetT>>(broker);
        gov_asset.asset_total_weight
    }

    /// Performing a settlement based given yield farming object and stake object.
    fun settle_with_param<PoolType: store,
                          GovTokenT: store,
                          AssetT: store>(gov_asset: &mut FarmingAsset<PoolType, AssetT>,
                                         stake: &mut Stake<PoolType, AssetT>) {
        let now_seconds = Timestamp::now_seconds();
        if (gov_asset.asset_total_weight <= 0) {
            let time_period = now_seconds - gov_asset.last_update_timestamp;
            let period_gain = gov_asset.release_per_second * (time_period as u128);

            stake.gain = stake.gain + period_gain;
            gov_asset.harvest_index = 0;
        } else {
            let period_gain = calculate_withdraw_amount(gov_asset.harvest_index, stake.last_harvest_index, stake.asset_weight);
            stake.last_harvest_index = gov_asset.harvest_index;
            stake.gain = stake.gain + period_gain;

            gov_asset.harvest_index = calculate_harvest_index(
                gov_asset.harvest_index, 
                gov_asset.asset_total_weight, 
                gov_asset.last_update_timestamp, 
                gov_asset.release_per_second);
        };
        gov_asset.last_update_timestamp = now_seconds;
    }

    /// There is calculating from harvest index and global parameters
    public fun calculate_harvest_index(harvest_index: u128,
                                       asset_total_weight: u128,
                                       last_update_timestamp: u64,
                                       release_per_second: u128): u128 {
        assert(asset_total_weight > 0, ERR_FARMING_TOTAL_WEIGHT_IS_ZERO);
        let time_period = Timestamp::now_seconds() - last_update_timestamp;
        harvest_index + (release_per_second * (time_period as u128)) / asset_total_weight
    }

    /// This function will return a gain index
    public fun calculate_withdraw_amount(harvest_index: u128,
                                         last_harvest_index: u128,
                                         asset_weight: u128): u128 {
        asset_weight * (harvest_index - last_harvest_index)
    }

    /// Check the Farming of TokenT is exists.
    public fun exists_at<PoolType: store, GovTokenT: store>(broker: address): bool {
        exists<Farming<PoolType, GovTokenT>>(broker)
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