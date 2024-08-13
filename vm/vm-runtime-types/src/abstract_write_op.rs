// Copyright (c) The Starcoin Core Contributors

// This is a ref in aptos-move/aptos-vm-types/src/abstract_write_op.rs

use starcoin_vm_types::write_set::WriteOp;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum AbstractResourceWriteOp {
    Write(WriteOp),
}
