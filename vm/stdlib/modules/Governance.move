// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

address 0x1 {
module Governance {
    use 0x1::Token;
    use 0x1::Signer;
    use 0x1::Option;
    use 0x1::Timestamp;
    // use 0x1::Debug;
    use 0x1::GovernanceTreasury;

    const ERR_GOVER_INIT_REPEATE: u64 = 101;
    const ERR_GOVER_OBJECT_NONE_EXISTS: u64 = 102;
    const ERR_GOVER_WITHDRAW_OVERFLOW: u64 = 103;
    const ERR_GOVER_WEIGHT_DECREASE_OVERLIMIT: u64 = 104;
    const ERR_GOVER_NOT_STILL_FREEZE: u64 = 105;
    const ERR_GOVER_STAKE_EXISTS: u64 = 106;
    const ERR_GOVER_STAKE_NOT_EXISTS: u64 = 107;
    const ERR_GOVER_HAVERST_NO_GAIN: u64 = 108;
    const ERR_GOVER_TOTAL_WEIGHT_IS_ZERO: u64 = 109;

    /// The object of governance
    /// GovTokenT meaning token of governance
    /// AssetT meaning asset which has been staked in governance
    struct Governance<PoolType, GovTokenT> has key, store {
        withdraw_cap: GovernanceTreasury::WithdrawCapability<PoolType, GovTokenT>,
    }

    struct GovernanceAsset<PoolType, AssetT> has key, store {
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
    /// this will declare a governance pool
    public fun initialize<
        PoolType: store,
        GovTokenT: store>(account: &signer,
                          treasury: Token::Token<GovTokenT>) {
        assert(!exists_at<PoolType, GovTokenT>(Signer::address_of(account)), ERR_GOVER_INIT_REPEATE);

        let withdraw_cap = GovernanceTreasury::initialize<PoolType, GovTokenT>(account, treasury);
        move_to(account, Governance<PoolType, GovTokenT> {
            withdraw_cap,
        });
    }

    // Initialize asset pools
    public fun initialize_asset<PoolType: store, AssetT: store>(
        account: &signer,
        release_per_second: u128,
        delay: u64): ParameterModifyCapability<PoolType, AssetT> {

        assert(!exists_asset_at<PoolType, AssetT>(Signer::address_of(account)), ERR_GOVER_INIT_REPEATE);

        let now_seconds = Timestamp::now_seconds();
        move_to(account, GovernanceAsset<PoolType, AssetT> {
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
        release_per_second: u128) acquires GovernanceAsset {
        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);

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
        asset: AssetT) acquires GovernanceAsset {
        assert(!exists_stake_at_address<PoolType, AssetT>(Signer::address_of(account)), ERR_GOVER_STAKE_EXISTS);

        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);

        move_to(account, Stake<PoolType, AssetT> {
            asset: Option::some(asset),
            asset_weight: 0,
            last_harvest_index : gov_asset.harvest_index,
            gain: 0,
        });
    }

    /// Call by stake user, staking amount of asset in order to get governance token
    public fun stake<PoolType: store,
                     GovTokenT: store,
                     AssetT: store>(
        account: &signer,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>) acquires GovernanceAsset, Stake {
        assert(exists_stake_at_address<PoolType, AssetT>(Signer::address_of(account)), ERR_GOVER_STAKE_NOT_EXISTS);
        inner_stake<PoolType, GovTokenT, AssetT>(Signer::address_of(account), broker, asset_wrapper);
    }

    public fun stake_with_cap<PoolType: store,
                              GovTokenT: store,
                              AssetT: store>(
        account: address,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>,
        _cap: &ParameterModifyCapability<PoolType, AssetT>
    ) acquires GovernanceAsset, Stake {
        inner_stake<PoolType, GovTokenT, AssetT>(account, broker, asset_wrapper);
    }

    /// This function called by user for staking users governance authority in this pool
    fun inner_stake<PoolType: store, GovTokenT: store, AssetT: store>(
        account: address,
        broker: address,
        asset_wrapper: AssetWrapper<PoolType, AssetT>) acquires Stake, GovernanceAsset {
        let AssetWrapper<PoolType, AssetT> { 
            asset, 
            asset_weight, 
            asset_origin_weight,
        } = asset_wrapper;
        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);

        // Check locking time
        assert(gov_asset.start_time <= Timestamp::now_seconds(), ERR_GOVER_NOT_STILL_FREEZE);

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
        broker: address) : Token::Token<GovTokenT> acquires Governance, GovernanceAsset, Stake {
        let zero: u128 = 0;
        harvest<PoolType, GovTokenT, AssetT>(account, broker,zero)
    }

    /// Harvest governance token from stake
    public fun harvest<PoolType: store,
                       GovTokenT: store,
                       AssetT: store>(
        account: &signer,
        broker: address,
        amount: u128) : Token::Token<GovTokenT> acquires Governance, GovernanceAsset, Stake {
        let gov = borrow_global_mut<Governance<PoolType, GovTokenT>>(broker);
        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);
        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(Signer::address_of(account));

        // Perform settlement
        settle_with_param<PoolType, GovTokenT, AssetT>(gov_asset, stake);

        assert(stake.gain > 0, ERR_GOVER_HAVERST_NO_GAIN);
        assert(stake.gain - amount > 0, ERR_GOVER_WITHDRAW_OVERFLOW);

        // Withdraw goverment token
        if (amount > 0) {
            stake.gain = stake.gain - amount;
            GovernanceTreasury::withdraw_with_capability<PoolType, GovTokenT>(&mut gov.withdraw_cap, amount)
        } else {
            stake.gain = 0;
            GovernanceTreasury::withdraw_with_capability<PoolType, GovTokenT>(&mut gov.withdraw_cap, stake.gain)
        }
    }

    /// The user can quering all governance amount in any time and scene
    public fun query_gov_token_amount<PoolType: store,
                                      GovTokenT: store,
                                      AssetT : store>(account: &signer, broker: address): u128 acquires GovernanceAsset, Stake {
        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);
        let stake = borrow_global_mut<Stake<PoolType, AssetT>>(Signer::address_of(account));

        // Perform settlement
        settle_with_param<PoolType, GovTokenT, AssetT>(gov_asset, stake);

        stake.gain
    }
    
    /// Query total stake count from governance resource
    public fun query_total_stake<PoolType: store,
                                 AssetT: store>(broker: address): u128 acquires GovernanceAsset {
        let gov_asset = borrow_global_mut<GovernanceAsset<PoolType, AssetT>>(broker);
        gov_asset.asset_total_weight
    }

    /// Performing a settlement based given governance object and stake object.
    fun settle_with_param<PoolType: store,
                          GovTokenT: store,
                          AssetT: store>(gov_asset: &mut GovernanceAsset<PoolType, AssetT>,
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
        assert(asset_total_weight > 0, ERR_GOVER_TOTAL_WEIGHT_IS_ZERO);
        let time_period = Timestamp::now_seconds() - last_update_timestamp;
        harvest_index + (release_per_second * (time_period as u128)) / asset_total_weight
    }

    /// This function will return a gain index
    public fun calculate_withdraw_amount(harvest_index: u128,
                                         last_harvest_index: u128,
                                         asset_weight: u128): u128 {
        asset_weight * (harvest_index - last_harvest_index)
    }

    /// Check the Governance of TokenT is exists.
    public fun exists_at<PoolType: store, GovTokenT: store>(broker: address): bool {
        exists<Governance<PoolType, GovTokenT>>(broker)
    }

    /// Check the Governance of AsssetT is exists.
    public fun exists_asset_at<PoolType: store, AssetT: store>(broker: address): bool {
        exists<GovernanceAsset<PoolType, AssetT>>(broker)
    }

    /// Check stake at address exists.
    public fun exists_stake_at_address<PoolType: store, AssetT: store>(account: address): bool {
        exists<Stake<PoolType, AssetT>>(account)
    }
}
}