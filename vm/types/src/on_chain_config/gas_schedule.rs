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
use starcoin_gas_algebra_ext::CostTable;
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
    pub fn is_different(&self, other: &GasSchedule) -> bool {
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
    const CONF_IDENTIFIER: &'static str = GAS_SCHEDULE_MODULE_NAME;

    fn deserialize_into_config(bytes: &[u8]) -> Result<Self> {
        let raw_gas_schedule = bcs_ext::from_bytes::<GasSchedule>(bytes).map_err(|e| {
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

        // native_table don't have these
        entries.push(("nursery.debug.print.base_cost".to_string(), 1));
        entries.push(("nursery.debug.print_stack_trace.base_cost".to_string(), 1));

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

        // see vm/gas_algebra-ext/src/transaction.rs
        let txn = &cost_table.gas_constants;
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

        Self { entries }
    }
}
