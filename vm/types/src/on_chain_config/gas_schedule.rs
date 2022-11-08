use crate::on_chain_config::{OnChainConfig, VMConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
        (String::from("move_to.base"), 13),
        (String::from("move_to_generic.base"), 27),
        (String::from("move_from.base"), 459),
        (String::from("move_from_generic.base"), 13),
        (String::from("br_true"), 1),
        (String::from("write_ref.base"), 1),
        (String::from("mul"), 1),
        (String::from("move_loc.base"), 1),
        (String::from("and"), 1),
        (String::from("pop"), 1),
        (String::from("bit_and"), 2),
        (String::from("read_ref.base"), 1),
        (String::from("sub"), 1),
        (String::from("mut_borrow_field"), 1),
        (String::from("mut_borrow_field_generic"), 1),
        (String::from("imm_borrow_field"), 1),
        (String::from("imm_borrow_field_generic"), 1),
        (String::from("add"), 1),
        (String::from("copy_loc.base"), 1),
        (String::from("st_loc.base"), 1),
        (String::from("ret"), 638),
        (String::from("lt"), 1),
        (String::from("ld_u8"), 1),
        (String::from("ld_u64"), 1),
        (String::from("ld_u128"), 1),
        (String::from("cast_u8"), 2),
        (String::from("cast_u64"), 1),
        (String::from("cast_u128"), 1),
        (String::from("abort"), 1),
        (String::from("mut_borrow_loc"), 2),
        (String::from("imm_borrow_loc"), 1),
        (String::from("ld_const.base"), 1),
        (String::from("ge"), 1),
        (String::from("xor"), 1),
        (String::from("shl"), 2),
        (String::from("shr"), 1),
        (String::from("neq"), 1),
        (String::from("not"), 1),
        (String::from("call.base"), 1132),
        (String::from("call_generic.base"), 582),
        (String::from("le"), 2),
        (String::from("branch"), 1),
        (String::from("unpack.base"), 2),
        (String::from("unpack_generic.base"), 2),
        (String::from("or"), 2),
        (String::from("ld_false"), 1),
        (String::from("ld_true"), 1),
        (String::from("mod"), 1),
        (String::from("br_false"), 1),
        (String::from("exists.base"), 41),
        (String::from("exists_generic.base"), 34),
        (String::from("bit_or"), 2),
        (String::from("freeze_ref"), 1),
        (String::from("mut_borrow_global.base"), 21),
        (String::from("mut_borrow_global_generic.base"), 15),
        (String::from("imm_borrow_global.base"), 23),
        (String::from("ImmBorrowGlobalGeneric"), 14),
        (String::from("div"), 3),
        (String::from("eq"), 1),
        (String::from("gt"), 1),
        (String::from("pack"), 2),
        (String::from("pack_generic"), 2),
        (String::from("nop"), 1),
    ])
}

// instruction_table_v2
pub fn instruction_gas_schedule_v2() -> BTreeMap<String, u64> {
    let mut instrs = instruction_gas_schedule_v1();
    let mut instrs_delta = BTreeMap::from([
        (String::from("vec_pack.base"), 84),
        (String::from("vec_len.base"), 98),
        (String::from("vec_imm_borrow.base"), 1334),
        (String::from("vec_mut_borrow.base"), 1902),
        (String::from("vec_push_back.base"), 53),
        (String::from("vec_pop_back.base"), 227),
        (String::from("vec_unpack.base"), 527),
        (String::from("vec_swap.base"), 1436),
    ]);
    instrs.append(&mut instrs_delta);
    instrs
}

// XXX FIXME YSG check
pub fn move_stdlib_native_gas_schedule_v1() -> BTreeMap<String, u64> {
    BTreeMap::from([
        (String::from(".hash.sha2_256.base"), 21),
        (String::from(".hash.sha3_256.base"), 64),
        //  (String::from(".signature.ed25519_verify.base"), 61),
        // ED25519_THRESHOLD_VERIFY 3 this native funciton is deprecated
        (String::from(".bcs.to_bytes.per_byte_serialized"), 181),
        (String::from(".vec.length.base"), 98),
        (String::from(".vec.borrow.base"), 1334),
        //   (String::from(".signature.ed25519_validate_publickey.base"),26),
        (String::from(".vec.push_back.base"), 53),
        (String::from(".vec.pop_back.base"), 227),
        (String::from(".vec.destroy_empty.base"), 572),
        (String::from(".vec.swap.base"), 1436),
        // ED25519_VALIDATE_KEY
        (String::from(".signer.borrow_address.base"), 353),
        // (String::from(".account.create_signer.base"), 24),
        // (String::from(".account.destroy_signer.base"), 212),
        (String::from(".event.write_to_event_store.unit_cost"), 52),
        (String::from(".bcs.to_address.base"), 26),
        (String::from(".token.address.base"), 2002),
    ])
}

pub fn move_stdlib_native_gas_schedule_v2() -> BTreeMap<String, u64> {
    let mut natives = move_stdlib_native_gas_schedule_v1();
    let mut natives_delta = BTreeMap::from([(String::from(".hash.keccak256.base"), 64)]);
    natives.append(&mut natives_delta);
    natives
}

pub fn move_stdlib_native_gas_schedule_v3() -> BTreeMap<String, u64> {
    let mut natives = move_stdlib_native_gas_schedule_v2();
    let mut natives_delta = BTreeMap::from([
        (String::from(".hash.ripemd160.base"), 64),
        (String::from(".hash.ec_recover.base"), 128),
        (String::from(".u256.from_bytes.base"), 2),
        (String::from(".u256.add.base"), 4),
        (String::from(".u256.sub.base"), 4),
        (String::from(".u256.mul.base"), 4),
        (String::from(".u256.div.base"), 10),
        (String::from(".u256.rem.base"), 4),
        (String::from(".u256.pow.base"), 8),
        (String::from(".vec.append.base"), 40),
        (String::from(".vec.remove.base"), 20),
        (String::from(".vec.reverse.base"), 10),
    ]);
    natives.append(&mut natives_delta);
    natives
}

pub fn move_stdlib_native_gas_schedule_v4() -> BTreeMap<String, u64> {
    let mut natives = move_stdlib_native_gas_schedule_v3();
    let mut natives_delta = BTreeMap::from([
        (String::from(".vec.append.base"), 40),
        (String::from(".string.check_utf8.base"), 4),
        (String::from(".string.sub_str.base"), 4),
        (String::from(".string.is_char_boundary.base"), 20),
        (String::from(".string.sub_string.base"), 4),
        (String::from(".string.index_of.base"), 4),
        // XXX FIXME YSG, need table
    ]);
    natives.append(&mut natives_delta);
    natives
}

// XXX FIXME YSG, check wether we need add gas_schedule in storage
impl OnChainConfig for GasSchedule {
    const MODULE_IDENTIFIER: &'static str = "gas_schedule";
    const CONF_IDENTIFIER: &'static str = "GasScheduleConfig";
}

// XXX FIXME YSG
impl From<VMConfig> for GasSchedule {
    fn from(_vm_config: VMConfig) -> Self {
        let entries: Vec<(String, u64)> = vec![("hello".to_owned(), 1)];
        Self { entries }
    }
}
