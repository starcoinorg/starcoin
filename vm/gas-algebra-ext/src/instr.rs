// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines all the gas parameters and formulae for instructions, along with their
//! initial values in the genesis and a mapping between the Rust representation and the on-chain
//! gas schedule.

use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use crate::InternalGasPerAbstractValueUnit;
use move_binary_format::errors::PartialVMResult;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerArg, InternalGasPerByte};
use move_vm_types::gas::SimpleInstruction;

// see starcoin/vm/types/src/on_chain_config/genesis_gas_schedule.rs
// same order as https://github.com/starcoinorg/starcoin-framework/blob/main/sources/VMConfig.move#instruction_schedule
// modify should with impl From<VMConfig> for GasSchedule
crate::params::define_gas_parameters!(
    InstructionGasParameters,
    "instr",
    [
        [pop: InternalGas, "pop", MUL],
        [ret: InternalGas, "ret", 638 * MUL],
        [br_true: InternalGas, "br_true", MUL],
        [br_false: InternalGas, "br_false", MUL],
        [branch: InternalGas, "branch", MUL],
        [ld_u64: InternalGas, "ld_u64", MUL],
        [ld_const_base: InternalGas, "ld_const.base", MUL],
        [
            ld_const_per_byte: InternalGasPerByte,
            optional "ld_const.per_byte",
            35 * MUL
        ],
        [ld_true: InternalGas, "ld_true", MUL],
        [ld_false: InternalGas, "ld_false", MUL],
        [copy_loc_base: InternalGas, "copy_loc.base", MUL],
        [
            copy_loc_per_abs_val_unit: InternalGasPerAbstractValueUnit,
            optional "copy_loc.per_abs_val_unit",
            4 * MUL
        ],
        [move_loc_base: InternalGas, "move_loc.base", MUL],
        [st_loc_base: InternalGas, "st_loc.base", MUL],
        [mut_borrow_loc: InternalGas, "mut_borrow_loc", 2 * MUL],
        [imm_borrow_loc: InternalGas, "imm_borrow_loc", MUL],
        [mut_borrow_field: InternalGas, "mut_borrow_field", MUL],
        [imm_borrow_field: InternalGas, "imm_borrow_field", MUL],
        [call_base: InternalGas, "call.base", 1132 * MUL],
        [call_per_arg: InternalGasPerArg, optional "call.per_arg", 100 * MUL],
        [pack_base: InternalGas, "pack.base", 2 * MUL],
        [pack_per_field: InternalGasPerArg, optional "pack.per_field", 40 * MUL],
        [unpack_base: InternalGas, "unpack.base", 2 * MUL],
        [unpack_per_field: InternalGasPerArg, optional "unpack.per_field", 40 * MUL],
        [read_ref_base: InternalGas, "read_ref.base", MUL],
        [
            read_ref_per_abs_val_unit: InternalGasPerAbstractValueUnit,
            optional "read_ref.per_abs_val_unit",
            4 * MUL
        ],
        [write_ref_base: InternalGas, "write_ref.base", MUL],
        [add: InternalGas, "add", MUL],
        [sub: InternalGas, "sub", MUL],
        [mul: InternalGas, "mul", MUL],
        [mod_: InternalGas, "mod", MUL],
        [div: InternalGas, "div", 3 * MUL],
        [bit_or: InternalGas, "bit_or", 2 * MUL],
        [bit_and: InternalGas, "bit_and", 2 * MUL],
        [xor: InternalGas, "xor", MUL],
        [or: InternalGas, "or", 2 * MUL],
        [and: InternalGas, "and", MUL],
        [not: InternalGas, "not", MUL],
        [eq_base: InternalGas, "eq.base", MUL],
        [
            eq_per_abs_val_unit: InternalGasPerAbstractValueUnit,
           optional "eq.per_abs_val_unit",
            4 * MUL
        ],
        [neq_base: InternalGas, "neq.base", MUL],
        [
            neq_per_abs_val_unit: InternalGasPerAbstractValueUnit,
            optional "neq.per_abs_val_unit",
            4 * MUL
        ],
        // comparison
        [lt: InternalGas, "lt", 2 * MUL],
        [gt: InternalGas, "gt", MUL],
        [le: InternalGas, "le", 2 * MUL],
        [ge: InternalGas, "ge", MUL],
        [abort: InternalGas, "abort", MUL],
        // nop
        [nop: InternalGas, "nop", MUL],
        [exists_base: InternalGas, "exists.base", 41 * MUL],
        [
            mut_borrow_global_base: InternalGas,
            "mut_borrow_global.base",
            21 * MUL
        ],
        [
            imm_borrow_global_base: InternalGas,
            "imm_borrow_global.base",
            23 * MUL
        ],
        [move_from_base: InternalGas, "move_from.base", 459 * MUL],
        [move_to_base: InternalGas, "move_to.base", 13 * MUL],
        [freeze_ref: InternalGas, "freeze_ref", MUL],
        [shl: InternalGas, "shl", 2 * MUL],
        [shr: InternalGas, "shr", MUL],
        [ld_u8: InternalGas, "ld_u8", MUL],
        [ld_u128: InternalGas, "ld_u128", MUL],
        // casting
        [cast_u8: InternalGas, "cast_u8", 2 * MUL],
        [cast_u64: InternalGas, "cast_u64", MUL],
        [cast_u128: InternalGas, "cast_u128", MUL],
        [
            mut_borrow_field_generic: InternalGas,
            "mut_borrow_field_generic.base",
            MUL
        ],
        [
            imm_borrow_field_generic: InternalGas,
            "imm_borrow_field_generic.base",
            MUL
        ],
        [
            call_generic_base: InternalGas,
            "call_generic.base",
            582 * MUL
        ],
        [
            call_generic_per_ty_arg: InternalGasPerArg,
            optional "call_generic.per_ty_arg",
            100 * MUL
        ],
        [
            call_generic_per_arg: InternalGasPerArg,
            optional "call_generic.per_arg",
            100 * MUL
        ],
        [pack_generic_base: InternalGas, "pack_generic.base", 2 * MUL],
        [
            pack_generic_per_field: InternalGasPerArg,
            optional "pack_generic.per_field",
            40 * MUL
        ],
        [
            unpack_generic_base: InternalGas,
            "unpack_generic.base",
            2 * MUL
        ],
        [
            unpack_generic_per_field: InternalGasPerArg,
            optional "unpack_generic.per_field",
            40 * MUL
        ],
        [
            exists_generic_base: InternalGas,
            "exists_generic.base",
            34 * MUL
        ],
        [
            mut_borrow_global_generic_base: InternalGas,
            "mut_borrow_global_generic.base",
            15 * MUL
        ],
        [
            imm_borrow_global_generic_base: InternalGas,
            "imm_borrow_global_generic.base",
            14 * MUL
        ],
        [
            move_from_generic_base: InternalGas,
            "move_from_generic.base",
            13 * MUL
        ],
        [
            move_to_generic_base: InternalGas,
            "move_to_generic.base",
            27 * MUL
        ],
        // vec
        [vec_pack_base: InternalGas, optional "vec_pack.base", 84 * MUL],
        [
            vec_pack_per_elem: InternalGasPerArg,
            optional "vec_pack.per_elem",
            40 * MUL
        ],
        [vec_len_base: InternalGas, optional "vec_len.base", 98 * MUL],
        [
            vec_imm_borrow_base: InternalGas,
            optional "vec_imm_borrow.base",
            1334 * MUL
        ],
        [
            vec_mut_borrow_base: InternalGas,
           optional "vec_mut_borrow.base",
            1902 * MUL
        ],
        [
            vec_push_back_base: InternalGas,
            optional "vec_push_back.base",
            53 * MUL
        ],
        [
            vec_pop_back_base: InternalGas,
            optional "vec_pop_back.base",
            227 * MUL
        ],
        [vec_unpack_base: InternalGas, optional "vec_unpack.base", 572 * MUL],
        [vec_swap_base: InternalGas, optional "vec_swap.base", 1436 * MUL],
    ]
);

