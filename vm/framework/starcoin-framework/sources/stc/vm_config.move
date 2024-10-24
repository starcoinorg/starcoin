/// `VMConfig` keep track of VM related configuration, like gas schedule.
module starcoin_framework::vm_config {
    use std::vector;
    use starcoin_framework::system_addresses;
    use starcoin_framework::stc_util;
    use starcoin_framework::on_chain_config;

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
        let table = vector::empty();

        // POP
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // RET
        vector::push_back(&mut table, new_gas_cost(638, 1));
        // BR_TRUE
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // BR_FALSE
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // BRANCH
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U64
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_CONST
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_TRUE
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_FALSE
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // COPY_LOC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MOVE_LOC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // ST_LOC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORROW_LOC
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // IMM_BORROW_LOC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORROW_FIELD
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // IMM_BORROW_FIELD
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // CALL
        vector::push_back(&mut table, new_gas_cost(1132, 1));
        // PACK
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // UNPACK
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // READ_REF
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // WRITE_REF
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // ADD
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // SUB
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUL
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MOD
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // DIV
        vector::push_back(&mut table, new_gas_cost(3, 1));
        // BIT_OR
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // BIT_AND
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // XOR
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // OR
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // AND
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // NOT
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // EQ
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // NEQ
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LT
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // GT
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LE
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // GE
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // ABORT
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // NOP
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // EXISTS
        vector::push_back(&mut table, new_gas_cost(41, 1));
        // MUT_BORROW_GLOBAL
        vector::push_back(&mut table, new_gas_cost(21, 1));
        // IML_BORROW_GLOBAL
        vector::push_back(&mut table, new_gas_cost(23, 1));
        // MOVE_FROM
        vector::push_back(&mut table, new_gas_cost(459, 1));
        // MOVE_TO
        vector::push_back(&mut table, new_gas_cost(13, 1));
        // FREEZE_REF
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // SHL
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // SHR
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U8
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // LD_U128
        vector::push_back(&mut table, new_gas_cost(1, 1));

        // CAST_U8
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // CAST_U64
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // CAST_U128
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // MUT_BORORW_FIELD_GENERIC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // IMM_BORORW_FIELD_GENERIC
        vector::push_back(&mut table, new_gas_cost(1, 1));
        // CALL_GENERIC
        vector::push_back(&mut table, new_gas_cost(582, 1));
        // PACK_GENERIC
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // UNPACK_GENERIC
        vector::push_back(&mut table, new_gas_cost(2, 1));
        // EXISTS_GENERIC
        vector::push_back(&mut table, new_gas_cost(34, 1));
        // MUT_BORROW_GLOBAL_GENERIC
        vector::push_back(&mut table, new_gas_cost(15, 1));
        // IMM_BORROW_GLOBAL_GENERIC
        vector::push_back(&mut table, new_gas_cost(14, 1));
        // MOVE_FROM_GENERIC
        vector::push_back(&mut table, new_gas_cost(13, 1));
        // MOVE_TO_GENERIC
        vector::push_back(&mut table, new_gas_cost(27, 1));

