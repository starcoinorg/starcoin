address StarcoinFramework {
/// `VMConfig` keep track of VM related configuration, like gas schedule.
module VMConfig {
    use StarcoinFramework::Config;
    use StarcoinFramework::Signer;
    use StarcoinFramework::CoreAddresses;
    use StarcoinFramework::Vector;
    use StarcoinFramework::ChainId;
    spec module {
        pragma verify = false;
        pragma aborts_if_is_strict;
    }

    /// The struct to hold all config data needed to operate the VM.
    /// * gas_schedule: Cost of running the VM.
    struct VMConfig has copy, drop, store {
        gas_schedule: GasSchedule,
    }

    /// The gas schedule keeps two separate schedules for the gas:
    /// * The instruction_schedule: This holds the gas for each bytecode instruction.
    /// * The native_schedule: This holds the gas for used (per-byte operated over) for each native
    ///   function.
    /// A couple notes:
    /// 1. In the case that an instruction is deleted from the bytecode, that part of the cost schedule
    ///    still needs to remain the same; once a slot in the table is taken by an instruction, that is its
    ///    slot for the rest of time (since that instruction could already exist in a module on-chain).
    /// 2. The initialization of the module will publish the instruction table to the genesis
    ///    address, and will preload the vector with the gas schedule for instructions. The VM will then
    ///    load this into memory at the startup of each block.
    struct GasSchedule has copy, drop, store {
        instruction_schedule: vector<u8>,
        native_schedule: vector<u8>,
        gas_constants: GasConstants,
    }

    /// The gas constants contains all kind of constants used in gas calculation.
    struct GasConstants has copy, drop, store {
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
        /// The max transaction size in bytes that a transaction can have.
        max_transaction_size_in_bytes: u64,
        /// gas unit scaling factor.
        gas_unit_scaling_factor: u64,
        /// default account size.
        default_account_size: u64,
    }

    /// The  `GasCost` tracks:
    /// - instruction cost: how much time/computational power is needed to perform the instruction
    /// - memory cost: how much memory is required for the instruction, and storage overhead
    struct GasCost has copy, drop, store {
        instruction_gas: u64,
        memory_gas: u64,
    }

    public fun instruction_schedule(): vector<GasCost> {
        let table = Vector::empty();

        // POP
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // RET
        Vector::push_back(&mut table, new_gas_cost(638, 1));
        // BR_TRUE
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // BR_FALSE
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // BRANCH
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U64
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_CONST
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_TRUE
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_FALSE
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // COPY_LOC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MOVE_LOC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // ST_LOC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORROW_LOC
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // IMM_BORROW_LOC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORROW_FIELD
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // IMM_BORROW_FIELD
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // CALL
        Vector::push_back(&mut table, new_gas_cost(1132, 1));
        // PACK
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // UNPACK
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // READ_REF
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // WRITE_REF
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // ADD
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // SUB
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUL
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MOD
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // DIV
        Vector::push_back(&mut table, new_gas_cost(3, 1));
        // BIT_OR
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // BIT_AND
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // XOR
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // OR
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // AND
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // NOT
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // EQ
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // NEQ
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LT
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // GT
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LE
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // GE
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // ABORT
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // NOP
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // EXISTS
        Vector::push_back(&mut table, new_gas_cost(41, 1));
        // MUT_BORROW_GLOBAL
        Vector::push_back(&mut table, new_gas_cost(21, 1));
        // IML_BORROW_GLOBAL
        Vector::push_back(&mut table, new_gas_cost(23, 1));
        // MOVE_FROM
        Vector::push_back(&mut table, new_gas_cost(459, 1));
        // MOVE_TO
        Vector::push_back(&mut table, new_gas_cost(13, 1));
        // FREEZE_REF
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // SHL
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // SHR
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U8
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U128
        Vector::push_back(&mut table, new_gas_cost(1, 1));

        // CAST_U8
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // CAST_U64
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // CAST_U128
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORORW_FIELD_GENERIC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // IMM_BORORW_FIELD_GENERIC
        Vector::push_back(&mut table, new_gas_cost(1, 1));
        // CALL_GENERIC
        Vector::push_back(&mut table, new_gas_cost(582, 1));
        // PACK_GENERIC
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // UNPACK_GENERIC
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        // EXISTS_GENERIC
        Vector::push_back(&mut table, new_gas_cost(34, 1));
        // MUT_BORROW_GLOBAL_GENERIC
        Vector::push_back(&mut table, new_gas_cost(15, 1));
        // IMM_BORROW_GLOBAL_GENERIC
        Vector::push_back(&mut table, new_gas_cost(14, 1));
        // MOVE_FROM_GENERIC
        Vector::push_back(&mut table, new_gas_cost(13, 1));
        // MOVE_TO_GENERIC
        Vector::push_back(&mut table, new_gas_cost(27, 1));

        // VEC_PACK
        Vector::push_back(&mut table, new_gas_cost(84, 1));
        // VEC_LEN
        Vector::push_back(&mut table, new_gas_cost(98, 1));
        // VEC_IMM_BORROW
        Vector::push_back(&mut table, new_gas_cost(1334, 1));
        // VEC_MUT_BORROW
        Vector::push_back(&mut table, new_gas_cost(1902, 1));
        // VEC_PUSH_BACK
        Vector::push_back(&mut table, new_gas_cost(53, 1));
        // VEC_POP_BACK
        Vector::push_back(&mut table, new_gas_cost(227, 1));
        // VEC_UNPACK
        Vector::push_back(&mut table, new_gas_cost(572, 1));
        // VEC_SWAP
        Vector::push_back(&mut table, new_gas_cost(1436, 1));
        table
    }

    public fun native_schedule(): vector<GasCost> {
        let table = Vector::empty();
        //Hash::sha2_256 0
        Vector::push_back(&mut table, new_gas_cost(21, 1));
        //Hash::sha3_256 1
        Vector::push_back(&mut table, new_gas_cost(64, 1));
        //Signature::ed25519_verify 2
        Vector::push_back(&mut table, new_gas_cost(61, 1));
        //ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated
        Vector::push_back(&mut table, new_gas_cost(3351, 1));
        //BSC::to_bytes 4
        Vector::push_back(&mut table, new_gas_cost(181, 1));
        //Vector::length 5
        Vector::push_back(&mut table, new_gas_cost(98, 1));
        //Vector::empty 6
        Vector::push_back(&mut table, new_gas_cost(84, 1));
        //Vector::borrow 7
        Vector::push_back(&mut table, new_gas_cost(1334, 1));
        //Vector::borrow_mut 8
        Vector::push_back(&mut table, new_gas_cost(1902, 1));
        //Vector::push_back 9
        Vector::push_back(&mut table, new_gas_cost(53, 1));
        //Vector::pop_back 10
        Vector::push_back(&mut table, new_gas_cost(227, 1));
        //Vector::destory_empty 11
        Vector::push_back(&mut table, new_gas_cost(572, 1));
        //Vector::swap 12
        Vector::push_back(&mut table, new_gas_cost(1436, 1));
        //Signature::ed25519_validate_pubkey 13
        Vector::push_back(&mut table, new_gas_cost(26, 1));
        //Signer::borrow_address 14
        Vector::push_back(&mut table, new_gas_cost(353, 1));
        //Account::creator_signer 15
        Vector::push_back(&mut table, new_gas_cost(24, 1));
        //Account::destroy_signer 16
        Vector::push_back(&mut table, new_gas_cost(212, 1));
        //Event::emit_event 17
        Vector::push_back(&mut table, new_gas_cost(52, 1));
        //BCS::to_address 18
        Vector::push_back(&mut table, new_gas_cost(26, 1));
        //Token::name_of 19
        Vector::push_back(&mut table, new_gas_cost(2002, 1));
        //Hash::keccak_256 20
        Vector::push_back(&mut table, new_gas_cost(64, 1));
        //Hash::ripemd160 21
        Vector::push_back(&mut table, new_gas_cost(64, 1));
        //Signature::native_ecrecover 22
        Vector::push_back(&mut table, new_gas_cost(128, 1));
        //U256::from_bytes 23
        Vector::push_back(&mut table, new_gas_cost(2, 1));
        //U256::add 24
        Vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::sub 25
        Vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::mul 26
        Vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::div 27
        Vector::push_back(&mut table, new_gas_cost(10, 1));
        // U256::rem 28
        Vector::push_back(&mut table, new_gas_cost(4, 1));
        // U256::pow 29
        Vector::push_back(&mut table, new_gas_cost(8, 1));
        // TODO: settle down the gas cost
        // Vector::append 30
        Vector::push_back(&mut table, new_gas_cost(40, 1));
        // Vector::remove 31
        Vector::push_back(&mut table, new_gas_cost(20, 1));
        // Vector::reverse 32
        Vector::push_back(&mut table, new_gas_cost(10, 1));

        table
    }

    public fun gas_constants(): GasConstants {
        let min_price_per_gas_unit: u64 = if (ChainId::is_test()) { 0 }  else { 1 };
        let maximum_number_of_gas_units: u64 = 40000000;//must less than base_block_gas_limit

        if (ChainId::is_test() || ChainId::is_dev() || ChainId::is_halley()) {
            maximum_number_of_gas_units = maximum_number_of_gas_units * 10
        };
        GasConstants {
            global_memory_per_byte_cost: 4,
            global_memory_per_byte_write_cost: 9,
            min_transaction_gas_units: 600,
            large_transaction_cutoff: 600,
            instrinsic_gas_per_byte: 8,
            maximum_number_of_gas_units,
            min_price_per_gas_unit,
            max_price_per_gas_unit: 10000,
            max_transaction_size_in_bytes: 1024 * 128,
            gas_unit_scaling_factor: 1,
            default_account_size: 800,
        }
    }

    fun new_gas_cost(instr_gas: u64, mem_gas: u64): GasCost {
        GasCost {
            instruction_gas: instr_gas,
            memory_gas: mem_gas,
        }
    }


    /// Create a new vm config, mainly used in DAO.
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

    /// Initialize the table under the genesis account
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

    spec initialize {
        aborts_if Signer::address_of(account) != CoreAddresses::SPEC_GENESIS_ADDRESS();
        aborts_if exists<Config::Config<VMConfig>>(Signer::address_of(account));
        aborts_if
            exists<Config::ModifyConfigCapabilityHolder<VMConfig>>(
                Signer::address_of(account),
            );
        ensures exists<Config::Config<VMConfig>>(Signer::address_of(account));
        ensures
            exists<Config::ModifyConfigCapabilityHolder<VMConfig>>(
                Signer::address_of(account),
            );
    }
}
}