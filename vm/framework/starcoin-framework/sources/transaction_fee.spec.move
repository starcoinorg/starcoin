spec starcoin_framework::transaction_fee {
    /// <high-level-req>
    /// No.: 1
    /// Requirement: Given the blockchain is in an operating state, it guarantees that the Starcoin framework signer may burn
    /// Starcoin coins.
    /// Criticality: Critical
    /// Implementation: The StarcoinCoinCapabilities structure is defined in this module and it stores burn capability to
    /// burn the gas fees.
    /// Enforcement: Formally Verified via [high-level-req-1](module).
    ///
    /// No.: 2
    /// Requirement: The initialization function may only be called once.
    /// Criticality: Medium
    /// Implementation: The initialize_fee_collection_and_distribution function ensures CollectedFeesPerBlock does not
    /// already exist.
    /// Enforcement: Formally verified via [high-level-req-2](initialize_fee_collection_and_distribution).
    ///
    /// No.: 3
    /// Requirement: Only the admin address is authorized to call the initialization function.
    /// Criticality: Critical
    /// Implementation: The initialize_fee_collection_and_distribution function ensures only the Starcoin framework address
    /// calls it.
    /// Enforcement: Formally verified via [high-level-req-3](initialize_fee_collection_and_distribution).
    ///
    /// No.: 4
    /// Requirement: The percentage of the burnt collected fee is always a value from 0 to 100.
    /// Criticality: Medium
    /// Implementation: During the initialization of CollectedFeesPerBlock in
    /// Initialize_fee_collection_and_distribution, and while upgrading burn percentage, it asserts that burn_percentage
    /// is within the specified limits.
    /// Enforcement: Formally verified via [high-level-req-4](CollectedFeesPerBlock).
    ///
    /// No.: 5
    /// Requirement: Prior to upgrading the burn percentage, it must process all the fees collected up to that point.
    /// Criticality: Critical
    /// Implementation: The upgrade_burn_percentage function ensures process_collected_fees function is called before
    /// updating the burn percentage.
    /// Enforcement: Formally verified in [high-level-req-5](ProcessCollectedFeesRequiresAndEnsures).
    ///
    /// No.: 6
    /// Requirement: The presence of the resource, indicating collected fees per block under the Starcoin framework account,
    /// is a prerequisite for the successful execution of the following functionalities: Upgrading burn percentage.
    /// Registering a block proposer. Processing collected fees.
    /// Criticality: Low
    /// Implementation: The functions: upgrade_burn_percentage, register_proposer_for_fee_collection, and
    /// process_collected_fees all ensure that the CollectedFeesPerBlock resource exists under starcoin_framework by
    /// calling the is_fees_collection_enabled method, which returns a boolean value confirming if the resource exists
    /// or not.
    /// Enforcement: Formally verified via [high-level-req-6.1](register_proposer_for_fee_collection), [high-level-req-6.2](process_collected_fees), and [high-level-req-6.3](upgrade_burn_percentage).
    /// </high-level-req>
    ///
    spec module {
        use starcoin_framework::chain_status;

        // TODO(fa_migration)
        pragma verify = false;

        pragma aborts_if_is_strict;
        // property 1: Given the blockchain is in an operating state, it guarantees that the Starcoin framework signer may burn Starcoin coins.
        /// [high-level-req-1]
        invariant [suspendable] chain_status::is_operating() ==> exists<CoinCapabilities>(@starcoin_framework) || exists<FABurnCapabilities>(@starcoin_framework);
    }

    spec CollectedFeesPerBlock {
        // property 4: The percentage of the burnt collected fee is always a value from 0 to 100.
        /// [high-level-req-4]
        invariant burn_percentage <= 100;
    }

    spec initialize_fee_collection_and_distribution(starcoin_framework: &signer, burn_percentage: u8) {
        use std::signer;
        use starcoin_framework::stake::ValidatorFees;
        use starcoin_framework::aggregator_factory;
        use starcoin_framework::system_addresses;

        // property 2: The initialization function may only be called once.
        /// [high-level-req-2]
        aborts_if exists<CollectedFeesPerBlock>(@starcoin_framework);
        aborts_if burn_percentage > 100;

        let starcoin_addr = signer::address_of(starcoin_framework);
        // property 3: Only the admin address is authorized to call the initialization function.
        /// [high-level-req-3]
        aborts_if !system_addresses::is_starcoin_framework_address(starcoin_addr);
        aborts_if exists<ValidatorFees>(starcoin_addr);

        include system_addresses::AbortsIfNotStarcoinFramework { account: starcoin_framework };
        include aggregator_factory::CreateAggregatorInternalAbortsIf;
        aborts_if exists<CollectedFeesPerBlock>(starcoin_addr);

        ensures exists<ValidatorFees>(starcoin_addr);
        ensures exists<CollectedFeesPerBlock>(starcoin_addr);
    }

    spec upgrade_burn_percentage(starcoin_framework: &signer, new_burn_percentage: u8) {
        use std::signer;

        // Percentage validation
        aborts_if new_burn_percentage > 100;
        // Signer validation
        let starcoin_addr = signer::address_of(starcoin_framework);
        aborts_if !system_addresses::is_starcoin_framework_address(starcoin_addr);

        // property 5: Prior to upgrading the burn percentage, it must process all the fees collected up to that point.
        // property 6: Ensure the presence of the resource.
        // Requirements and ensures conditions of `process_collected_fees`
        /// [high-level-req-5]
        /// [high-level-req-6.3]
        include ProcessCollectedFeesRequiresAndEnsures;

        // The effect of upgrading the burn percentage
        ensures exists<CollectedFeesPerBlock>(@starcoin_framework) ==>
            global<CollectedFeesPerBlock>(@starcoin_framework).burn_percentage == new_burn_percentage;
    }

    spec register_proposer_for_fee_collection(proposer_addr: address) {
        aborts_if false;
        // property 6: Ensure the presence of the resource.
        /// [high-level-req-6.1]
        ensures is_fees_collection_enabled() ==>
            option::spec_borrow(global<CollectedFeesPerBlock>(@starcoin_framework).proposer) == proposer_addr;
    }

    spec burn_coin_fraction(coin: &mut Coin<STC>, burn_percentage: u8) {
        use starcoin_framework::coin::CoinInfo;
        use starcoin_framework::starcoin_coin::STC;

        requires burn_percentage <= 100;
        requires exists<CoinCapabilities>(@starcoin_framework);
        requires exists<CoinInfo<STC>>(@starcoin_framework);

        let amount_to_burn = (burn_percentage * coin::value(coin)) / 100;
        // include (amount_to_burn > 0) ==> coin::AbortsIfNotExistCoinInfo<StarcoinCoin>;
        include amount_to_burn > 0 ==> coin::CoinSubAbortsIf<STC> { amount: amount_to_burn };
        ensures coin.value == old(coin).value - amount_to_burn;
    }

    spec fun collectedFeesAggregator(): AggregatableCoin<STC> {
        global<CollectedFeesPerBlock>(@starcoin_framework).amount
    }

    spec schema RequiresCollectedFeesPerValueLeqBlockStarcoinSupply {
        use starcoin_framework::optional_aggregator;
        use starcoin_framework::aggregator;
        let maybe_supply = coin::get_coin_supply_opt<STC>();
        // property 6: Ensure the presence of the resource.
        requires
            (is_fees_collection_enabled() && option::is_some(maybe_supply)) ==>
                (aggregator::spec_aggregator_get_val(global<CollectedFeesPerBlock>(@starcoin_framework).amount.value) <=
                    optional_aggregator::optional_aggregator_value(
                        option::spec_borrow(coin::get_coin_supply_opt<STC>())
                    ));
    }

    spec schema ProcessCollectedFeesRequiresAndEnsures {
        use starcoin_framework::coin::CoinInfo;
        use starcoin_framework::starcoin_coin::STC;
        use starcoin_framework::aggregator;
        use starcoin_std::table;

        requires exists<CoinCapabilities>(@starcoin_framework);
        requires exists<stake::ValidatorFees>(@starcoin_framework);
        requires exists<CoinInfo<STC>>(@starcoin_framework);
        include RequiresCollectedFeesPerValueLeqBlockStarcoinSupply;

        aborts_if false;

        let collected_fees = global<CollectedFeesPerBlock>(@starcoin_framework);
        let post post_collected_fees = global<CollectedFeesPerBlock>(@starcoin_framework);
        let pre_amount = aggregator::spec_aggregator_get_val(collected_fees.amount.value);
        let post post_amount = aggregator::spec_aggregator_get_val(post_collected_fees.amount.value);
        let fees_table = global<stake::ValidatorFees>(@starcoin_framework).fees_table;
        let post post_fees_table = global<stake::ValidatorFees>(@starcoin_framework).fees_table;
        let proposer = option::spec_borrow(collected_fees.proposer);
        let fee_to_add = pre_amount - pre_amount * collected_fees.burn_percentage / 100;
        ensures is_fees_collection_enabled() ==> option::spec_is_none(post_collected_fees.proposer) && post_amount == 0;
        ensures is_fees_collection_enabled() && aggregator::spec_read(collected_fees.amount.value) > 0 &&
            option::spec_is_some(collected_fees.proposer) ==>
            if (proposer != @vm_reserved) {
                if (table::spec_contains(fees_table, proposer)) {
                    table::spec_get(post_fees_table, proposer).value == table::spec_get(
                        fees_table,
                        proposer
                    ).value + fee_to_add
                } else {
                    table::spec_get(post_fees_table, proposer).value == fee_to_add
                }
            } else {
                option::spec_is_none(post_collected_fees.proposer) && post_amount == 0
            };
    }

    spec process_collected_fees() {
        /// [high-level-req-6.2]
        include ProcessCollectedFeesRequiresAndEnsures;
    }

    /// `StarcoinCoinCapabilities` should be exists.
    spec burn_fee(account: address, fee: u64) {
        use starcoin_std::type_info;
        use starcoin_framework::optional_aggregator;
        use starcoin_framework::coin;
        use starcoin_framework::coin::{CoinInfo, CoinStore};
        // TODO(fa_migration)
        pragma verify = false;

        aborts_if !exists<CoinCapabilities>(@starcoin_framework);

        // This function essentially calls `coin::burn_coin`, monophormized for `StarcoinCoin`.
        let account_addr = account;
        let amount = fee;

        let starcoin_addr = type_info::type_of<STC>().account_address;
        let coin_store = global<CoinStore<STC>>(account_addr);
        let post post_coin_store = global<CoinStore<STC>>(account_addr);

        // modifies global<CoinStore<StarcoinCoin>>(account_addr);

        aborts_if amount != 0 && !(exists<CoinInfo<STC>>(starcoin_addr)
            && exists<CoinStore<STC>>(account_addr));
        aborts_if coin_store.coin.value < amount;

        let maybe_supply = global<CoinInfo<STC>>(starcoin_addr).supply;
        let supply_aggr = option::spec_borrow(maybe_supply);
        let value = optional_aggregator::optional_aggregator_value(supply_aggr);

        let post post_maybe_supply = global<CoinInfo<STC>>(starcoin_addr).supply;
        let post post_supply = option::spec_borrow(post_maybe_supply);
        let post post_value = optional_aggregator::optional_aggregator_value(post_supply);

        aborts_if option::spec_is_some(maybe_supply) && value < amount;

        ensures post_coin_store.coin.value == coin_store.coin.value - amount;
        ensures if (option::spec_is_some(maybe_supply)) {
            post_value == value - amount
        } else {
            option::spec_is_none(post_maybe_supply)
        };
        ensures coin::supply<STC> == old(coin::supply<STC>) - amount;
    }

    spec mint_and_refund(account: address, refund: u64) {
        use starcoin_std::type_info;
        use starcoin_framework::starcoin_coin::STC;
        use starcoin_framework::coin::{CoinInfo, CoinStore};
        use starcoin_framework::coin;
        // TODO(fa_migration)
        pragma verify = false;
        // pragma opaque;

        let starcoin_addr = type_info::type_of<STC>().account_address;

        aborts_if (refund != 0) && !exists<CoinInfo<STC>>(starcoin_addr);
        include coin::CoinAddAbortsIf<STC> { amount: refund };

        aborts_if !exists<CoinStore<STC>>(account);
        // modifies global<CoinStore<StarcoinCoin>>(account);

        aborts_if !exists<CoinMintCapability>(@starcoin_framework);

        let supply = coin::supply<STC>;
        let post post_supply = coin::supply<STC>;
        aborts_if [abstract] supply + refund > MAX_U128;
        ensures post_supply == supply + refund;
    }

    spec collect_fee(account: address, fee: u64) {
        use starcoin_framework::aggregator;
        // TODO(fa_migration)
        pragma verify = false;

        let collected_fees = global<CollectedFeesPerBlock>(@starcoin_framework).amount;
        let aggr = collected_fees.value;
        let coin_store = global<coin::CoinStore<STC>>(account);
        aborts_if !exists<CollectedFeesPerBlock>(@starcoin_framework);
        aborts_if fee > 0 && !exists<coin::CoinStore<STC>>(account);
        aborts_if fee > 0 && coin_store.coin.value < fee;
        aborts_if fee > 0 && aggregator::spec_aggregator_get_val(aggr)
            + fee > aggregator::spec_get_limit(aggr);
        aborts_if fee > 0 && aggregator::spec_aggregator_get_val(aggr)
            + fee > MAX_U128;

        let post post_coin_store = global<coin::CoinStore<STC>>(account);
        let post post_collected_fees = global<CollectedFeesPerBlock>(@starcoin_framework).amount;
        ensures post_coin_store.coin.value == coin_store.coin.value - fee;
        ensures aggregator::spec_aggregator_get_val(post_collected_fees.value) == aggregator::spec_aggregator_get_val(
            aggr
        ) + fee;
    }

    /// Ensure caller is admin.
    /// Aborts if `StarcoinCoinCapabilities` already exists.
    spec store_coin_burn_cap(starcoin_framework: &signer, burn_cap: BurnCapability<STC>) {
        use std::signer;

        // TODO(fa_migration)
        pragma verify = false;

        let addr = signer::address_of(starcoin_framework);
        aborts_if !system_addresses::is_starcoin_framework_address(addr);

        aborts_if exists<FABurnCapabilities>(addr);
        aborts_if exists<CoinCapabilities>(addr);

        ensures exists<FABurnCapabilities>(addr) || exists<CoinCapabilities>(addr);
    }

    /// Ensure caller is admin.
    /// Aborts if `StarcoinCoinMintCapability` already exists.
    spec store_coin_mint_cap(starcoin_framework: &signer, mint_cap: MintCapability<STC>) {
        use std::signer;
        let addr = signer::address_of(starcoin_framework);
        aborts_if !system_addresses::is_starcoin_framework_address(addr);
        aborts_if exists<CoinMintCapability>(addr);
        ensures exists<CoinMintCapability>(addr);
    }

    /// Historical. Aborts.
    spec initialize_storage_refund(_: &signer) {
        aborts_if true;
    }

    /// Aborts if module event feature is not enabled.
    spec emit_fee_statement {}
}
