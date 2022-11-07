// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module defines all the gas parameters and formulae for instructions, along with their
//! initial values in the genesis and a mapping between the Rust representation and the on-chain
//! gas schedule.

// XXX FIXME YSG (config/src/genesis_config.rs)
use crate::gas_meter::EXECUTION_GAS_MULTIPLIER as MUL;
use move_core_types::gas_algebra::InternalGas;

crate::params::define_gas_parameters!(
    InstructionGasParameters,
    "instr",
    [
        [pop: InternalGas, "pop", 1 * MUL],
        [ret: InternalGas, "ret", 638 * MUL],
        [br_true: InternalGas, "br_true", 1 * MUL],
        [br_false: InternalGas, "br_false", 1 * MUL],
        [branch: InternalGas, "branch", 1 * MUL],
        [ld_u64: InternalGas, "ld_u64", 1 * MUL],
        [ld_const_base: InternalGas, "ld_const.base", 1 * MUL],
        [ld_true: InternalGas, "ld_true", 1 * MUL],
        [ld_false: InternalGas, "ld_false", 1 * MUL],
        [copy_loc_base: InternalGas, "copy_loc.base", 1 * MUL],
        [move_loc_base: InternalGas, "move_loc.base", 120 * MUL],
        [st_loc_base: InternalGas, "st_loc.base", 120 * MUL],
        [mut_borrow_loc: InternalGas, "mut_borrow_loc", 60 * MUL],
        [imm_borrow_loc: InternalGas, "imm_borrow_loc", 60 * MUL],
        [mut_borrow_field: InternalGas, "mut_borrow_field", 200 * MUL],
        [imm_borrow_field: InternalGas, "imm_borrow_field", 200 * MUL],
        [call_base: InternalGas, "call.base", 1000 * MUL],
        [pack_base: InternalGas, "pack.base", 220 * MUL],
        [unpack_base: InternalGas, "unpack.base", 220 * MUL],
        [read_ref_base: InternalGas, "read_ref.base", 200 * MUL],
        [write_ref_base: InternalGas, "write_ref.base", 200 * MUL],
        [add: InternalGas, "add", 160 * MUL],
        [sub: InternalGas, "sub", 160 * MUL],
        [mul: InternalGas, "mul", 160 * MUL],
        [mod_: InternalGas, "mod", 160 * MUL],
        [div: InternalGas, "div", 160 * MUL],
        [bit_or: InternalGas, "bit_or", 160 * MUL],
        [bit_and: InternalGas, "bit_and", 160 * MUL],
        [xor: InternalGas, "bit_xor", 160 * MUL],
        [or: InternalGas, "or", 160 * MUL],
        [and: InternalGas, "and", 160 * MUL],
        [not: InternalGas, "not", 160 * MUL],
        [eq_base: InternalGas, "eq.base", 100 * MUL],
        [neq_base: InternalGas, "neq.base", 100 * MUL],
        // comparison
        [lt: InternalGas, "lt", 160 * MUL],
        [gt: InternalGas, "gt", 160 * MUL],
        [le: InternalGas, "le", 160 * MUL],
        [ge: InternalGas, "ge", 160 * MUL],
        [abort: InternalGas, "abort", 60 * MUL],
        // nop
        [nop: InternalGas, "nop", 10 * MUL],
        [exists_base: InternalGas, "exists.base", 250 * MUL],
        [
            mut_borrow_global_base: InternalGas,
            "mut_borrow_global.base",
            500 * MUL
        ],
        [
            imm_borrow_global_base: InternalGas,
            "imm_borrow_global.base",
            500 * MUL
        ],
        [move_from_base: InternalGas, "move_from.base", 350 * MUL],
        [move_to_base: InternalGas, "move_to.base", 500 * MUL],
        [freeze_ref: InternalGas, "freeze_ref", 10 * MUL],
        [shl: InternalGas, "bit_shl", 160 * MUL],
        [shr: InternalGas, "bit_shr", 160 * MUL],
        [ld_u8: InternalGas, "ld_u8", 60 * MUL],
        [ld_u128: InternalGas, "ld_u128", 80 * MUL],
        // casting
        [cast_u8: InternalGas, "cast_u8", 120 * MUL],
        [cast_u64: InternalGas, "cast_u64", 120 * MUL],
        [cast_u128: InternalGas, "cast_u128", 120 * MUL],
        [
            mut_borrow_field_generic: InternalGas,
            "mut_borrow_field_generic",
            200 * MUL
        ],
        [
            imm_borrow_field_generic: InternalGas,
            "imm_borrow_field_generic",
            200 * MUL
        ],
        [
            move_from_generic_base: InternalGas,
            "move_from_generic.base",
            350 * MUL
        ],
        [
            move_to_generic_base: InternalGas,
            "move_to_generic.base",
            500 * MUL
        ],
        [vec_pack_base: InternalGas, "vec_pack.base", 600 * MUL],
        // vec
        [vec_len_base: InternalGas, "vec_len.base", 220 * MUL],
        [
            vec_imm_borrow_base: InternalGas,
            "vec_imm_borrow.base",
            330 * MUL
        ],
        [
            vec_mut_borrow_base: InternalGas,
            "vec_mut_borrow.base",
            330 * MUL
        ],
        [
            vec_push_back_base: InternalGas,
            "vec_push_back.base",
            380 * MUL
        ],
        [
            vec_pop_back_base: InternalGas,
            "vec_pop_back.base",
            260 * MUL
        ],
        [vec_unpack_base: InternalGas, "vec_unpack.base", 500 * MUL],
        [vec_swap_base: InternalGas, "vec_swap.base", 300 * MUL],
    ]
);
