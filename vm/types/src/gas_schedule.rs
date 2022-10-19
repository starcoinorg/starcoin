pub use move_core_types::gas_schedule::*;
pub use move_vm_types::gas_schedule::{
    calculate_intrinsic_gas, new_from_instructions, zero_cost_schedule, GasStatus,
};
use once_cell::sync::Lazy;

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum NativeCostIndex {
    SHA2_256 = 0,
    SHA3_256 = 1,
    ED25519_VERIFY = 2,
    ED25519_THRESHOLD_VERIFY = 3,
    BCS_TO_BYTES = 4,
    LENGTH = 5,
    EMPTY = 6,
    BORROW = 7,
    BORROW_MUT = 8,
    PUSH_BACK = 9,
    POP_BACK = 10,
    DESTROY_EMPTY = 11,
    SWAP = 12,
    ED25519_VALIDATE_KEY = 13,
    SIGNER_BORROW = 14,
    CREATE_SIGNER = 15,
    DESTROY_SIGNER = 16,
    EMIT_EVENT = 17,
    BCS_TO_ADDRESS = 18,
    TOKEN_NAME_OF = 19,
    KECCAK_256 = 20,
    RIPEMD160 = 21,
    ECRECOVER = 22,
    U256_FROM_BYTES = 23,
    U256_ADD = 24,
    U256_SUB = 25,
    U256_MUL = 26,
    U256_DIV = 27,
    U256_REM = 28,
    U256_POW = 29,
    VEC_APPEND = 30,
    VEC_REMOVE = 31,
    VEC_REVERSE = 32,
    TABLE_NEW = 33,
    TABLE_INSERT = 34,
    TABLE_BORROW = 35,
    TABLE_BORROW_MUT = 36,
    TABLE_REMOVE = 37,
    TABLE_CONTAINS = 38,
    TABLE_DESTROY = 39,
    TABLE_DROP = 40,
    STRING_CHECK_UT8 = 41,
    STRING_SUB_STR = 42,
    SRING_CHAR_BOUNDARY = 43,
    STRING_INDEX_OF = 44,
}

impl NativeCostIndex {
    //note: should change this value when add new native function.
    pub const NUMBER_OF_NATIVE_FUNCTIONS: usize = 45;
}

static G_MAX_TRANSACTION_SIZE_IN_BYTES_V1: u64 = 4096 * 10;
static G_MAX_TRANSACTION_SIZE_IN_BYTES_V2: u64 = 60000;
static G_MAX_TRANSACTION_SIZE_IN_BYTES_V3: u64 = 128 * 1024;

/// For V1 all accounts will be ~800 bytes
pub static G_DEFAULT_ACCOUNT_SIZE: Lazy<AbstractMemorySize<GasCarrier>> =
    Lazy::new(|| AbstractMemorySize::new(800));

/// Any transaction over this size will be charged `INTRINSIC_GAS_PER_BYTE` per byte
pub static G_LARGE_TRANSACTION_CUTOFF: Lazy<AbstractMemorySize<GasCarrier>> =
    Lazy::new(|| AbstractMemorySize::new(600));

pub static G_GAS_CONSTANTS_V1: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: InternalGasUnits::new(4),
        global_memory_per_byte_write_cost: InternalGasUnits::new(9),
        min_transaction_gas_units: InternalGasUnits::new(600),
        large_transaction_cutoff: *G_LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: InternalGasUnits::new(8),
        maximum_number_of_gas_units: GasUnits::new(40_000_000), //must less than base_block_gas_limit
        min_price_per_gas_unit: GasPrice::new(1),
        max_price_per_gas_unit: GasPrice::new(10_000),
        max_transaction_size_in_bytes: G_MAX_TRANSACTION_SIZE_IN_BYTES_V1, // to pass stdlib_upgrade
        gas_unit_scaling_factor: 1,
        default_account_size: *G_DEFAULT_ACCOUNT_SIZE,
    }
});

pub static G_GAS_CONSTANTS_V2: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: InternalGasUnits::new(4),
        global_memory_per_byte_write_cost: InternalGasUnits::new(9),
        min_transaction_gas_units: InternalGasUnits::new(600),
        large_transaction_cutoff: *G_LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: InternalGasUnits::new(8),
        maximum_number_of_gas_units: GasUnits::new(40_000_000), //must less than base_block_gas_limit
        min_price_per_gas_unit: GasPrice::new(1),
        max_price_per_gas_unit: GasPrice::new(10_000),
        max_transaction_size_in_bytes: G_MAX_TRANSACTION_SIZE_IN_BYTES_V2, // to pass stdlib_upgrade
        gas_unit_scaling_factor: 1,
        default_account_size: *G_DEFAULT_ACCOUNT_SIZE,
    }
});
pub static G_GAS_CONSTANTS_V3: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: InternalGasUnits::new(4),
        global_memory_per_byte_write_cost: InternalGasUnits::new(9),
        min_transaction_gas_units: InternalGasUnits::new(600),
        large_transaction_cutoff: *G_LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: InternalGasUnits::new(8),
        maximum_number_of_gas_units: GasUnits::new(40_000_000), //must less than base_block_gas_limit
        min_price_per_gas_unit: GasPrice::new(1),
        max_price_per_gas_unit: GasPrice::new(10_000),
        max_transaction_size_in_bytes: G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        gas_unit_scaling_factor: 1,
        default_account_size: *G_DEFAULT_ACCOUNT_SIZE,
    }
});

pub static G_TEST_GAS_CONSTANTS: Lazy<GasConstants> = Lazy::new(|| {
    GasConstants {
        global_memory_per_byte_cost: InternalGasUnits::new(4),
        global_memory_per_byte_write_cost: InternalGasUnits::new(9),
        min_transaction_gas_units: InternalGasUnits::new(600),
        large_transaction_cutoff: *G_LARGE_TRANSACTION_CUTOFF,
        intrinsic_gas_per_byte: InternalGasUnits::new(8),
        maximum_number_of_gas_units: GasUnits::new(40_000_000 * 10), //must less than base_block_gas_limit
        min_price_per_gas_unit: GasPrice::new(0),
        max_price_per_gas_unit: GasPrice::new(10_000),
        max_transaction_size_in_bytes: G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        gas_unit_scaling_factor: 1,
        default_account_size: *G_DEFAULT_ACCOUNT_SIZE,
    }
});

pub static G_LATEST_GAS_CONSTANTS: Lazy<GasConstants> = Lazy::new(|| G_GAS_CONSTANTS_V3.clone());

pub fn latest_cost_table(gas_constants: GasConstants) -> CostTable {
    CostTable {
        instruction_table: crate::on_chain_config::G_LATEST_INSTRUCTION_TABLE.clone(),
        native_table: crate::on_chain_config::G_LATEST_NATIVE_TABLE.clone(),
        gas_constants,
    }
}

/// only used in starcoin vm when init genesis
pub static G_LATEST_GAS_SCHEDULE: Lazy<CostTable> =
    Lazy::new(|| latest_cost_table(G_LATEST_GAS_CONSTANTS.clone()));
