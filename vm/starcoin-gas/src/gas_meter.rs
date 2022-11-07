// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the official gas meter implementation, along with some top-level gas
//! parameters and traits to help manipulate them.

use gas_algebra_ext::{FromOnChainGasSchedule, Gas, InitialGasSchedule, ToOnChainGasSchedule};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    gas_algebra::{InternalGas, NumBytes},
    vm_status::StatusCode,
};
use std::collections::BTreeMap;
use move_core_types::gas_algebra::NumArgs;
use move_core_types::language_storage::ModuleId;
use move_vm_types::gas::{GasMeter, SimpleInstruction};
use move_vm_types::views::{TypeView, ValueView};

// Change log:
// - V3
//   - Add memory quota
//   - Storage charges:
//     - Distinguish between new and existing resources
//     - One item write comes with 1K free bytes
//     - abort with STORATGE_WRITE_LIMIT_REACHED if WriteOps or Events are too large
// - V2
//   - Table
//     - Fix the gas formula for loading resources so that they are consistent with other
//       global operations.
// - V1
//   - TBA

use gas_algebra_ext::InstructionGasParameters;
use gas_algebra_ext::TransactionGasParameters;

/// Gas parameters for all native functions.
#[derive(Debug, Clone)]
pub struct NativeGasParameters {
    pub move_stdlib: move_stdlib::natives::GasParameters,
    pub starcoin_natives: starcoin_natives::GasParameters,
    pub table: move_table_extension::GasParameters,
}

impl FromOnChainGasSchedule for NativeGasParameters {
    fn from_on_chain_gas_schedule(gas_schedule: &BTreeMap<String, u64>) -> Option<Self> {
        Some(Self {
            move_stdlib: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            starcoin_natives: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            table: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
        })
    }
}

impl ToOnChainGasSchedule for NativeGasParameters {
    fn to_on_chain_gas_schedule(&self) -> Vec<(String, u64)> {
        let mut entries = self.move_stdlib.to_on_chain_gas_schedule();
        entries.extend(self.starcoin_natives.to_on_chain_gas_schedule());
        entries.extend(self.table.to_on_chain_gas_schedule());
        entries
    }
}

impl NativeGasParameters {
    pub fn zeros() -> Self {
        Self {
            move_stdlib: move_stdlib::natives::GasParameters::zeros(),
            starcoin_natives: starcoin_natives::GasParameters::zeros(),
            table: move_table_extension::GasParameters::zeros(),
        }
    }
}

impl InitialGasSchedule for NativeGasParameters {
    fn initial() -> Self {
        Self {
            move_stdlib: InitialGasSchedule::initial(),
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
            instr: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            txn: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
            natives: FromOnChainGasSchedule::from_on_chain_gas_schedule(gas_schedule)?,
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

// XXX FIXME YSG StarcoinGasMeter for GasMeter
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

    #[inline]
    fn charge(&mut self, amount: InternalGas) -> PartialVMResult<()> {
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

    pub fn deduct_gas(&mut self, amount: InternalGas) -> PartialVMResult<()> {
        self.charge(amount)
    }

    pub fn charge_intrinsic_gas_for_transaction(&mut self, txn_size: NumBytes) -> VMResult<()> {
        let cost = self.gas_params.txn.calculate_intrinsic_gas(txn_size);
        self.charge(cost).map_err(|e| e.finish(Location::Undefined))
    }

    pub fn cal_write_set_gas(&self) -> InternalGas {
        self.gas_params.txn.cal_write_set_gas()
    }
}

// XXX FIXME YSG
impl GasMeter for StarcoinGasMeter {
    fn charge_simple_instr(&mut self, instr: SimpleInstruction) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_call(&mut self, module_id: &ModuleId, func_name: &str, args: impl ExactSizeIterator<Item=impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_call_generic(&mut self, module_id: &ModuleId, func_name: &str, ty_args: impl ExactSizeIterator<Item=impl TypeView>, args: impl ExactSizeIterator<Item=impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_ld_const(&mut self, size: NumBytes) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_copy_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_move_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_store_loc(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_pack(&mut self, is_generic: bool, args: impl ExactSizeIterator<Item=impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_unpack(&mut self, is_generic: bool, args: impl ExactSizeIterator<Item=impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_read_ref(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_write_ref(&mut self, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_eq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_neq(&mut self, lhs: impl ValueView, rhs: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_borrow_global(&mut self, is_mut: bool, is_generic: bool, ty: impl TypeView, is_success: bool) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_exists(&mut self, is_generic: bool, ty: impl TypeView, exists: bool) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_move_from(&mut self, is_generic: bool, ty: impl TypeView, val: Option<impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_move_to(&mut self, is_generic: bool, ty: impl TypeView, val: impl ValueView, is_success: bool) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_pack<'a>(&mut self, ty: impl TypeView + 'a, args: impl ExactSizeIterator<Item=impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_len(&mut self, ty: impl TypeView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_borrow(&mut self, is_mut: bool, ty: impl TypeView, is_success: bool) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_push_back(&mut self, ty: impl TypeView, val: impl ValueView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_pop_back(&mut self, ty: impl TypeView, val: Option<impl ValueView>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_unpack(&mut self, ty: impl TypeView, expect_num_elements: NumArgs) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_vec_swap(&mut self, ty: impl TypeView) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_load_resource(&mut self, loaded: Option<NumBytes>) -> PartialVMResult<()> {
        todo!()
    }

    fn charge_native_function(&mut self, amount: InternalGas) -> PartialVMResult<()> {
        todo!()
    }
}