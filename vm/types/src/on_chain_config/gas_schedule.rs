use crate::gas_schedule::{
    G_MAX_TRANSACTION_SIZE_IN_BYTES_V1, G_MAX_TRANSACTION_SIZE_IN_BYTES_V2,
    G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
};

use crate::on_chain_config::OnChainConfig;
use anyhow::{format_err, Result};
#[cfg(feature = "print_gas_info")]
use log::info;
use move_core_types::identifier::Identifier;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use starcoin_gas_algebra::{CostTable, GasConstants};
use std::collections::BTreeMap;

const GAS_SCHEDULE_MODULE_NAME: &str = "GasSchedule";
pub static G_GAS_SCHEDULE_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new(GAS_SCHEDULE_MODULE_NAME).unwrap());
pub static G_GAS_SCHEDULE_GAS_SCHEDULE: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("gas_schedule").unwrap());

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GasSchedule {
    pub entries: Vec<(String, u64)>,
}

impl GasSchedule {
    pub fn to_btree_map(self) -> BTreeMap<String, u64> {
        // TODO: what if the gas schedule contains duplicated entries?
        self.entries.into_iter().collect()
    }

    #[cfg(feature = "print_gas_info")]
    pub fn info(&self, message: &str) {
        let mut gas_info = String::from("GasSchedule info begin\n");
        gas_info.push_str(&format!("{}\n", message));
        self.entries.iter().for_each(|(key, value)| {
            gas_info.push_str(&format!("key = {}, gas value = {}\n", key, value));
        });
        gas_info.push_str("GasSchedule info end\n");
        info!("{}", gas_info);
    }

    /// check if there is any one of entry different from the other
    /// if it is, return true otherwise false
    pub fn is_different(&self, other: &Self) -> bool {
        let diff_len = self.entries.len() != other.entries.len();
        if diff_len {
            debug_assert!(
                !diff_len,
                "self.entries.len() = {} not the same as other.entries.len() = {}",
                self.entries.len(),
                other.entries.len()
            );
            return true;
        }
        self.entries
            .iter()
            .enumerate()
            .any(|(index, (key, value))| {
                let tuple = &other.entries[index];
                let diff = &tuple.0 != key || &tuple.1 != value;
                debug_assert!(
                    !diff,
                    "self.entries[{}] = {} not the same as other.entries[{}] = {}",
                    key, value, tuple.0, tuple.1
                );
                diff
            })
    }
}

// instruction_table_v1
pub fn instruction_gas_schedule_v1() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    vec![
        ("instr.pop".to_string(), gas_total(1, 1)),
        ("instr.ret".to_string(), gas_total(638, 1)),
        ("instr.br_true".to_string(), gas_total(1, 1)),
        ("instr.br_false".to_string(), gas_total(1, 1)),
        ("instr.branch".to_string(), gas_total(1, 1)),
        ("instr.ld_u64".to_string(), gas_total(1, 1)),
        ("instr.ld_const.per_byte".to_string(), gas_total(1, 1)),
        ("instr.ld_true".to_string(), gas_total(1, 1)),
        ("instr.ld_false".to_string(), gas_total(1, 1)),
        (
            "instr.copy_loc.per_abs_mem_unit".to_string(),
            gas_total(1, 1),
        ),
        (
            "instr.move_loc.per_abs_mem_unit".to_string(),
            gas_total(1, 1),
        ),
        ("instr.st_loc.per_abs_mem_unit".to_string(), gas_total(1, 1)),
        ("instr.mut_borrow_loc".to_string(), gas_total(2, 1)),
        ("instr.imm_borrow_loc".to_string(), gas_total(1, 1)),
        ("instr.mut_borrow_field".to_string(), gas_total(1, 1)),
        ("instr.imm_borrow_field".to_string(), gas_total(1, 1)),
        ("instr.call.per_arg".to_string(), gas_total(1132, 1)),
        ("instr.pack.per_abs_mem_unit".to_string(), gas_total(2, 1)),
        ("instr.unpack.per_abs_mem_unit".to_string(), gas_total(2, 1)),
        (
            "instr.read_ref.per_abs_mem_unit".to_string(),
            gas_total(1, 1),
        ),
        (
            "instr.write_ref.per_abs_mem_unit".to_string(),
            gas_total(1, 1),
        ),
        ("instr.add".to_string(), gas_total(1, 1)),
        ("instr.sub".to_string(), gas_total(1, 1)),
        ("instr.mul".to_string(), gas_total(1, 1)),
        ("instr.mod".to_string(), gas_total(1, 1)),
        ("instr.div".to_string(), gas_total(3, 1)),
        ("instr.bit_or".to_string(), gas_total(2, 1)),
        ("instr.bit_and".to_string(), gas_total(2, 1)),
        ("instr.xor".to_string(), gas_total(1, 1)),
        ("instr.or".to_string(), gas_total(2, 1)),
        ("instr.and".to_string(), gas_total(1, 1)),
        ("instr.not".to_string(), gas_total(1, 1)),
        ("instr.eq.per_abs_mem_unit".to_string(), gas_total(1, 1)),
        ("instr.neq.per_abs_mem_unit".to_string(), gas_total(1, 1)),
        ("instr.lt".to_string(), gas_total(1, 1)),
        ("instr.gt".to_string(), gas_total(1, 1)),
        ("instr.le".to_string(), gas_total(2, 1)),
        ("instr.ge".to_string(), gas_total(1, 1)),
        ("instr.abort".to_string(), gas_total(1, 1)),
        ("instr.nop".to_string(), gas_total(1, 1)),
        (
            "instr.exists.per_abs_mem_unit".to_string(),
            gas_total(41, 1),
        ),
        (
            "instr.mut_borrow_global.per_abs_mem_unit".to_string(),
            gas_total(21, 1),
        ),
        (
            "instr.imm_borrow_global.per_abs_mem_unit".to_string(),
            gas_total(23, 1),
        ),
        (
            "instr.move_from.per_abs_mem_unit".to_string(),
            gas_total(459, 1),
        ),
        (
            "instr.move_to.per_abs_mem_unit".to_string(),
            gas_total(13, 1),
        ),
        ("instr.freeze_ref".to_string(), gas_total(1, 1)),
        ("instr.shl".to_string(), gas_total(2, 1)),
        ("instr.shr".to_string(), gas_total(1, 1)),
        ("instr.ld_u8".to_string(), gas_total(1, 1)),
        ("instr.ld_u128".to_string(), gas_total(1, 1)),
        ("instr.cast_u8".to_string(), gas_total(2, 1)),
        ("instr.cast_u64".to_string(), gas_total(1, 1)),
        ("instr.cast_u128".to_string(), gas_total(1, 1)),
        (
            "instr.mut_borrow_field_generic.base".to_string(),
            gas_total(1, 1),
        ),
        (
            "instr.imm_borrow_field_generic.base".to_string(),
            gas_total(1, 1),
        ),
        ("instr.call_generic.per_arg".to_string(), gas_total(582, 1)),
        (
            "instr.pack_generic.per_abs_mem_unit".to_string(),
            gas_total(2, 1),
        ),
        (
            "instr.unpack_generic.per_abs_mem_unit".to_string(),
            gas_total(2, 1),
        ),
        (
            "instr.exists_generic.per_abs_mem_unit".to_string(),
            gas_total(34, 1),
        ),
        (
            "instr.mut_borrow_global_generic.per_abs_mem_unit".to_string(),
            gas_total(15, 1),
        ),
        (
            "instr.imm_borrow_global_generic.per_abs_mem_unit".to_string(),
            gas_total(14, 1),
        ),
        (
            "instr.move_from_generic.per_abs_mem_unit".to_string(),
            gas_total(13, 1),
        ),
        (
            "instr.move_to_generic.per_abs_mem_unit".to_string(),
            gas_total(27, 1),
        ),
    ]
}