impl InstructionGasParameters {
    pub fn simple_instr_cost(&self, instr: SimpleInstruction) -> PartialVMResult<InternalGas> {
        use SimpleInstruction::*;

        Ok(match instr {
            Nop => self.nop,

            Abort => self.abort,
            Ret => self.ret,

            BrTrue => self.br_true,
            BrFalse => self.br_false,
            Branch => self.branch,

            Pop => self.pop,
            LdU8 => self.ld_u8,
            LdU64 => self.ld_u64,
            LdU128 => self.ld_u128,
            LdTrue => self.ld_true,
            LdFalse => self.ld_false,

            ImmBorrowLoc => self.imm_borrow_loc,
            MutBorrowLoc => self.mut_borrow_loc,
            ImmBorrowField => self.imm_borrow_field,
            MutBorrowField => self.mut_borrow_field,
            ImmBorrowFieldGeneric => self.imm_borrow_field_generic,
            MutBorrowFieldGeneric => self.mut_borrow_field_generic,
            FreezeRef => self.freeze_ref,

            CastU8 => self.cast_u8,
            CastU64 => self.cast_u64,
            CastU128 => self.cast_u128,

            Add => self.add,
            Sub => self.sub,
            Mul => self.mul,
            Mod => self.mod_,
            Div => self.div,

            BitOr => self.bit_or,
            BitAnd => self.bit_and,
            Xor => self.xor,
            Shl => self.shl,
            Shr => self.shr,

            Or => self.or,
            And => self.and,
            Not => self.not,

            Lt => self.lt,
            Gt => self.gt,
            Le => self.le,
            Ge => self.ge,
        })
    }
}
