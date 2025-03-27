// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the official gas meter implementation, along with some top-level gas
//! parameters and traits to help manipulate them.

use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::gas_algebra::{
    AbstractMemorySize, InternalGasPerAbstractMemoryUnit, InternalGasPerArg, InternalGasPerByte,
    NumArgs,
};
use move_core_types::language_storage::ModuleId;
use move_core_types::{
    gas_algebra::{InternalGas, NumBytes},
    vm_status::StatusCode,
};
use move_vm_types::gas::{GasMeter, SimpleInstruction};
use move_vm_types::views::{TypeView, ValueView};
use starcoin_gas_algebra_ext::{
    FromOnChainGasSchedule, Gas, InitialGasSchedule, ToOnChainGasSchedule,
};
#[cfg(feature = "testing")]
use starcoin_logger::prelude::*;
use std::collections::BTreeMap;

use move_binary_format::file_format_common::Opcodes;
use starcoin_gas_algebra_ext::InstructionGasParameters;
use starcoin_gas_algebra_ext::TransactionGasParameters;

/// The size in bytes for a reference on the stack
const REFERENCE_SIZE: AbstractMemorySize = AbstractMemorySize::new(8);

/// For exists checks on data that doesn't exists this is the multiplier that is used.
const MIN_EXISTS_DATA_SIZE: AbstractMemorySize = AbstractMemorySize::new(100);

/// Gas parameters for all native functions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeGasParameters {
    pub move_stdlib: move_stdlib::natives::GasParameters,
    pub nursery: move_stdlib::natives::NurseryGasParameters,
    pub starcoin_natives: starcoin_natives::GasParameters,
    pub table: move_table_extension::GasParameters,
}

impl FromOnChainGasSchedule for NativeGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            move_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            nursery: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            starcoin_natives: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            table: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
        })
    }
}

impl ToOnChainGasSchedule for NativeGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entries = self.move_stdlib.to_on_chain_gas_schedule();
        entries.extend(self.nursery.to_on_chain_gas_schedule());
        entries.extend(self.starcoin_natives.to_on_chain_gas_schedule());
        entries.extend(self.table.to_on_chain_gas_schedule());
        entries
    }
}

impl NativeGasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            nursery: move_stdlib::natives::NurseryGasParameters::zeros(),
            starcoin_natives: starcoin_natives::GasParameters::zeros(),
            table: move_table_extension::GasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for NativeGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
            nursery: InitialGasSchedule::initial(),
            starcoin_natives: InitialGasSchedule::initial(),
            table: InitialGasSchedule::initial(),
        }
    }
}

/// Gas parameters for everything that is needed to run the Starcoin blockchain, including
/// instructions, transactions and native functions from various packages.
#[derive(Debug, Clone)]
pub struct StarcoinGasParameters {
    pub instr: InstructionGasParameters,
    pub txn: TransactionGasParameters,
    pub natives: NativeGasParameters,
}

impl FromOnChainGasSchedule for StarcoinGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            natives: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            instr: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            txn: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
        })
    }
}

impl ToOnChainGasSchedule for StarcoinGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entries = self.instr.to_on_chain_gas_schedule();
        entries.extend(self.txn.to_on_chain_gas_schedule());
        entries.extend(self.natives.to_on_chain_gas_schedule());
        entries
    }
}