// instruction_table_v2
pub fn instruction_gas_schedule_v2() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    let mut instrs = instruction_gas_schedule_v1();
    let mut instrs_delta = vec![
        ("instr.vec_pack.per_elem".to_string(), gas_total(84, 1)),
        ("instr.vec_len.base".to_string(), gas_total(98, 1)),
        ("instr.vec_imm_borrow.base".to_string(), gas_total(1334, 1)),
        ("instr.vec_mut_borrow.base".to_string(), gas_total(1902, 1)),
        (
            "instr.vec_push_back.per_abs_mem_unit".to_string(),
            gas_total(53, 1),
        ),
        ("instr.vec_pop_back.base".to_string(), gas_total(227, 1)),
        (
            "instr.vec_unpack.per_expected_elem".to_string(),
            gas_total(572, 1),
        ),
        ("instr.vec_swap.base".to_string(), gas_total(1436, 1)),
    ];
    instrs.append(&mut instrs_delta);
    instrs
}

// native_table_v1
pub fn native_gas_schedule_v1() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    vec![
        (
            "move_stdlib.hash.sha2_256.per_byte".to_string(),
            gas_total(21, 1),
        ),
        (
            "move_stdlib.hash.sha3_256.per_byte".to_string(),
            gas_total(64, 1),
        ),
        (
            "starcoin_natives.signature.ed25519_verify.per_byte".to_string(),
            gas_total(61, 1),
        ),
        // ED25519_THRESHOLD_VERIFY 3 this native function is deprecated
        (
            "move_stdlib.bcs.to_bytes.per_byte_serialized".to_string(),
            gas_total(181, 1),
        ),
        (
            "move_stdlib.vector.length.base".to_string(),
            gas_total(98, 1),
        ),
        (
            "move_stdlib.vector.empty.base".to_string(),
            gas_total(84, 1),
        ),
        (
            "move_stdlib.vector.borrow.base".to_string(),
            gas_total(1334, 1),
        ),
        // Vector::borrow_mut is same as Vector::borrow
        (
            "move_stdlib.vector.push_back.legacy_per_abstract_memory_unit".to_string(),
            gas_total(53, 1),
        ),
        (
            "move_stdlib.vector.pop_back.base".to_string(),
            gas_total(227, 1),
        ),
        (
            "move_stdlib.vector.destroy_empty.base".to_string(),
            gas_total(572, 1),
        ),
        (
            "move_stdlib.vector.swap.base".to_string(),
            gas_total(1436, 1),
        ),
        (
            "starcoin_natives.signature.ed25519_validate_key.per_byte".to_string(),
            gas_total(26, 1),
        ),
        (
            "move_stdlib.signer.borrow_address.base".to_string(),
            gas_total(353, 1),
        ),
        (
            "starcoin_natives.account.create_signer.base".to_string(),
            gas_total(24, 1),
        ),
        (
            "starcoin_natives.account.destroy_signer.base".to_string(),
            gas_total(212, 1),
        ),
        (
            "nursery.event.write_to_event_store.unit_cost".to_string(),
            gas_total(52, 1),
        ),
        (
            "move_stdlib.bcs.to_address.per_byte".to_string(),
            gas_total(26, 1),
        ),
        (
            "starcoin_natives.token.name_of.base".to_string(),
            gas_total(2002, 1),
        ),
    ]
}

// native_table_v2
pub fn native_gas_schedule_v2() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    let mut natives = native_gas_schedule_v1();
    let mut natives_delta = vec![(
        "starcoin_natives.hash.keccak256.per_byte".to_string(),
        gas_total(64, 1),
    )];
    natives.append(&mut natives_delta);
    natives
}

// v3_native_table
pub fn native_gas_schedule_v3() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    let mut natives = native_gas_schedule_v2();
    let mut natives_delta = vec![
        (
            "starcoin_natives.hash.ripemd160.per_byte".to_string(),
            gas_total(64, 1),
        ),
        (
            "starcoin_natives.signature.ec_recover.per_byte".to_string(),
            gas_total(128, 1),
        ),
        (
            "starcoin_natives.u256.from_bytes.per_byte".to_string(),
            gas_total(2, 1),
        ),
        (
            "starcoin_natives.u256.add.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "starcoin_natives.u256.sub.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "starcoin_natives.u256.mul.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "starcoin_natives.u256.div.base".to_string(),
            gas_total(10, 1),
        ),
        (
            "starcoin_natives.u256.rem.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "starcoin_natives.u256.pow.base".to_string(),
            gas_total(8, 1),
        ),
        (
            "move_stdlib.vector.append.legacy_per_abstract_memory_unit".to_string(),
            gas_total(40, 1),
        ),
        (
            "move_stdlib.vector.remove.legacy_per_abstract_memory_unit".to_string(),
            gas_total(20, 1),
        ),
        (
            "move_stdlib.vector.reverse.legacy_per_abstract_memory_unit".to_string(),
            gas_total(10, 1),
        ),
    ];
    natives.append(&mut natives_delta);
    natives
}