        // VEC_PACK
        vector::push_back(&mut table, new_gas_cost(84, 1));
        // VEC_LEN
        vector::push_back(&mut table, new_gas_cost(98, 1));
        // VEC_IMM_BORROW
        vector::push_back(&mut table, new_gas_cost(1334, 1));
        // VEC_MUT_BORROW
        vector::push_back(&mut table, new_gas_cost(1902, 1));
        // VEC_PUSH_BACK
        vector::push_back(&mut table, new_gas_cost(53, 1));
        // VEC_POP_BACK
        vector::push_back(&mut table, new_gas_cost(227, 1));
        // VEC_UNPACK
        vector::push_back(&mut table, new_gas_cost(572, 1));
        // VEC_SWAP
        vector::push_back(&mut table, new_gas_cost(1436, 1));
        table
    }

    public fun native_schedule(): vector<GasCost> {
        let table = vector::empty();
        //Hash::sha2_256 0
        vector::push_back(&mut table, new_gas_cost(21, 1));
        //Hash::sha3_256 1
        vector::push_back(&mut table, new_gas_cost(64, 1));
        //Signature::ed25519_verify 2
        vector::push_back(&mut table, new_gas_cost(61, 1));
        //ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated
        vector::push_back(&mut table, new_gas_cost(3351, 1));
        //BSC::to_bytes 4
        vector::push_back(&mut table, new_gas_cost(181, 1));
        //vector::length 5
        vector::push_back(&mut table, new_gas_cost(98, 1));
        //vector::empty 6
        vector::push_back(&mut table, new_gas_cost(84, 1));
        //vector::borrow 7
        vector::push_back(&mut table, new_gas_cost(1334, 1));
        //vector::borrow_mut 8
        vector::push_back(&mut table, new_gas_cost(1902, 1));
        //vector::push_back 9
        vector::push_back(&mut table, new_gas_cost(53, 1));
        //vector::pop_back 10
        vector::push_back(&mut table, new_gas_cost(227, 1));
        //vector::destory_empty 11
        vector::push_back(&mut table, new_gas_cost(572, 1));
        //vector::swap 12
        vector::push_back(&mut table, new_gas_cost(1436, 1));
        //Signature::ed25519_validate_pubkey 13
        vector::push_back(&mut table, new_gas_cost(26, 1));
        //Signer::borrow_address 14
        vector::push_back(&mut table, new_gas_cost(353, 1));
        //Account::creator_signer 15
        vector::push_back(&mut table, new_gas_cost(24, 1));
        //Account::destroy_signer 16
        vector::push_back(&mut table, new_gas_cost(212, 1));
        //Event::emit_event 17
        vector::push_back(&mut table, new_gas_cost(52, 1));
        //BCS::to_address 18
        vector::push_back(&mut table, new_gas_cost(26, 1));
        //Token::name_of 19
        vector::push_back(&mut table, new_gas_cost(2002, 1));
        //Hash::keccak_256 20
        vector::push_back(&mut table, new_gas_cost(64, 1));
        //Hash::ripemd160 21
        vector::push_back(&mut table, new_gas_cost(64, 1));
        //Signature::native_ecrecover 22
        vector::push_back(&mut table, new_gas_cost(128, 1));
        //U256::from_bytes 23
        vector::push_back(&mut table, new_gas_cost(2, 1));
        //U256::add 24
        vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::sub 25
        vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::mul 26
        vector::push_back(&mut table, new_gas_cost(4, 1));
        //U256::div 27
        vector::push_back(&mut table, new_gas_cost(10, 1));
        // U256::rem 28
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // U256::pow 29
        vector::push_back(&mut table, new_gas_cost(8, 1));
        // TODO: settle down the gas cost
        // vector::append 30
        vector::push_back(&mut table, new_gas_cost(40, 1));
        // vector::remove 31
        vector::push_back(&mut table, new_gas_cost(20, 1));
        // vector::reverse 32
        vector::push_back(&mut table, new_gas_cost(10, 1));

        // Table::new_table_handle 33
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // Table::add_box 34
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // Table::borrow_box 35
        vector::push_back(&mut table, new_gas_cost(10, 1));
        // Table::remove_box 36
        vector::push_back(&mut table, new_gas_cost(8, 1));
        // Table::contains_box 37
        vector::push_back(&mut table, new_gas_cost(40, 1));
        // Table::destroy_empty_box 38
        vector::push_back(&mut table, new_gas_cost(20, 1));
        // Table::drop_unchecked_box 39
        vector::push_back(&mut table, new_gas_cost(73, 1));
        // string.check_utf8 40
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // string.sub_str 41
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // string.is_char_boundary 42
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // Table::string.index_of 43
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // FromBCS::from_bytes 44
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // Secp256k1::ecdsa_recover_internal 45
        vector::push_back(&mut table, new_gas_cost(4, 1));
        // vector::spawn_from 46
        vector::push_back(&mut table, new_gas_cost(4, 1));

        table
    }

    public fun gas_constants(): GasConstants {
        let min_price_per_gas_unit: u64 = if (stc_util::is_net_test()) { 0 }  else { 1 };
        let maximum_number_of_gas_units: u64 = 40000000;//must less than base_block_gas_limit

        if (stc_util::is_net_test() || stc_util::is_net_dev() || stc_util::is_net_halley()) {
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
        // CoreAddresses::assert_genesis_address(account);
        system_addresses::assert_starcoin_framework(account);
        on_chain_config::publish_new_config<VMConfig>(
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
        use std::signer;
        use starcoin_framework::on_chain_config;

        aborts_if signer::address_of(account) != system_addresses::get_starcoin_framework();
        aborts_if exists<on_chain_config::Config<VMConfig>>(signer::address_of(account));
        aborts_if
            exists<on_chain_config::ModifyConfigCapabilityHolder<VMConfig>>(
                signer::address_of(account),
            );
        ensures exists<on_chain_config::Config<VMConfig>>(signer::address_of(account));
        ensures
            exists<on_chain_config::ModifyConfigCapabilityHolder<VMConfig>>(
                signer::address_of(account),
            );
    }
}