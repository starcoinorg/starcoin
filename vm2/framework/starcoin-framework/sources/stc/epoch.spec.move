/// The module provide epoch functionality for starcoin.
spec starcoin_framework::epoch {
    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    spec initialize(account: &signer) {
        use std::timestamp;
        use std::signer;
        use starcoin_framework::on_chain_config;

        // aborts_if !Timestamp::is_genesis();
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<timestamp::CurrentTimeMicroseconds>(system_addresses::get_starcoin_framework());
        aborts_if !exists<on_chain_config::Config<ConsensusConfig>>(system_addresses::get_starcoin_framework());

        aborts_if exists<Epoch>(signer::address_of(account));
        aborts_if exists<EpochData>(signer::address_of(account));
    }

    spec compute_next_block_time_target {
        pragma verify = false;
    }

    spec adjust_epoch {
        use std::signer;

        pragma verify = false; //timeout
        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if !exists<Epoch>(signer::address_of(account));
        aborts_if global<Epoch>(signer::address_of(account)).max_uncles_per_block < uncles;
        aborts_if exists<EpochData>(signer::address_of(account));
        aborts_if block_number == global<Epoch>(signer::address_of(account)).end_block_number && uncles != 0;
        // ...
    }

    spec adjust_gas_limit {
        pragma verify = false; //mul_div() timeout
    }


    spec compute_gas_limit {
        pragma verify = false; //mul_div() timeout
    }

    spec in_or_decrease_gas_limit(last_epoch_block_gas_limit: u64, percent: u64, min_block_gas_limit: u64): u64 {
        include math128::MulDivAbortsIf { a: last_epoch_block_gas_limit, b: percent, c: HUNDRED };
        // aborts_if math64::spec_mul_div() > MAX_U64;
    }


    spec update_epoch_data {
        aborts_if !new_epoch && epoch_data.total_reward + reward > MAX_U128;
        aborts_if !new_epoch && epoch_data.uncles + uncles > MAX_U64;
        aborts_if !new_epoch && epoch_data.total_gas + parent_gas_used > MAX_U128;
    }

    spec emit_epoch_event {
        aborts_if false;
    }

    spec start_time {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }


    spec uncles {
        aborts_if !exists<EpochData>(system_addresses::get_starcoin_framework());
    }

    spec total_gas {
        aborts_if !exists<EpochData>(system_addresses::get_starcoin_framework());
    }


    spec block_gas_limit {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }

    spec start_block_number {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }

    spec end_block_number {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }

    spec number {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }

    spec block_time_target {
        aborts_if !exists<Epoch>(system_addresses::get_starcoin_framework());
    }
}