// v4_native_table
pub fn native_gas_schedule_v4() -> Vec<(String, u64)> {
    let gas_total = |x: u64, y: u64| -> u64 { x + y };
    let mut natives = native_gas_schedule_v3();
    let mut natives_delta = vec![
        ("table.new_table_handle.base".to_string(), gas_total(4, 1)),
        (
            "table.add_box.per_byte_serialized".to_string(),
            gas_total(4, 1),
        ),
        (
            "table.borrow_box.per_byte_serialized".to_string(),
            gas_total(10, 1),
        ),
        (
            "table.remove_box.per_byte_serialized".to_string(),
            gas_total(8, 1),
        ),
        (
            "table.contains_box.per_byte_serialized".to_string(),
            gas_total(40, 1),
        ),
        ("table.destroy_empty_box.base".to_string(), gas_total(20, 1)),
        (
            "table.drop_unchecked_box.base".to_string(),
            gas_total(73, 1),
        ),
        (
            "move_stdlib.string.check_utf8.per_byte".to_string(),
            gas_total(4, 1),
        ),
        (
            "move_stdlib.string.sub_string.per_byte".to_string(),
            gas_total(4, 1),
        ),
        (
            "move_stdlib.string.is_char_boundary.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "move_stdlib.string.index_of.per_byte_searched".to_string(),
            gas_total(4, 1),
        ),
        ("starcoin_natives.frombcs.base".to_string(), gas_total(4, 1)),
        (
            "starcoin_natives.secp256k1.base".to_string(),
            gas_total(4, 1),
        ),
        (
            "move_stdlib.vector.spawn_from.legacy_per_abstract_memory_unit".to_string(),
            gas_total(4, 1),
        ),
    ];
    natives.append(&mut natives_delta);
    natives
}

// G_GAS_CONSTANTS_V1
pub fn txn_gas_schedule_v1() -> Vec<(String, u64)> {
    vec![
        ("txn.global_memory_per_byte_cost".to_string(), 4),
        ("txn.global_memory_per_byte_write_cost".to_string(), 9),
        ("txn.min_transaction_gas_units".to_string(), 600),
        ("txn.large_transaction_cutoff".to_string(), 600),
        ("txn.intrinsic_gas_per_byte".to_string(), 8),
        ("txn.maximum_number_of_gas_units".to_string(), 40_000_000),
        ("txn.min_price_per_gas_unit".to_string(), 1),
        ("txn.max_price_per_gas_unit".to_string(), 10_000),
        (
            "txn.max_transaction_size_in_bytes".to_string(),
            G_MAX_TRANSACTION_SIZE_IN_BYTES_V1,
        ),
        ("txn.gas_unit_scaling_factor".to_string(), 1),
        ("txn.default_account_size".to_string(), 800),
    ]
}

// G_GAS_CONSTANTS_V2
pub fn txn_gas_schedule_v2() -> Vec<(String, u64)> {
    vec![
        ("txn.global_memory_per_byte_cost".to_string(), 4),
        ("txn.global_memory_per_byte_write_cost".to_string(), 9),
        ("txn.min_transaction_gas_units".to_string(), 600),
        ("txn.large_transaction_cutoff".to_string(), 600),
        ("txn.intrinsic_gas_per_byte".to_string(), 8),
        ("txn.maximum_number_of_gas_units".to_string(), 40_000_000),
        ("txn.min_price_per_gas_unit".to_string(), 1),
        ("txn.max_price_per_gas_unit".to_string(), 10_000),
        (
            "txn.max_transaction_size_in_bytes".to_string(),
            G_MAX_TRANSACTION_SIZE_IN_BYTES_V2,
        ),
        ("txn.gas_unit_scaling_factor".to_string(), 1),
        ("txn.default_account_size".to_string(), 800),
    ]
}

// G_GAS_CONSTANTS_V3
pub fn txn_gas_schedule_v3() -> Vec<(String, u64)> {
    vec![
        ("txn.global_memory_per_byte_cost".to_string(), 4),
        ("txn.global_memory_per_byte_write_cost".to_string(), 9),
        ("txn.min_transaction_gas_units".to_string(), 600),
        ("txn.large_transaction_cutoff".to_string(), 600),
        ("txn.intrinsic_gas_per_byte".to_string(), 8),
        ("txn.maximum_number_of_gas_units".to_string(), 40_000_000),
        ("txn.min_price_per_gas_unit".to_string(), 1),
        ("txn.max_price_per_gas_unit".to_string(), 10_000),
        (
            "txn.max_transaction_size_in_bytes".to_string(),
            G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        ),
        ("txn.gas_unit_scaling_factor".to_string(), 1),
        ("txn.default_account_size".to_string(), 800),
    ]
}

// G_GAS_CONSTANTS_TEST
pub fn txn_gas_schedule_test() -> Vec<(String, u64)> {
    vec![
        ("txn.global_memory_per_byte_cost".to_string(), 4),
        ("txn.global_memory_per_byte_write_cost".to_string(), 9),
        ("txn.min_transaction_gas_units".to_string(), 600),
        ("txn.large_transaction_cutoff".to_string(), 600),
        ("txn.intrinsic_gas_per_byte".to_string(), 8),
        (
            "txn.maximum_number_of_gas_units".to_string(),
            40_000_000 * 10,
        ),
        ("txn.min_price_per_gas_unit".to_string(), 0),
        ("txn.max_price_per_gas_unit".to_string(), 10_000),
        (
            "txn.max_transaction_size_in_bytes".to_string(),
            G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        ),
        ("txn.gas_unit_scaling_factor".to_string(), 1),
        ("txn.default_account_size".to_string(), 800),
    ]
}

