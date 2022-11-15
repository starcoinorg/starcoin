use crate::on_chain_config::{OnChainConfig, VMConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct GasSchedule {
    pub entries: Vec<(String, u64)>,
}

impl GasSchedule {
    pub fn to_btree_map(self) -> BTreeMap<String, u64> {
        // TODO: what if the gas schedule contains duplicated entries?
        self.entries.into_iter().collect()
    }
}

// instruction_table_v1
pub fn instruction_gas_schedule_v1() -> BTreeMap<String, u64> {
    BTreeMap::from([
        ("instr.move_to.base".to_string(), 13),
        ("instr.move_to_generic.base".to_string(), 27),
        ("instr.move_from.base".to_string(), 459),
        ("instr.move_from_generic.base".to_string(), 13),
        ("instr.br_true".to_string(), 1),
        ("instr.write_ref.base".to_string(), 1),
        ("instr.mul".to_string(), 1),
        ("instr.move_loc.base".to_string(), 1),
        ("instr.and".to_string(), 1),
        ("instr.pop".to_string(), 1),
        ("instr.bit_and".to_string(), 2),
        ("instr.read_ref.base".to_string(), 1),
        ("instr.sub".to_string(), 1),
        ("instr.mut_borrow_field".to_string(), 1),
        ("instr.mut_borrow_field_generic.base".to_string(), 1),
        ("instr.imm_borrow_field".to_string(), 1),
        ("instr.imm_borrow_field_generic.base".to_string(), 1),
        ("instr.add".to_string(), 1),
        ("instr.copy_loc.base".to_string(), 1),
        ("instr.st_loc.base".to_string(), 1),
        ("instr.ret".to_string(), 638),
        ("instr.lt".to_string(), 1),
        ("instr.ld_u8".to_string(), 1),
        ("instr.ld_u64".to_string(), 1),
        ("instr.ld_u128".to_string(), 1),
        ("instr.cast_u8".to_string(), 2),
        ("instr.cast_u64".to_string(), 1),
        ("instr.cast_u128".to_string(), 1),
        ("instr.abort".to_string(), 1),
        ("instr.mut_borrow_loc".to_string(), 2),
        ("instr.imm_borrow_loc".to_string(), 1),
        ("instr.ld_const.base".to_string(), 1),
        ("instr.ge".to_string(), 1),
        ("instr.xor".to_string(), 1),
        ("instr.shl".to_string(), 2),
        ("instr.shr".to_string(), 1),
        ("instr.neq.base".to_string(), 1),
        ("instr.not".to_string(), 1),
        ("instr.call.base".to_string(), 1132),
        ("instr.call_generic.base".to_string(), 582),
        ("instr.le".to_string(), 2),
        ("instr.branch".to_string(), 1),
        ("instr.unpack.base".to_string(), 2),
        ("instr.unpack_generic.base".to_string(), 2),
        ("instr.or".to_string(), 2),
        ("instr.ld_false".to_string(), 1),
        ("instr.ld_true".to_string(), 1),
        ("instr.mod".to_string(), 1),
        ("instr.br_false".to_string(), 1),
        ("instr.exists.base".to_string(), 41),
        ("instr.exists_generic.base".to_string(), 34),
        ("instr.bit_or".to_string(), 2),
        ("instr.freeze_ref".to_string(), 1),
        ("instr.mut_borrow_global.base".to_string(), 21),
        ("instr.mut_borrow_global_generic.base".to_string(), 15),
        ("instr.imm_borrow_global.base".to_string(), 23),
        ("instr.imm_borrow_global_generic.base".to_string(), 14),
        ("instr.div".to_string(), 3),
        ("instr.eq.base".to_string(), 1),
        ("instr.gt".to_string(), 1),
        ("instr.pack.base".to_string(), 2),
        ("instr.pack_generic.base".to_string(), 2),
        ("instr.nop".to_string(), 1),
    ])
}

// instruction_table_v2
pub fn instruction_gas_schedule_v2() -> BTreeMap<String, u64> {
    let mut instrs = instruction_gas_schedule_v1();
    let mut instrs_delta = BTreeMap::from([
        ("instr.vec_pack.base".to_string(), 84),
        ("instr.vec_len.base".to_string(), 98),
        ("instr.vec_imm_borrow.base".to_string(), 1334),
        ("instr.vec_mut_borrow.base".to_string(), 1902),
        ("instr.vec_push_back.base".to_string(), 53),
        ("instr.vec_pop_back.base".to_string(), 227),
        ("instr.vec_unpack.base".to_string(), 527),
        ("instr.vec_swap.base".to_string(), 1436),
    ]);
    instrs.append(&mut instrs_delta);
    instrs
}

// native_table_v1
pub fn native_gas_schedule_v1() -> BTreeMap<String, u64> {
    BTreeMap::from([
        ("move_stdlib.hash.sha2_256.base".to_string(), 21),
        ("move_stdlib.hash.sha3_256.base".to_string(), 64),
        (
            "starcoin_natives.signature.ed25519_verify.base".to_string(),
            61,
        ),
        // ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated
        (
            "move_stdlib.bcs.to_bytes.per_byte_serialized".to_string(),
            181,
        ),
        ("move_stdlib.vector.length.base".to_string(), 98),
        ("move_stdlib.vector.empty.base".to_string(), 84),
        ("move_stdlib.vector.borrow.base".to_string(), 1334),
        ("move_stdlib.vector.push_back.base".to_string(), 53),
        ("move_stdlib.vector.pop_back.base".to_string(), 227),
        ("move_stdlib.vector.destroy_empty.base".to_string(), 572),
        ("move_stdlib.vector.swap.base".to_string(), 1436),
        (
            "starcoin_natives.signature.ed25519_validate_key.base".to_string(),
            26,
        ),
        ("move_stdlib.signer.borrow_address.base".to_string(), 353),
        (
            "starcoin_natives.account.create_signer.base".to_string(),
            24,
        ),
        (
            "starcoin_natives.account.destroy_signer.base".to_string(),
            212,
        ),
        (
            "nursery.event.write_to_event_store.unit_cost".to_string(),
            52,
        ),
        ("move_stdlib.bcs.to_address.base".to_string(), 26),
        ("starcoin_natives.token.name_of.base".to_string(), 2002),
    ])
}

// native_table_v2
pub fn native_gas_schedule_v2() -> BTreeMap<String, u64> {
    let mut natives = native_gas_schedule_v1();
    let mut natives_delta =
        BTreeMap::from([("starcoin_natives.hash.keccak256.base".to_string(), 64)]);
    natives.append(&mut natives_delta);
    natives
}

// v3_native_table
pub fn native_gas_schedule_v3() -> BTreeMap<String, u64> {
    let mut natives = native_gas_schedule_v2();
    let mut natives_delta = BTreeMap::from([
        ("starcoin_natives.hash.ripemd160.base".to_string(), 64),
        (
            "starcoin_natives.signature.ec_recover.base".to_string(),
            128,
        ),
        ("starcoin_natives.u256.from_bytes.base".to_string(), 2),
        ("starcoin_natives.u256.add.base".to_string(), 4),
        ("starcoin_natives.u256.sub.base".to_string(), 4),
        ("starcoin_natives.u256.mul.base".to_string(), 4),
        ("starcoin_natives.u256.div.base".to_string(), 10),
        ("starcoin_natives.u256.rem.base".to_string(), 4),
        ("starcoin_natives.u256.pow.base".to_string(), 8),
        ("move_stdlib.vector.append.base".to_string(), 40),
        ("move_stdlib.vector.remove.base".to_string(), 20),
        ("move_stdlib.vector.reverse.base".to_string(), 10),
    ]);
    natives.append(&mut natives_delta);
    natives
}

// v4_native_table
pub fn native_gas_schedule_v4() -> BTreeMap<String, u64> {
    let mut natives = native_gas_schedule_v3();
    let mut natives_delta = BTreeMap::from([
        ("table.new_table_handle.base".to_string(), 4),
        ("table.add_box.base".to_string(), 4),
        ("table.borrow_box.base".to_string(), 4),
        ("table.remove_box.base".to_string(), 4),
        ("table.contains_box.base".to_string(), 4),
        ("table.destroy_empty_box.base".to_string(), 4),
        ("table.drop_unchecked_box.base".to_string(), 4),
        ("move_stdlib.string.check_utf8.base".to_string(), 4),
        ("move_stdlib.string.sub_string.base".to_string(), 4),
        ("move_stdlib.string.is_char_boundary.base".to_string(), 4),
        ("move_stdlib.string.index_of.base".to_string(), 4),
    ]);
    natives.append(&mut natives_delta);
    natives
}

static G_MAX_TRANSACTION_SIZE_IN_BYTES_V1: u64 = 4096 * 10;
static G_MAX_TRANSACTION_SIZE_IN_BYTES_V2: u64 = 60000;
static G_MAX_TRANSACTION_SIZE_IN_BYTES_V3: u64 = 128 * 1024;

pub fn txn_gas_schedule_v1() -> BTreeMap<String, u64> {
    BTreeMap::from([
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
    ])
}

pub fn txn_gas_schedule_v2() -> BTreeMap<String, u64> {
    BTreeMap::from([
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
    ])
}

pub fn txn_gas_schedule_v3() -> BTreeMap<String, u64> {
    BTreeMap::from([
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
    ])
}

pub fn txn_gas_schedule_test() -> BTreeMap<String, u64> {
    BTreeMap::from([
        ("txn.global_memory_per_byte_cost".to_string(), 4),
        ("txn.global_memory_per_byte_write_cost".to_string(), 9),
        ("txn.min_transaction_gas_units".to_string(), 600),
        ("txn.large_transaction_cutoff".to_string(), 600),
        ("txn.intrinsic_gas_per_byte".to_string(), 8),
        (
            "txn.maximum_number_of_gas_units".to_string(),
            40_000_000 * 10,
        ),
        ("txn.min_price_per_gas_unit".to_string(), 1),
        ("txn.max_price_per_gas_unit".to_string(), 10_000),
        (
            "txn.max_transaction_size_in_bytes".to_string(),
            G_MAX_TRANSACTION_SIZE_IN_BYTES_V3,
        ),
        ("txn.gas_unit_scaling_factor".to_string(), 1),
        ("txn.default_account_size".to_string(), 800),
    ])
}

// XXX FIXME YSG, check wether we need add gas_schedule in storage
impl OnChainConfig for GasSchedule {
    const MODULE_IDENTIFIER: &'static str = "gas_schedule";
    const CONF_IDENTIFIER: &'static str = "GasScheduleConfig";
}

// XXX FIXME YSG, have some bug, should confirm
// https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move
impl From<VMConfig> for GasSchedule {
    fn from(vm_config: VMConfig) -> Self {
        let mut entries = vec![];

        // see vm/gas_algebra-ext/src/instr.rs
        // see https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#instruction_schedule
        let instrs = vm_config.gas_schedule.instruction_table.clone();
        let instr_strs = vec![
            "instr.pop",
            "instr.ret",
            "instr.br_true",
            "instr.br_false",
            "instr.branch",
            "instr.ld_u64",
            "instr.ld_const.base",
            "instr.ld_true",
            "instr.ld_false",
            "instr.copy_loc.base",
            "instr.move_loc.base",
            "instr.st_loc.base",
            "instr.mut_borrow_loc",
            "instr.imm_borrow_loc",
            "instr.mut_borrow_field",
            "instr.imm_borrow_field",
            "instr.call.base",
            "instr.pack.base",
            "instr.unpack.base",
            "instr.read_ref.base",
            "instr.write_ref.base",
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
            "instr.eq.base",
            "instr.neq.base",
            "instr.lt",
            "instr.gt",
            "instr.le",
            "instr.ge",
            "instr.abort",
            "instr.nop",
            "instr.exists.base",
            "instr.mut_borrow_global.base",
            "instr.imm_borrow_global.base",
            "instr.move_from.base",
            "instr.move_to.base",
            "instr.freeze_ref",
            "instr.shl",
            "instr.shr",
            "instr.ld_u8",
            "instr.ld_u128",
            "instr.cast_u8",
            "instr.cast_u64",
            "instr.cast_u128",
            "instr.imm_borrow_field_generic.base",
            "instr.mut_borrow_field_generic.base",
            "instr.call_generic.base",
            "instr.pack_generic.base",
            "instr.unpack_generic.base",
            "instr.exists_generic.base",
            "instr.mut_borrow_global_generic.base",
            "instr.imm_borrow_global_generic.base",
            "instr.move_from_generic.base",
            "instr.move_to_generic.base",
            "instr.vec_pack.base",
            "instr.vec_len.base",
            "instr.vec_imm_borrow.base",
            "instr.vec_mut_borrow.base",
            "instr.vec_push_back.base",
            "instr.vec_pop_back.base",
            "instr.vec_unpack.base",
            "instr.vec_swap.base",
        ];
        for (idx, cost) in instrs.into_iter().enumerate() {
            entries.push((instr_strs[idx].to_string(), cost.instruction_gas));
        }

        // see vm/gas_algebra-ext/src/{move_stdlib.rs starcoin_framework.rs nursery.rs table.rs}
        // see https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#native_schedule
        let natives = vm_config.gas_schedule.native_table.clone();
        let native_strs = vec![
            "move_stdlib.hash.sha2_256.base",
            "move_stdlib.hash.sha3_256.base",
            "starcoin_natives.signature.ed25519_verify.base",
            // ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated, ignore, use ""
            "",
            "move_stdlib.bcs.to_bytes.per_byte_serialized",
            "move_stdlib.vector.length.base",
            "move_stdlib.vector.empty.base",
            "move_stdlib.vector.borrow.base",
            // Vector::borrow_mut is same Vector::borrow ignore ""
            "",
            "move_stdlib.vector.push_back.base",
            "move_stdlib.vector.pop_back.base",
            "move_stdlib.vector.destroy_empty.base",
            "move_stdlib.vector.swap.base",
            "starcoin_natives.signature.ed25519_validate_key.base",
            "move_stdlib.signer.borrow_address.base",
            "starcoin_natives.account.create_signer.base",
            "starcoin_natives.account.destroy_signer.base",
            "nursery.event.write_to_event_store.unit_cost",
            "move_stdlib.bcs.to_address.base",
            "starcoin_natives.token.name_of.base",
            "starcoin_natives.hash.keccak256.base",
            "starcoin_natives.hash.ripemd160.base",
            "starcoin_natives.signature.ec_recover.base",
            "starcoin_natives.u256.from_bytes.base",
            "starcoin_natives.u256.add.base",
            "starcoin_natives.u256.sub.base",
            "starcoin_natives.u256.mul.base",
            "starcoin_natives.u256.div.base",
            "starcoin_natives.u256.rem.base",
            "starcoin_natives.u256.pow.base",
            "move_stdlib.vector.append.base",
            "move_stdlib.vector.remove.base",
            "move_stdlib.vector.reverse.base",
            "table.new_table_handle.base",
            "table.add_box.base",
            "table.borrow_box.base",
            "table.remove_box.base",
            "table.contains_box.base",
            "table.destroy_empty_box.base",
            "table.drop_unchecked_box.base",
            "move_stdlib.string.check_utf8.base",
            "move_stdlib.string.sub_string.base",
            "move_stdlib.string.is_char_boundary.base",
            "move_stdlib.string.index_of.base",
        ];
        for (idx, cost) in natives.into_iter().enumerate() {
            if native_strs[idx].is_empty() {
                continue;
            }
            entries.push((native_strs[idx].to_string(), cost.instruction_gas));
        }

        // see vm/gas_algebra-ext/src/transaction.rs
        let txn = vm_config.gas_schedule.gas_constants;
        entries.push((
            "txn.global_memory_per_byte_cost".to_string(),
            u64::from(txn.global_memory_per_byte_cost),
        ));
        entries.push((
            "txn.global_memory_per_byte_write_cost".to_string(),
            u64::from(txn.global_memory_per_byte_write_cost),
        ));
        entries.push((
            "txn.min_transaction_gas_units".to_string(),
            u64::from(txn.min_transaction_gas_units),
        ));
        entries.push((
            "txn.large_transaction_cutoff".to_string(),
            u64::from(txn.large_transaction_cutoff),
        ));
        entries.push((
            "txn.intrinsic_gas_per_byte".to_string(),
            u64::from(txn.intrinsic_gas_per_byte),
        ));
        entries.push((
            "txn.maximum_number_of_gas_units".to_string(),
            u64::from(txn.maximum_number_of_gas_units),
        ));
        entries.push((
            "txn.min_price_per_gas_unit".to_string(),
            u64::from(txn.min_price_per_gas_unit),
        ));
        entries.push((
            "txn.max_price_per_gas_unit".to_string(),
            u64::from(txn.max_price_per_gas_unit),
        ));
        entries.push((
            "txn.max_transaction_size_in_bytes".to_string(),
            u64::from(txn.max_transaction_size_in_bytes),
        ));
        entries.push((
            "txn.gas_unit_scaling_factor".to_string(),
            u64::from(txn.gas_unit_scaling_factor),
        ));
        entries.push((
            "txn.default_account_size".to_string(),
            u64::from(txn.default_account_size),
        ));

        Self { entries }
    }
}
