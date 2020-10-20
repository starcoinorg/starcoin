address 0x1 {
module VMConfig {
    use 0x1::Config;
    use 0x1::Signer;
    use 0x1::CoreAddresses;

    spec module {
        pragma verify;
        pragma aborts_if_is_strict;
    }

    // The struct to hold all config data needed to operate the VM.
    // * gas_schedule: Cost of running the VM.
    struct VMConfig {
        gas_schedule: GasSchedule,
    }

    // The gas schedule keeps two separate schedules for the gas:
    // * The instruction_schedule: This holds the gas for each bytecode instruction.
    // * The native_schedule: This holds the gas for used (per-byte operated over) for each native
    //   function.
    // A couple notes:
    // 1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
    //    still needs to remain the same; once a slot in the table is taken by an instruction, that is its
    //    slot for the rest of time (since that instruction could already exist in a module on-chain).
    // 2. The initialization of the module will publish the instruction table to the genesis
    //    address, and will preload the vector with the gas schedule for instructions. The VM will then
    //    load this into memory at the startup of each block.
    struct GasSchedule {
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        gas_constants: GasConstants,
    }

    struct GasConstants {
        /// The cost per-byte written to global storage.
        global_memory_per_byte_cost: u64,
        /// The cost per-byte written to storage.
        global_memory_per_byte_write_cost: u64,
        /// We charge one unit of gas per-byte for the first 600 bytes
        min_transaction_gas_units: u64,
        /// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
        large_transaction_cutoff: u64,
        /// The units of gas that should be charged per byte for every transaction.
        instrinsic_gas_per_byte: u64,
        /// 1 nanosecond should equal one unit of computational gas. We bound the maximum
        /// computational time of any given transaction at 10 milliseconds. We want this number and
        /// `MAX_PRICE_PER_GAS_UNIT` to always satisfy the inequality that
        ///         MAXIMUM_NUMBER_OF_GAS_UNITS * MAX_PRICE_PER_GAS_UNIT < min(u64::MAX, GasUnits<GasCarrier>::MAX)
        maximum_number_of_gas_units: u64,
        /// The minimum gas price that a transaction can be submitted with.
        min_price_per_gas_unit: u64,
        /// The maximum gas unit price that a transaction can be submitted with.
        max_price_per_gas_unit: u64,
        max_transaction_size_in_bytes: u64,
        gas_unit_scaling_factor: u64,
        default_account_size: u64,
    }

    public fun new_vm_config(
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        global_memory_per_byte_cost: u64,
        global_memory_per_byte_write_cost: u64,
        min_transaction_gas_units: u64,
        large_transaction_cutoff: u64,
        instrinsic_gas_per_byte: u64,
        maximum_number_of_gas_units: u64,
        min_price_per_gas_unit: u64,
        max_price_per_gas_unit: u64,
        max_transaction_size_in_bytes: u64,
        gas_unit_scaling_factor: u64,
        default_account_size: u64,
    ): VMConfig {
        let gas_constants = GasConstants {
            global_memory_per_byte_cost,
            global_memory_per_byte_write_cost,
            min_transaction_gas_units,
            large_transaction_cutoff,
            instrinsic_gas_per_byte,
            maximum_number_of_gas_units,
            min_price_per_gas_unit,
            max_price_per_gas_unit,
            max_transaction_size_in_bytes,
            gas_unit_scaling_factor,
            default_account_size,
        };
        VMConfig {
            gas_schedule: GasSchedule { instruction_schedule, native_schedule, gas_constants },
        }
    }

    // Initialize the table under the genesis account
    public fun initialize(
        account: &signer,
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        global_memory_per_byte_cost: u64,
        global_memory_per_byte_write_cost: u64,
        min_transaction_gas_units: u64,
        large_transaction_cutoff: u64,
        instrinsic_gas_per_byte: u64,
        maximum_number_of_gas_units: u64,
        min_price_per_gas_unit: u64,
        max_price_per_gas_unit: u64,
        max_transaction_size_in_bytes: u64,
        gas_unit_scaling_factor: u64,
        default_account_size: u64,
    ) {
        CoreAddresses::assert_genesis_address(account);
        Config::publish_new_config<VMConfig>(
            account,
            new_vm_config(
                instruction_schedule,
                native_schedule,
                global_memory_per_byte_cost,
                global_memory_per_byte_write_cost,
                min_transaction_gas_units,
                large_transaction_cutoff,
                instrinsic_gas_per_byte,
                maximum_number_of_gas_units,
                min_price_per_gas_unit,
                max_price_per_gas_unit,
                max_transaction_size_in_bytes,
                gas_unit_scaling_factor,
                default_account_size,
            ),
        );
    }

    spec fun initialize {
        aborts_if Signer::spec_address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<Config::Config<VMConfig>>(Signer::spec_address_of(account));
        aborts_if
            exists<Config::ModifyConfigCapabilityHolder<VMConfig>>(
                Signer::spec_address_of(account),
            );
        ensures exists<Config::Config<VMConfig>>(Signer::spec_address_of(account));
        ensures
            exists<Config::ModifyConfigCapabilityHolder<VMConfig>>(
                Signer::spec_address_of(account),
            );
    }
}
}