impl StarcoinGasParameters {
    // Only used for genesis and for tests where we need a cost table and
    // don't have a genesis storage state.
    pub fn zeros() -> Self {
        Self {
            instr: InstructionGasParameters::zeros(),
            txn: TransactionGasParameters::zeros(),
            natives: NativeGasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for StarcoinGasParameters {
    fn initial() -> Self {
        Self {
            instr: InitialGasSchedule::initial(),
            txn: InitialGasSchedule::initial(),
            natives: InitialGasSchedule::initial(),
        }
    }
}

/// The official gas meter used inside the Starcoin VM.
/// It maintains an internal gas counter, measured in internal gas units, and carries an environment
/// consisting all the gas parameters, which it can lookup when performing gas calculations.
pub struct StarcoinGasMeter {
    gas_params: StarcoinGasParameters,
    balance: InternalGas,
    charge: bool,
}

impl StarcoinGasMeter {
    pub fn new(gas_params: StarcoinGasParameters, balance: impl Into<Gas>) -> Self {
        let balance = balance.into().to_unit_with_params(&gas_params.txn);
        Self {
            gas_params,
            balance,
            charge: true,
        }
    }

    pub fn balance(&self) -> Gas {
        self.balance
            .to_unit_round_down_with_params(&self.gas_params.txn)
    }

    pub fn deduct_gas(&mut self, amount: InternalGas) -> PartialVMResult<()> {
        if !self.charge {
            return Ok(());
        }
        match self.balance.checked_sub(amount) {
            Some(new_balance) => {
                self.balance = new_balance;
                Ok(())
            }
            None => {
                self.balance = 0.into();
                Err(PartialVMError::new(StatusCode::OUT_OF_GAS))
            }
        }
    }

    pub fn set_metering(&mut self, enabled: bool) {
        self.charge = enabled;
    }

    pub fn get_metering(&self) -> bool {
        self.charge
    }

    pub fn charge_intrinsic_gas_for_transaction(&mut self, txn_size: NumBytes) -> VMResult<()> {
        let cost = self.gas_params.txn.calculate_intrinsic_gas(txn_size);
        #[cfg(feature = "testing")]
        info!(
            "charge_intrinsic_gas cost InternalGasUnits({}) {}",
            cost, self.charge
        );
        self.deduct_gas(cost)
            .map_err(|e| e.finish(Location::Undefined))
    }

    pub fn cal_write_set_gas(&self) -> InternalGas {
        self.gas_params.txn.cal_write_set_gas()
    }
}

#[inline]
fn cal_instr_with_size(
    per_mem: InternalGasPerAbstractMemoryUnit,
    size: AbstractMemorySize,
) -> InternalGas {
    let size = std::cmp::max(1.into(), size);
    per_mem * size
}

#[inline]
fn cal_instr_with_arg(per_arg: InternalGasPerArg, size: NumArgs) -> InternalGas {
    let size = std::cmp::max(1.into(), size);
    per_arg * size
}

#[inline]
fn cal_instr_with_byte(per_arg: InternalGasPerByte, size: NumBytes) -> InternalGas {
    let size = std::cmp::max(1.into(), size);
    per_arg * size
}

#[allow(dead_code)]
#[inline]
fn simple_instr_to_opcode(instr: SimpleInstruction) -> Opcodes {
    match instr {
        SimpleInstruction::Nop => Opcodes::NOP,
        SimpleInstruction::Ret => Opcodes::RET,

        SimpleInstruction::BrTrue => Opcodes::BR_TRUE,
        SimpleInstruction::BrFalse => Opcodes::BR_FALSE,
        SimpleInstruction::Branch => Opcodes::BRANCH,

        SimpleInstruction::LdU8 => Opcodes::LD_U8,
        SimpleInstruction::LdU64 => Opcodes::LD_U64,
        SimpleInstruction::LdU128 => Opcodes::LD_U128,
        SimpleInstruction::LdTrue => Opcodes::LD_TRUE,
        SimpleInstruction::LdFalse => Opcodes::LD_FALSE,

        SimpleInstruction::FreezeRef => Opcodes::FREEZE_REF,
        SimpleInstruction::MutBorrowLoc => Opcodes::MUT_BORROW_LOC,
        SimpleInstruction::ImmBorrowLoc => Opcodes::IMM_BORROW_LOC,
        SimpleInstruction::ImmBorrowField => Opcodes::IMM_BORROW_FIELD,
        SimpleInstruction::MutBorrowField => Opcodes::MUT_BORROW_FIELD,
        SimpleInstruction::ImmBorrowFieldGeneric => Opcodes::IMM_BORROW_FIELD_GENERIC,
        SimpleInstruction::MutBorrowFieldGeneric => Opcodes::MUT_BORROW_FIELD_GENERIC,

        SimpleInstruction::CastU8 => Opcodes::CAST_U8,
        SimpleInstruction::CastU64 => Opcodes::CAST_U64,
        SimpleInstruction::CastU128 => Opcodes::CAST_U128,

        SimpleInstruction::Add => Opcodes::ADD,
        SimpleInstruction::Sub => Opcodes::SUB,
        SimpleInstruction::Mul => Opcodes::MUL,
        SimpleInstruction::Mod => Opcodes::MOD,
        SimpleInstruction::Div => Opcodes::DIV,

        SimpleInstruction::BitOr => Opcodes::BIT_OR,
        SimpleInstruction::BitAnd => Opcodes::BIT_AND,
        SimpleInstruction::Xor => Opcodes::XOR,
        SimpleInstruction::Shl => Opcodes::SHL,
        SimpleInstruction::Shr => Opcodes::SHR,

        SimpleInstruction::Or => Opcodes::OR,
        SimpleInstruction::And => Opcodes::AND,
        SimpleInstruction::Not => Opcodes::NOT,

        SimpleInstruction::Lt => Opcodes::LT,
        SimpleInstruction::Gt => Opcodes::GT,
        SimpleInstruction::Le => Opcodes::LE,
        SimpleInstruction::Ge => Opcodes::GE,

        SimpleInstruction::Abort => Opcodes::ABORT,

        SimpleInstruction::LdU16 => Opcodes::LD_U16,
        SimpleInstruction::LdU32 => Opcodes::LD_U32,
        SimpleInstruction::LdU256 => Opcodes::LD_U256,

        SimpleInstruction::CastU16 => Opcodes::CAST_U16,
        SimpleInstruction::CastU32 => Opcodes::CAST_U32,
        SimpleInstruction::CastU256 => Opcodes::CAST_U256,
    }
}

impl GasMeter for StarcoinGasMeter {
    fn balance_internal(&self) -> InternalGas {
        self.balance
    }

    #[inline]
    fn charge_simple_instr(&mut self, instr: SimpleInstruction) -> PartialVMResult<()> {
        let cost = self.gas_params.instr.simple_instr_cost(instr)?;
        #[cfg(feature = "testing")]
        info!(
            "simple_instr {:#?} cost InternalGasUnits({}) {}",
            simple_instr_to_opcode(instr),
            cost,
            self.charge
        );
        self.deduct_gas(cost)
    }

    fn charge_pop(&mut self, _popped_val: impl ValueView) -> PartialVMResult<()> {
        let params = &self.gas_params.instr;
        let cost = params.pop;
        #[cfg(feature = "testing")]
        info!(
            "simple_instr pop cost InternalGasUnits({}) {}",
            cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_call(
        &mut self,
        _module_id: &ModuleId,
        _func_name: &str,
        args: impl ExactSizeIterator<Item = impl ValueView>,
        _num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let params = &self.gas_params.instr;
        // Note args.len() may be zero, can't use args.len() + 1 directly
        let cost1 = cal_instr_with_arg(params.call_per_arg, NumArgs::new(1));
        #[cfg(feature = "testing")]
        info!("CALL cost InternalGasUnits({}) {}", cost1, self.charge);
        let cost2 = cal_instr_with_arg(params.call_per_arg, NumArgs::new(args.len() as u64));
        #[cfg(feature = "testing")]
        info!("CALL cost InternalGasUnits({}) {}", cost2, self.charge);
        self.deduct_gas(cost1 + cost2)
    }

    #[inline]
    fn charge_call_generic(
        &mut self,
        _module_id: &ModuleId,
        _func_name: &str,
        ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        args: impl ExactSizeIterator<Item = impl ValueView>,
        _num_locals: NumArgs,
    ) -> PartialVMResult<()> {
        let params = &self.gas_params.instr;
        // Note args.len() may be zero, can't use ty_args.len() + args.len() + 1 directly
        let cost1 = cal_instr_with_arg(
            params.call_generic_per_arg,
            NumArgs::new((ty_args.len() + 1) as u64),
        );
        #[cfg(feature = "testing")]
        info!(
            "CALL_GENERIC cost InternalGasUnits({}) {}",
            cost1, self.charge
        );
        let cost2 =
            cal_instr_with_arg(params.call_generic_per_arg, NumArgs::new(args.len() as u64));
        #[cfg(feature = "testing")]
        info!(
            "CALL_GENERIC cost InternalGasUnits({}) {}",
            cost2, self.charge
        );
        self.deduct_gas(cost1 + cost2)
    }

    #[inline]
    fn charge_ld_const(&mut self, size: NumBytes) -> PartialVMResult<()> {
        let instr = &self.gas_params.instr;
        let cost = cal_instr_with_byte(instr.ld_const_per_byte, size);
        #[cfg(feature = "testing")]
        info!("LD_CONST cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    fn charge_ld_const_after_deserialization(
        &mut self,
        _val: impl ValueView,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    #[inline]
    fn charge_copy_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        let instr_params = &self.gas_params.instr;
        let cost = cal_instr_with_size(
            instr_params.copy_loc_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("COPY_LOC cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_move_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        let instr_params = &self.gas_params.instr;
        let cost = cal_instr_with_size(
            instr_params.move_loc_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("MOVE_LOC cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_store_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        let instr_params = &self.gas_params.instr;
        let cost = cal_instr_with_size(
            instr_params.st_loc_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("ST_LOC cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_pack(
        &mut self,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let field_count = AbstractMemorySize::new(args.len() as u64);
        let params = &self.gas_params.instr;
        let size = args.fold(field_count, |acc, val| {
            acc + val.legacy_abstract_memory_size()
        });
        let cost = match is_generic {
            false => cal_instr_with_size(params.pack_per_abs_mem_unit, size),
            true => cal_instr_with_size(params.pack_generic_per_abs_mem_unit, size),
        };
        #[cfg(feature = "testing")]
        {
            if is_generic {
                info!(
                    "PACK_GENERIC cost InternalGasUnits({}) {}",
                    cost, self.charge
                );
            } else {
                info!("PACK cost InternalGasUnits({}) {}", cost, self.charge);
            }
        }
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_unpack(
        &mut self,
        is_generic: bool,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        #[cfg(feature = "testing")]
        let opcode = {
            if is_generic {
                Opcodes::UNPACK_GENERIC
            } else {
                Opcodes::UNPACK
            }
        };
        let params = &self.gas_params.instr;
        let param = if is_generic {
            params.unpack_generic_per_abs_mem_unit
        } else {
            params.unpack_per_abs_mem_unit
        };
        let field_count = AbstractMemorySize::new(args.len() as u64);
        let mut cost = cal_instr_with_size(param, field_count);
        #[cfg(feature = "testing")]
        info!(
            "{:#?} cost InternalGasUnits({}) {}",
            opcode, cost, self.charge
        );
        for val in args {
            let cost2 = cal_instr_with_size(param, val.legacy_abstract_memory_size());
            #[cfg(feature = "testing")]
            info!(
                "{:#?} cost InternalGasUnits({}) {}",
                opcode, cost2, self.charge
            );
            cost += cost2;
        }
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_read_ref(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        let cost = cal_instr_with_size(
            self.gas_params.instr.read_ref_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("READ_REF cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_write_ref(
        &mut self,
        val: impl ValueView,
        _old_val: impl ValueView,
    ) -> PartialVMResult<()> {
        let cost = cal_instr_with_size(
            self.gas_params.instr.write_ref_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("WRITE_REF cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_eq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        let instr_params = &self.gas_params.instr;
        let cost = cal_instr_with_size(
            instr_params.eq_per_abs_mem_unit,
            lhs.legacy_abstract_memory_size() + rhs.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("EQ cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_neq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        let instr_params = &self.gas_params.instr;
        let cost = cal_instr_with_size(
            instr_params.eq_per_abs_mem_unit,
            lhs.legacy_abstract_memory_size() + rhs.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!("NEQ cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_borrow_global(
        &mut self,
        _is_mut: bool,
        is_generic: bool,
        _ty: impl TypeView,
        is_success: bool,
    ) -> PartialVMResult<()> {
        let cost = if !is_success {
            0.into()
        } else {
            let params = &self.gas_params.instr;
            // NOTE only use mut see https://github.com/starcoinorg/move/blob/starcoin-main/language/move-vm/runtime/src/interpreter.rs#L1018-L1030
            let param = match is_generic {
                false => params.mut_borrow_global_per_abs_mem_unit,
                true => params.mut_borrow_global_generic_per_abs_mem_unit,
            };
            cal_instr_with_size(param, REFERENCE_SIZE)
        };
        #[cfg(feature = "testing")]
        let opcode = match is_generic {
            false => Opcodes::MUT_BORROW_GLOBAL,
            true => Opcodes::MUT_BORROW_GLOBAL_GENERIC,
        };
        #[cfg(feature = "testing")]
        info!(
            "{:#?} cost InternalGasUnits({}) {}",
            opcode, cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_exists(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        exists: bool,
    ) -> PartialVMResult<()> {
        let params = &self.gas_params.instr;
        let param = match is_generic {
            false => params.exists_per_abs_mem_unit,
            true => params.exists_generic_per_abs_mem_unit,
        };
        let size = match exists {
            false => MIN_EXISTS_DATA_SIZE,
            true => REFERENCE_SIZE,
        };
        let cost = cal_instr_with_size(param, size);
        #[cfg(feature = "testing")]
        let opcode = match is_generic {
            false => Opcodes::EXISTS,
            true => Opcodes::EXISTS_GENERIC,
        };
        #[cfg(feature = "testing")]
        info!(
            "{:#?} cost InternalGasUnits({}) {}",
            opcode, cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_move_from(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        if let Some(val) = val {
            let params = &self.gas_params.instr;
            let param = match is_generic {
                false => params.move_from_per_abs_mem_unit,
                true => params.move_from_generic_per_abs_mem_unit,
            };
            let cost = cal_instr_with_size(param, val.legacy_abstract_memory_size());
            #[cfg(feature = "testing")]
            let opcode = match is_generic {
                false => Opcodes::MOVE_FROM,
                true => Opcodes::MOVE_FROM_GENERIC,
            };
            #[cfg(feature = "testing")]
            info!(
                "MOVE_FROM {:#?} cost InternalGasUnits({}) {}",
                opcode, cost, self.charge
            );
            return self.deduct_gas(cost);
        }
        Ok(())
    }

    #[inline]
    fn charge_move_to(
        &mut self,
        is_generic: bool,
        _ty: impl TypeView,
        val: impl ValueView,
        is_success: bool,
    ) -> PartialVMResult<()> {
        let cost = if !is_success {
            0.into()
        } else {
            let params = &self.gas_params.instr;
            let param = match is_generic {
                false => params.move_to_per_abs_mem_unit,
                true => params.move_to_generic_per_abs_mem_unit,
            };
            cal_instr_with_size(param, val.legacy_abstract_memory_size())
        };
        #[cfg(feature = "testing")]
        let opcode = match is_generic {
            false => Opcodes::MOVE_TO,
            true => Opcodes::MOVE_TO_GENERIC,
        };
        #[cfg(feature = "testing")]
        info!(
            "charge_MOVE_TO {:#?} cost InternalGasUnits({}) {}",
            opcode, cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_pack<'a>(
        &mut self,
        _ty: impl TypeView + 'a,
        args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let num_args = NumArgs::new(args.len() as u64);
        let params = &self.gas_params.instr;
        let cost = cal_instr_with_arg(params.vec_pack_per_elem, num_args);
        #[cfg(feature = "testing")]
        info!("VEC_PACK cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_len(&mut self, _ty: impl TypeView) -> PartialVMResult<()> {
        let cost = self.gas_params.instr.vec_len_base;
        #[cfg(feature = "testing")]
        info!("VEC_LEN cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_borrow(
        &mut self,
        is_mut: bool,
        _ty: impl TypeView,
        is_success: bool,
    ) -> PartialVMResult<()> {
        let cost = if !is_success {
            0.into()
        } else {
            let params = &self.gas_params.instr;
            match is_mut {
                false => params.vec_imm_borrow_base,
                true => params.vec_mut_borrow_base,
            }
        };
        #[cfg(feature = "testing")]
        let opcode = match is_mut {
            false => Opcodes::VEC_MUT_BORROW,
            true => Opcodes::VEC_IMM_BORROW,
        };
        #[cfg(feature = "testing")]
        info!(
            "{:#?} cost InternalGasUnits({}) {}",
            opcode, cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_push_back(
        &mut self,
        _ty: impl TypeView,
        val: impl ValueView,
    ) -> PartialVMResult<()> {
        let cost = cal_instr_with_size(
            self.gas_params.instr.vec_push_back_per_abs_mem_unit,
            val.legacy_abstract_memory_size(),
        );
        #[cfg(feature = "testing")]
        info!(
            "VEC_PUSH_BACK cost InternalGasUnits({}) {}",
            cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_pop_back(
        &mut self,
        _ty: impl TypeView,
        _val: Option<impl ValueView>,
    ) -> PartialVMResult<()> {
        let cost = self.gas_params.instr.vec_pop_back_base;
        #[cfg(feature = "testing")]
        info!(
            "VEC_POP_BACK cost InternalGasUnits({}) {}",
            cost, self.charge
        );
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_unpack(
        &mut self,
        _ty: impl TypeView,
        expect_num_elements: NumArgs,
        _elems: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        let cost = cal_instr_with_arg(
            self.gas_params.instr.vec_unpack_per_expected_elem,
            expect_num_elements,
        );
        #[cfg(feature = "testing")]
        info!("VEC_UNPACK cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_vec_swap(&mut self, _ty: impl TypeView) -> PartialVMResult<()> {
        let cost = self.gas_params.instr.vec_swap_base;
        #[cfg(feature = "testing")]
        info!("VEC_SWAP cost InternalGasUnits({}) {}", cost, self.charge);
        self.deduct_gas(cost)
    }

    #[inline]
    fn charge_load_resource(
        &mut self,
        _loaded: Option<(NumBytes, impl ValueView)>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    #[inline]
    fn charge_native_function(
        &mut self,
        amount: InternalGas,
        _ret_vals: Option<impl ExactSizeIterator<Item = impl ValueView>>,
    ) -> PartialVMResult<()> {
        #[cfg(feature = "testing")]
        info!(
            "NATIVE_FUNCTION cost InternalGasUnits({}) {}",
            amount, self.charge
        );
        self.deduct_gas(amount)
    }

    fn charge_native_function_before_execution(
        &mut self,
        _ty_args: impl ExactSizeIterator<Item = impl TypeView>,
        _args: impl ExactSizeIterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }

    fn charge_drop_frame(
        &mut self,
        _locals: impl Iterator<Item = impl ValueView>,
    ) -> PartialVMResult<()> {
        Ok(())
    }
}
