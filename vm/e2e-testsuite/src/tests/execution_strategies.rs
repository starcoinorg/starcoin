// Copyright (c) The Starcoin Contributors
// SPDX-License-Identifier: Apache-2.0

use move_core_types::vm_status::VMStatus;
use starcoin_language_e2e_tests::account::Account;
use starcoin_language_e2e_tests::common_transactions::create_account_txn;
use starcoin_language_e2e_tests::execution_strategies::basic_strategy::BasicExecutor;
use starcoin_language_e2e_tests::execution_strategies::guided_strategy::{
    AnnotatedTransaction, GuidedExecutor, PartitionedGuidedStrategy, UnPartitionedGuidedStrategy,
};
use starcoin_language_e2e_tests::execution_strategies::multi_strategy::MultiExecutor;
use starcoin_language_e2e_tests::execution_strategies::random_strategy::RandomExecutor;
use starcoin_language_e2e_tests::execution_strategies::types::Executor;
use starcoin_vm_types::transaction::SignedUserTransaction;

fn txn(seq_num: u64) -> SignedUserTransaction {
    let account = Account::new();
    let diem_root = Account::new_starcoin_root();
    create_account_txn(&diem_root, &account, seq_num)
}

#[test]
fn test_execution_strategies() {
    {
        println!("===========================================================================");
        println!("TESTING BASIC STRATEGY");
        println!("===========================================================================");
        let big_block = (0..10).map(txn).collect();
        let mut exec = BasicExecutor::new();
        exec.execute_block(big_block).unwrap();
    }

    {
        println!("===========================================================================");
        println!("TESTING RANDOM STRATEGY");
        println!("===========================================================================");
        let big_block = (0..10).map(txn).collect();
        let mut exec = RandomExecutor::from_os_rng();
        exec.execute_block(big_block).unwrap();
    }

    {
        println!("===========================================================================");
        println!("TESTING GUIDED STRATEGY");
        println!("===========================================================================");
        let mut block1: Vec<_> = (0..10)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i))))
            .collect();
        block1.push(AnnotatedTransaction::Block);
        let mut block = (0..5)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 10))))
            .collect();
        block1.append(&mut block);
        block1.push(AnnotatedTransaction::Block);
        let mut block: Vec<_> = (0..7)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 15))))
            .collect();
        block1.append(&mut block);
        block1.push(AnnotatedTransaction::Block);
        let mut block = (0..20)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 22))))
            .collect();
        block1.append(&mut block);

        let mut exec = GuidedExecutor::new(PartitionedGuidedStrategy);
        exec.execute_block(block1).unwrap();
    }

    {
        println!("===========================================================================");
        println!("TESTING COMPOSED STRATEGY 1");
        println!("===========================================================================");
        let mut block1: Vec<_> = (0..10)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i))))
            .collect();
        block1.push(AnnotatedTransaction::Block);
        let mut block = (0..5)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 10))))
            .collect();
        block1.append(&mut block);
        block1.push(AnnotatedTransaction::Block);
        let mut block: Vec<_> = (0..7)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 15))))
            .collect();
        block1.append(&mut block);
        block1.push(AnnotatedTransaction::Block);
        let mut block = (0..20)
            .map(|i| AnnotatedTransaction::Txn(Box::new(txn(i + 22))))
            .collect();
        block1.append(&mut block);

        let mut exec = MultiExecutor::<AnnotatedTransaction, VMStatus>::new();
        exec.add_executor(GuidedExecutor::new(PartitionedGuidedStrategy));
        exec.add_executor(GuidedExecutor::new(UnPartitionedGuidedStrategy));
        exec.execute_block(block1).unwrap();
    }

    {
        println!("===========================================================================");
        println!("TESTING COMPOSED STRATEGY 2");
        println!("===========================================================================");
        let block = (0..10).map(txn).collect();

        let mut exec = MultiExecutor::<SignedUserTransaction, VMStatus>::new();
        exec.add_executor(RandomExecutor::from_os_rng());
        exec.add_executor(RandomExecutor::from_os_rng());
        exec.add_executor(RandomExecutor::from_os_rng());
        exec.execute_block(block).unwrap();
    }
}
