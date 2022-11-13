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
        ("instr.mut_borrow_field_generic".to_string(), 1),
        ("instr.imm_borrow_field".to_string(), 1),
        ("instr.imm_borrow_field_generic".to_string(), 1),
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
        ("instr.neq".to_string(), 1),
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
        ("instr.eq".to_string(), 1),
        ("instr.gt".to_string(), 1),
        ("instr.pack".to_string(), 2),
        ("instr.pack_generic".to_string(), 2),
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
        ("move_stdlib.vec.length.base".to_string(), 98),
        ("move_stdlib.vec.empty.base".to_string(), 84),
        ("move_stdlib.vec.borrow.base".to_string(), 1334),
        ("move_stdlib.vec.push_back.base".to_string(), 53),
        ("move_stdlib.vec.pop_back.base".to_string(), 227),
        ("move_stdlib.vec.destroy_empty.base".to_string(), 572),
        ("move_stdlib.vec.swap.base".to_string(), 1436),
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
            "move_stdlib.event.write_to_event_store.unit_cost".to_string(),
            52,
        ),
        ("move_stdlib.bcs.to_address.base".to_string(), 26),
        (
            "starcoin_natives.token.token_name_of.base".to_string(),
            2002,
        ),
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
        ("starcoin_natives.hash.ec_recover.base".to_string(), 128),
        ("starcoin_natives.u256.from_bytes.base".to_string(), 2),
        ("starcoin_natives.u256.add.base".to_string(), 4),
        ("starcoin_natives.u256.sub.base".to_string(), 4),
        ("starcoin_natives.u256.mul.base".to_string(), 4),
        ("starcoin_natives.u256.div.base".to_string(), 10),
        ("starcoin_natives.u256.rem.base".to_string(), 4),
        ("starcoin_natives.u256.pow.base".to_string(), 8),
        ("move_stdlib.vec.append.base".to_string(), 40),
        ("move_stdlib.vec.remove.base".to_string(), 20),
        ("move_stdlib.vec.reverse.base".to_string(), 10),
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
        ("move_stdlib.string.sub_str.base".to_string(), 4),
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

// XXX FIXME YSG, see genesis_gas_schedule.rs gas-algebra-ext/{instr.rs move_stdlib.rs nursery.rs starcoin_framework.rs}
// https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move
impl From<VMConfig> for GasSchedule {
    fn from(_vm_config: VMConfig) -> Self {
        let entries: Vec<(String, u64)> = vec![("hello".to_owned(), 1)];
        Self { entries }
    }
}