impl OnChainConfig for GasSchedule {
    const MODULE_IDENTIFIER: &'static str = GAS_SCHEDULE_MODULE_NAME;
    const TYPE_IDENTIFIER: &'static str = GAS_SCHEDULE_MODULE_NAME;

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_gas_schedule = bcs_ext::from_bytes::<Self>(bytes).map_err(|e| {
            format_err!(
                "Failed first round of deserialization for GasSchedule: {}",
                e
            )
        })?;
        Ok(raw_gas_schedule)
    }
}

static G_INSTR_STRS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "instr.pop",
        "instr.ret",
        "instr.br_true",
        "instr.br_false",
        "instr.branch",
        "instr.ld_u64",
        "instr.ld_const.per_byte",
        "instr.ld_true",
        "instr.ld_false",
        "instr.copy_loc.per_abs_mem_unit",
        "instr.move_loc.per_abs_mem_unit",
        "instr.st_loc.per_abs_mem_unit",
        "instr.mut_borrow_loc",
        "instr.imm_borrow_loc",
        "instr.mut_borrow_field",
        "instr.imm_borrow_field",
        "instr.call.per_arg",
        "instr.pack.per_abs_mem_unit",
        "instr.unpack.per_abs_mem_unit",
        "instr.read_ref.per_abs_mem_unit",
        "instr.write_ref.per_abs_mem_unit",
        "instr.add",
        "instr.sub",
        "instr.mul",
        "instr.mod",
        "instr.div",
        "instr.bit_or",
        "instr.bit_and",
        "instr.xor",
        "instr.or",
        "instr.and",
        "instr.not",
        "instr.eq.per_abs_mem_unit",
        "instr.neq.per_abs_mem_unit",
        "instr.lt",
        "instr.gt",
        "instr.le",
        "instr.ge",
        "instr.abort",
        "instr.nop",
        "instr.exists.per_abs_mem_unit",
        "instr.mut_borrow_global.per_abs_mem_unit",
        "instr.imm_borrow_global.per_abs_mem_unit",
        "instr.move_from.per_abs_mem_unit",
        "instr.move_to.per_abs_mem_unit",
        "instr.freeze_ref",
        "instr.shl",
        "instr.shr",
        "instr.ld_u8",
        "instr.ld_u128",
        "instr.cast_u8",
        "instr.cast_u64",
        "instr.cast_u128",
        "instr.mut_borrow_field_generic.base",
        "instr.imm_borrow_field_generic.base",
        "instr.call_generic.per_arg",
        "instr.pack_generic.per_abs_mem_unit",
        "instr.unpack_generic.per_abs_mem_unit",
        "instr.exists_generic.per_abs_mem_unit",
        "instr.mut_borrow_global_generic.per_abs_mem_unit",
        "instr.imm_borrow_global_generic.per_abs_mem_unit",
        "instr.move_from_generic.per_abs_mem_unit",
        "instr.move_to_generic.per_abs_mem_unit",
        "instr.vec_pack.per_elem",
        "instr.vec_len.base",
        "instr.vec_imm_borrow.base",
        "instr.vec_mut_borrow.base",
        "instr.vec_push_back.per_abs_mem_unit",
        "instr.vec_pop_back.base",
        "instr.vec_unpack.per_expected_elem",
        "instr.vec_swap.base",
    ]
});

static G_NATIVE_STRS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "move_stdlib.hash.sha2_256.per_byte",
        "move_stdlib.hash.sha3_256.per_byte",
        "starcoin_natives.signature.ed25519_verify.per_byte",
        // ED25519_THRESHOLD_VERIFY 3 this native function is deprecated, ignore, use ""
        "",
        "move_stdlib.bcs.to_bytes.per_byte_serialized",
        "move_stdlib.vector.length.base",
        "move_stdlib.vector.empty.base",
        "move_stdlib.vector.borrow.base",
        // Vector::borrow_mut is same Vector::borrow ignore ""
        "",
        "move_stdlib.vector.push_back.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.pop_back.base",
        "move_stdlib.vector.destroy_empty.base",
        "move_stdlib.vector.swap.base",
        "starcoin_natives.signature.ed25519_validate_key.per_byte",
        "move_stdlib.signer.borrow_address.base",
        "starcoin_natives.account.create_signer.base",
        "starcoin_natives.account.destroy_signer.base",
        "nursery.event.write_to_event_store.unit_cost",
        "move_stdlib.bcs.to_address.per_byte",
        "starcoin_natives.token.name_of.base",
        "starcoin_natives.hash.keccak256.per_byte",
        "starcoin_natives.hash.ripemd160.per_byte",
        "starcoin_natives.signature.ec_recover.per_byte",
        "starcoin_natives.u256.from_bytes.per_byte",
        "starcoin_natives.u256.add.base",
        "starcoin_natives.u256.sub.base",
        "starcoin_natives.u256.mul.base",
        "starcoin_natives.u256.div.base",
        "starcoin_natives.u256.rem.base",
        "starcoin_natives.u256.pow.base",
        "move_stdlib.vector.append.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.remove.legacy_per_abstract_memory_unit",
        "move_stdlib.vector.reverse.legacy_per_abstract_memory_unit",
        "table.new_table_handle.base",
        "table.add_box.per_byte_serialized",
        "table.borrow_box.per_byte_serialized",
        "table.remove_box.per_byte_serialized",
        "table.contains_box.per_byte_serialized",
        "table.destroy_empty_box.base",
        "table.drop_unchecked_box.base",
        "move_stdlib.string.check_utf8.per_byte",
        "move_stdlib.string.sub_string.per_byte",
        "move_stdlib.string.is_char_boundary.base",
        "move_stdlib.string.index_of.per_byte_searched",
        "starcoin_natives.frombcs.base",
        "starcoin_natives.secp256k1.base",
        "move_stdlib.vector.spawn_from.legacy_per_abstract_memory_unit",
    ]
});

// https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move
impl From<&CostTable> for GasSchedule {
    fn from(cost_table: &CostTable) -> Self {
        let mut entries = vec![];

        // see vm/gas_algebra-ext/src/instr.rs
        // see https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#instruction_schedule
        let instrs = cost_table.instruction_table.clone();
        for (idx, cost) in instrs.into_iter().enumerate() {
            entries.push((G_INSTR_STRS[idx].to_string(), cost.total()));
        }
        entries.push(("instr.ld_u16".to_string(), 3));
        entries.push(("instr.ld_u32".to_string(), 2));
        entries.push(("instr.ld_u256".to_string(), 3));
        entries.push(("instr.cast_u16".to_string(), 3));
        entries.push(("instr.cast_u32".to_string(), 2));
        entries.push(("instr.cast_u256".to_string(), 3));

        // see vm/gas_algebra-ext/src/{move_stdlib.rs starcoin_framework.rs nursery.rs table.rs}
        // see https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
        let natives = cost_table.native_table.clone();
        for (idx, cost) in natives.into_iter().enumerate() {
            if G_NATIVE_STRS[idx].is_empty() {
                continue;
            }
            entries.push((G_NATIVE_STRS[idx].to_string(), cost.total()));
        }

        append_extra_gas_cost_for_nursery(&mut entries);
        append_extra_gas_cost_for_move_stdlib(&mut entries);
        append_extra_gas_cost_for_txn_gas_constants(&mut entries, &cost_table.gas_constants);
        append_extra_gas_cost_framework_upgrade(&mut entries);

        Self { entries }
    }
}

fn append_extra_gas_cost_for_nursery(entries: &mut Vec<(String, u64)>) {
    // native_table don't have these
    entries.push(("nursery.debug.print.base_cost".to_string(), 1));
    entries.push(("nursery.debug.print_stack_trace.base_cost".to_string(), 1));
}

fn append_extra_gas_cost_for_move_stdlib(entries: &mut Vec<(String, u64)>) {
    entries.push((
        "move_stdlib.hash.sha2_256.legacy_min_input_len".to_string(),
        1,
    ));
    entries.push((
        "move_stdlib.hash.sha3_256.legacy_min_input_len".to_string(),
        1,
    ));
    entries.push(("move_stdlib.bcs.to_bytes.failure".to_string(), 182));
    entries.push((
        "move_stdlib.bcs.to_bytes.legacy_min_output_size".to_string(),
        1,
    ));
}

fn append_extra_gas_cost_for_txn_gas_constants(
    entries: &mut Vec<(String, u64)>,
    txn: &GasConstants,
) {
    entries.push((
        "txn.global_memory_per_byte_cost".to_string(),
        txn.global_memory_per_byte_cost,
    ));
    entries.push((
        "txn.global_memory_per_byte_write_cost".to_string(),
        txn.global_memory_per_byte_write_cost,
    ));
    entries.push((
        "txn.min_transaction_gas_units".to_string(),
        txn.min_transaction_gas_units,
    ));
    entries.push((
        "txn.large_transaction_cutoff".to_string(),
        txn.large_transaction_cutoff,
    ));
    entries.push((
        "txn.intrinsic_gas_per_byte".to_string(),
        txn.intrinsic_gas_per_byte,
    ));
    entries.push((
        "txn.maximum_number_of_gas_units".to_string(),
        txn.maximum_number_of_gas_units,
    ));
    entries.push((
        "txn.min_price_per_gas_unit".to_string(),
        txn.min_price_per_gas_unit,
    ));
    entries.push((
        "txn.max_price_per_gas_unit".to_string(),
        txn.max_price_per_gas_unit,
    ));
    entries.push((
        "txn.max_transaction_size_in_bytes".to_string(),
        txn.max_transaction_size_in_bytes,
    ));
    entries.push((
        "txn.gas_unit_scaling_factor".to_string(),
        txn.gas_unit_scaling_factor,
    ));
    entries.push((
        "txn.default_account_size".to_string(),
        txn.default_account_size,
    ));
}

static G_MOVE_FRAMEWORK_UPGRADE_STRS: Lazy<Vec<&str>> = Lazy::new(|| {
    vec![
        "misc.abs_val.u8",
        "misc.abs_val.u16",
        "misc.abs_val.u32",
        "misc.abs_val.u64",
        "misc.abs_val.u128",
        "misc.abs_val.u256",
        "misc.abs_val.bool",
        "misc.abs_val.address",
        "misc.abs_val.struct",
        "misc.abs_val.vector",
        "misc.abs_val.reference",
        "misc.abs_val.per_u8_packed",
        "misc.abs_val.per_u16_packed",
        "misc.abs_val.per_u32_packed",
        "misc.abs_val.per_u64_packed",
        "misc.abs_val.per_u128_packed",
        "misc.abs_val.per_u256_packed",
        "misc.abs_val.per_bool_packed",
        "misc.abs_val.per_address_packed",
        "move_stdlib.hash.sha2_256.base",
        "move_stdlib.hash.sha2_256.per_byte",
        "move_stdlib.hash.sha2_256.legacy_min_input_len",
        "move_stdlib.hash.sha3_256.base",
        "move_stdlib.hash.sha3_256.per_byte",
        "move_stdlib.hash.sha3_256.legacy_min_input_len",
        "move_stdlib.bcs.to_bytes.per_byte_serialized",
        "move_stdlib.bcs.to_bytes.failure",
        "move_stdlib.bcs.to_bytes.legacy_min_output_size",
        // Note(Gas): this initial value is guesswork.
        "move_stdlib.signer.borrow_address.base",
        "move_stdlib.bcs.to_address.base",
        "move_stdlib.bcs.to_address.per_byte",
        // Note(Gas): these initial values are guesswork.
        "move_stdlib.string.check_utf8.base",
        "move_stdlib.string.check_utf8.per_byte",
        "move_stdlib.string.is_char_boundary.base",
        "move_stdlib.string.sub_string.base",
        "move_stdlib.string.sub_string.per_byte",
        "move_stdlib.string.index_of.base",
        "move_stdlib.string.index_of.per_byte_pattern",
        "move_stdlib.string.index_of.per_byte_searched",
        "move_stdlib.vector.spawn_from.base",
        /////////////////////////////////
        "table.new_table_handle.base",
        "table.add_box.per_byte_serialized",
        "table.borrow_box.per_byte_serialized",
        "table.remove_box.per_byte_serialized",
        "table.contains_box.per_byte_serialized",
        "table.destroy_empty_box.base",
        "table.drop_unchecked_box.base",
        "table.common.load.base",
        "table.common.load.base_new",
        "table.common.load.per_byte",
        "table.common.load.failure",
        "table.add_box.base",
        "table.borrow_box.base",
        "table.contains_box.base",
        "table.remove_box.base",
        ////////////////////////////////
        // starcoin framework
        "starcoin_framework.account.create_signer.base",
        "starcoin_framework.account.destroy_signer.base",
        "starcoin_framework.token.name_of.base",
        "starcoin_framework.hash.keccak256.base",
        "starcoin_framework.hash.keccak256.per_byte",
        "starcoin_framework.hash.ripemd160.base",
        "starcoin_framework.hash.ripemd160.per_byte",
        "starcoin_framework.signature.ec_recover.base",
        "starcoin_framework.signature.ec_recover.per_byte",
        "starcoin_framework.u256.from_bytes.base",
        "starcoin_framework.u256.from_bytes.per_byte",
        "starcoin_framework.u256.add.base",
        "starcoin_framework.u256.sub.base",
        "starcoin_framework.u256.mul.base",
        "starcoin_framework.u256.div.base",
        "starcoin_framework.u256.rem.base",
        "starcoin_framework.u256.pow.base",
        "starcoin_framework.signature.ed25519.pubkey.base",
        "starcoin_framework.signature.ed25519.pubkey.per_byte",
        "starcoin_framework.signature.ed25519.verify.base",
        "starcoin_framework.signature.ed25519.verify.per_byte",
        "starcoin_framework.account.create_address.base",
        "starcoin_framework.account.create_signer.base",
        "starcoin_framework.algebra.ark_bn254_fq12_add",
        "starcoin_framework.algebra.ark_bn254_fq12_clone",
        "starcoin_framework.algebra.ark_bn254_fq12_deser",
        "starcoin_framework.algebra.ark_bn254_fq12_div",
        "starcoin_framework.algebra.ark_bn254_fq12_eq",
        "starcoin_framework.algebra.ark_bn254_fq12_from_u64",
        "starcoin_framework.algebra.ark_bn254_fq12_inv",
        "starcoin_framework.algebra.ark_bn254_fq12_mul",
        "starcoin_framework.algebra.ark_bn254_fq12_neg",
        "starcoin_framework.algebra.ark_bn254_fq12_one",
        "starcoin_framework.algebra.ark_bn254_fq12_pow_u256",
        "starcoin_framework.algebra.ark_bn254_fq12_serialize",
        "starcoin_framework.algebra.ark_bn254_fq12_square",
        "starcoin_framework.algebra.ark_bn254_fq12_sub",
        "starcoin_framework.algebra.ark_bn254_fq12_zero",
        "starcoin_framework.algebra.ark_bn254_fq_add",
        "starcoin_framework.algebra.ark_bn254_fq_clone",
        "starcoin_framework.algebra.ark_bn254_fq_deser",
        "starcoin_framework.algebra.ark_bn254_fq_div",
        "starcoin_framework.algebra.ark_bn254_fq_eq",
        "starcoin_framework.algebra.ark_bn254_fq_from_u64",
        "starcoin_framework.algebra.ark_bn254_fq_inv",
        "starcoin_framework.algebra.ark_bn254_fq_mul",
        "starcoin_framework.algebra.ark_bn254_fq_neg",
        "starcoin_framework.algebra.ark_bn254_fq_one",
        "starcoin_framework.algebra.ark_bn254_fq_pow_u256",
        "starcoin_framework.algebra.ark_bn254_fq_serialize",
        "starcoin_framework.algebra.ark_bn254_fq_square",
        "starcoin_framework.algebra.ark_bn254_fq_sub",
        "starcoin_framework.algebra.ark_bn254_fq_zero",
        "starcoin_framework.algebra.ark_bn254_fr_add",
        "starcoin_framework.algebra.ark_bn254_fr_deser",
        "starcoin_framework.algebra.ark_bn254_fr_div",
        "starcoin_framework.algebra.ark_bn254_fr_eq",
        "starcoin_framework.algebra.ark_bn254_fr_from_u64",
        "starcoin_framework.algebra.ark_bn254_fr_inv",
        "starcoin_framework.algebra.ark_bn254_fr_mul",
        "starcoin_framework.algebra.ark_bn254_fr_neg",
        "starcoin_framework.algebra.ark_bn254_fr_one",
        "starcoin_framework.algebra.ark_bn254_fr_serialize",
        "starcoin_framework.algebra.ark_bn254_fr_square",
        "starcoin_framework.algebra.ark_bn254_fr_sub",
        "starcoin_framework.algebra.ark_bn254_fr_zero",
        "starcoin_framework.algebra.ark_bn254_g1_affine_deser_comp",
        "starcoin_framework.algebra.ark_bn254_g1_affine_deser_uncomp",
        "starcoin_framework.algebra.ark_bn254_g1_affine_serialize_comp",
        "starcoin_framework.algebra.ark_bn254_g1_affine_serialize_uncomp",
        "starcoin_framework.algebra.ark_bn254_g1_proj_add",
        "starcoin_framework.algebra.ark_bn254_g1_proj_double",
        "starcoin_framework.algebra.ark_bn254_g1_proj_eq",
        "starcoin_framework.algebra.ark_bn254_g1_proj_generator",
        "starcoin_framework.algebra.ark_bn254_g1_proj_infinity",
        "starcoin_framework.algebra.ark_bn254_g1_proj_neg",
        "starcoin_framework.algebra.ark_bn254_g1_proj_scalar_mul",
        "starcoin_framework.algebra.ark_bn254_g1_proj_sub",
        "starcoin_framework.algebra.ark_bn254_g1_proj_to_affine",
        "starcoin_framework.algebra.ark_bn254_g2_affine_deser_comp",
        "starcoin_framework.algebra.ark_bn254_g2_affine_deser_uncomp",
        "starcoin_framework.algebra.ark_bn254_g2_affine_serialize_comp",
        "starcoin_framework.algebra.ark_bn254_g2_affine_serialize_uncomp",
        "starcoin_framework.algebra.ark_bn254_g2_proj_add",
        "starcoin_framework.algebra.ark_bn254_g2_proj_double",
        "starcoin_framework.algebra.ark_bn254_g2_proj_eq",
        "starcoin_framework.algebra.ark_bn254_g2_proj_generator",
        "starcoin_framework.algebra.ark_bn254_g2_proj_infinity",
        "starcoin_framework.algebra.ark_bn254_g2_proj_neg",
        "starcoin_framework.algebra.ark_bn254_g2_proj_scalar_mul",
        "starcoin_framework.algebra.ark_bn254_g2_proj_sub",
        "starcoin_framework.algebra.ark_bn254_g2_proj_to_affine",
        "starcoin_framework.algebra.ark_bn254_multi_pairing_base",
        "starcoin_framework.algebra.ark_bn254_multi_pairing_per_pair",
        "starcoin_framework.algebra.ark_bn254_pairing",
        "starcoin_framework.algebra.ark_bls12_381_fq12_add",
        "starcoin_framework.algebra.ark_bls12_381_fq12_clone",
        "starcoin_framework.algebra.ark_bls12_381_fq12_deser",
        "starcoin_framework.algebra.ark_bls12_381_fq12_div",
        "starcoin_framework.algebra.ark_bls12_381_fq12_eq",
        "starcoin_framework.algebra.ark_bls12_381_fq12_from_u64",
        "starcoin_framework.algebra.ark_bls12_381_fq12_inv",
        "starcoin_framework.algebra.ark_bls12_381_fq12_mul",
        "starcoin_framework.algebra.ark_bls12_381_fq12_neg",
        "starcoin_framework.algebra.ark_bls12_381_fq12_one",
        "starcoin_framework.algebra.ark_bls12_381_fq12_pow_u256",
        "starcoin_framework.algebra.ark_bls12_381_fq12_serialize",
        "starcoin_framework.algebra.ark_bls12_381_fq12_square",
        "starcoin_framework.algebra.ark_bls12_381_fq12_sub",
        "starcoin_framework.algebra.ark_bls12_381_fq12_zero",
        "starcoin_framework.algebra.ark_bls12_381_fr_add",
        "starcoin_framework.algebra.ark_bls12_381_fr_deser",
        "starcoin_framework.algebra.ark_bls12_381_fr_div",
        "starcoin_framework.algebra.ark_bls12_381_fr_eq",
        "starcoin_framework.algebra.ark_bls12_381_fr_from_u64",
        "starcoin_framework.algebra.ark_bls12_381_fr_inv",
        "starcoin_framework.algebra.ark_bls12_381_fr_mul",
        "starcoin_framework.algebra.ark_bls12_381_fr_neg",
        "starcoin_framework.algebra.ark_bls12_381_fr_one",
        "starcoin_framework.algebra.ark_bls12_381_fr_serialize",
        "starcoin_framework.algebra.ark_bls12_381_fr_square",
        "starcoin_framework.algebra.ark_bls12_381_fr_sub",
        "starcoin_framework.algebra.ark_bls12_381_fr_zero",
        "starcoin_framework.algebra.ark_bls12_381_g1_affine_deser_comp",
        "starcoin_framework.algebra.ark_bls12_381_g1_affine_deser_uncomp",
        "starcoin_framework.algebra.ark_bls12_381_g1_affine_serialize_comp",
        "starcoin_framework.algebra.ark_bls12_381_g1_affine_serialize_uncomp",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_add",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_double",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_eq",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_generator",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_infinity",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_neg",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_scalar_mul",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_sub",
        "starcoin_framework.algebra.ark_bls12_381_g1_proj_to_affine",
        "starcoin_framework.algebra.ark_bls12_381_g2_affine_deser_comp",
        "starcoin_framework.algebra.ark_bls12_381_g2_affine_deser_uncomp",
        "starcoin_framework.algebra.ark_bls12_381_g2_affine_serialize_comp",
        "starcoin_framework.algebra.ark_bls12_381_g2_affine_serialize_uncomp",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_add",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_double",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_eq",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_generator",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_infinity",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_neg",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_scalar_mul",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_sub",
        "starcoin_framework.algebra.ark_bls12_381_g2_proj_to_affine",
        "starcoin_framework.algebra.ark_bls12_381_multi_pairing_base",
        "starcoin_framework.algebra.ark_bls12_381_multi_pairing_per_pair",
        "starcoin_framework.algebra.ark_bls12_381_pairing",
        "starcoin_framework.algebra.ark_h2c_bls12381g1_xmd_sha256_sswu_base",
        "starcoin_framework.algebra.ark_h2c_bls12381g1_xmd_sha256_sswu_per_msg_byte",
        "starcoin_framework.algebra.ark_h2c_bls12381g2_xmd_sha256_sswu_base",
        "starcoin_framework.algebra.ark_h2c_bls12381g2_xmd_sha256_sswu_per_msg_byte",
        "starcoin_framework.bls12381.base",
        "starcoin_framework.bls12381.per_pubkey_deserialize",
        "starcoin_framework.bls12381.per_pubkey_aggregate",
        "starcoin_framework.bls12381.per_pubkey_subgroup_check",
        "starcoin_framework.bls12381.per_sig_deserialize",
        "starcoin_framework.bls12381.per_sig_aggregate",
        "starcoin_framework.bls12381.per_sig_subgroup_check",
        "starcoin_framework.bls12381.per_sig_verify",
        "starcoin_framework.bls12381.per_pop_verify",
        "starcoin_framework.bls12381.per_pairing",
        "starcoin_framework.bls12381.per_msg_hashing",
        "starcoin_framework.bls12381.per_byte_hashing",
        "starcoin_framework.signature.base",
        "starcoin_framework.signature.per_pubkey_deserialize",
        "starcoin_framework.signature.per_pubkey_small_order_check",
        "starcoin_framework.signature.per_sig_deserialize",
        "starcoin_framework.signature.per_sig_strict_verify",
        "starcoin_framework.signature.per_msg_hashing_base",
        "starcoin_framework.signature.per_msg_byte_hashing",
        "starcoin_framework.secp256k1.base",
        "starcoin_framework.secp256k1.ecdsa_recover",
        "starcoin_framework.ristretto255.basepoint_mul",
        "starcoin_framework.ristretto255.basepoint_double_mul",
        "starcoin_framework.ristretto255.point_add",
        "starcoin_framework.ristretto255.point_clone",
        "starcoin_framework.ristretto255.point_compress",
        "starcoin_framework.ristretto255.point_decompress",
        "starcoin_framework.ristretto255.point_equals",
        "starcoin_framework.ristretto255.point_from_64_uniform_bytes",
        "starcoin_framework.ristretto255.point_identity",
        "starcoin_framework.ristretto255.point_mul",
        "starcoin_framework.ristretto255.point_double_mul",
        "starcoin_framework.ristretto255.point_neg",
        "starcoin_framework.ristretto255.point_sub",
        "starcoin_framework.ristretto255.point_parse_arg",
        "starcoin_framework.ristretto255.scalar_sha512_per_byte",
        "starcoin_framework.ristretto255.scalar_sha512_per_hash",
        "starcoin_framework.ristretto255.scalar_add",
        "starcoin_framework.ristretto255.scalar_reduced_from_32_bytes",
        "starcoin_framework.ristretto255.scalar_uniform_from_64_bytes",
        "starcoin_framework.ristretto255.scalar_from_u128",
        "starcoin_framework.ristretto255.scalar_from_u64",
        "starcoin_framework.ristretto255.scalar_invert",
        "starcoin_framework.ristretto255.scalar_is_canonical",
        "starcoin_framework.ristretto255.scalar_mul",
        "starcoin_framework.ristretto255.scalar_neg",
        "starcoin_framework.ristretto255.scalar_sub",
        "starcoin_framework.ristretto255.scalar_parse_arg",
        "starcoin_framework.hash.sip_hash.base",
        "starcoin_framework.hash.sip_hash.per_byte",
        "starcoin_framework.hash.keccak256.base",
        "starcoin_framework.hash.keccak256.per_byte",
        "starcoin_framework.bulletproofs.base",
        "starcoin_framework.bulletproofs.per_bit_rangeproof_verify",
        "starcoin_framework.bulletproofs.per_byte_rangeproof_deserialize",
        "starcoin_framework.type_info.type_of.base",
        "starcoin_framework.type_info.type_of.per_abstract_memory_unit",
        "starcoin_framework.type_info.type_name.base",
        "starcoin_framework.type_info.type_name.per_abstract_memory_unit",
        "starcoin_framework.type_info.chain_id.base",
        "starcoin_framework.function_info.is_identifier.base",
        "starcoin_framework.function_info.is_identifier.per_byte",
        "starcoin_framework.function_info.check_dispatch_type_compatibility_impl.base",
        "starcoin_framework.function_info.load_function.base",
        "starcoin_framework.dispatchable_fungible_asset.dispatch.base",
        "starcoin_framework.hash.sha2_512.base",
        "starcoin_framework.hash.sha2_512.per_byte",
        "starcoin_framework.hash.sha3_512.base",
        "starcoin_framework.hash.sha3_512.per_byte",
        "starcoin_framework.hash.ripemd160.base",
        "starcoin_framework.hash.ripemd160.per_byte",
        "starcoin_framework.hash.blake2b_256.base",
        "starcoin_framework.hash.blake2b_256.per_byte",
        "starcoin_framework.util.from_bytes.base",
        "starcoin_framework.util.from_bytes.per_byte",
        "starcoin_framework.transaction_context.get_txn_hash.base",
        "starcoin_framework.transaction_context.get_script_hash.base",
        "starcoin_framework.transaction_context.generate_unique_address.base",
        "starcoin_framework.transaction_context.sender.base",
        "starcoin_framework.transaction_context.secondary_signers.base",
        "starcoin_framework.transaction_context.secondary_signers.per_signer",
        "starcoin_framework.transaction_context.fee_payer.base",
        "starcoin_framework.transaction_context.max_gas_amount.base",
        "starcoin_framework.transaction_context.gas_unit_price.base",
        "starcoin_framework.transaction_context.chain_id.base",
        "starcoin_framework.transaction_context.entry_function_payload.base",
        "starcoin_framework.transaction_context.entry_function_payload.per_abstract_memory_unit",
        "starcoin_framework.transaction_context.multisig_payload.base",
        "starcoin_framework.transaction_context.multisig_payload.per_abstract_memory_unit",
        "starcoin_framework.code.request_publish.base",
        "starcoin_framework.code.request_publish.per_byte",
        "starcoin_framework.event.write_to_event_store.base",
        "starcoin_framework.event.write_to_event_store.per_abstract_memory_unit",
        "starcoin_framework.state_storage.get_usage.base",
        "starcoin_framework.aggregator.add.base",
        "starcoin_framework.aggregator.read.base",
        "starcoin_framework.aggregator.sub.base",
        "starcoin_framework.aggregator.destroy.base",
        "starcoin_framework.aggregator_factory.new_aggregator.base",
        "starcoin_framework.aggregator_v2.create_aggregator.base",
        "starcoin_framework.aggregator_v2.try_add.base",
        "starcoin_framework.aggregator_v2.try_sub.base",
        "starcoin_framework.aggregator_v2.is_at_least.base",
        "starcoin_framework.aggregator_v2.read.base",
        "starcoin_framework.aggregator_v2.snapshot.base",
        "starcoin_framework.aggregator_v2.create_snapshot.base",
        "starcoin_framework.aggregator_v2.create_snapshot.per_byte",
        "starcoin_framework.aggregator_v2.copy_snapshot.base",
        "starcoin_framework.aggregator_v2.read_snapshot.base",
        "starcoin_framework.aggregator_v2.string_concat.base",
        "starcoin_framework.aggregator_v2.string_concat.per_byte",
        "starcoin_framework.object.exists_at.base",
        "starcoin_framework.object.user_derived_address.base",
        "starcoin_framework.object.exists_at.per_byte_loaded",
        "starcoin_framework.object.exists_at.per_item_loaded",
        "starcoin_framework.string_utils.format.base",
        "starcoin_framework.string_utils.format.per_byte",
    ]
});

fn append_extra_gas_cost_framework_upgrade(entries: &mut Vec<(String, u64)>) {
    for (_idx, cost) in G_MOVE_FRAMEWORK_UPGRADE_STRS.iter().enumerate() {
        entries.push((cost.to_string(), 1));
    }
}
