use parking_lot::{Mutex, MutexGuard};
use starcoin_crypto::HashValue;
use starcoin_executor::VMMetrics;
use starcoin_storage::Store2;
use starcoin_vm2_statedb::ChainStateDB as ChainStateDB2;
use starcoin_vm2_vm_runtime::starcoin_vm::StarcoinVM as StarcoinVM2;
use starcoin_vm2_vm_types::transaction::SignedUserTransaction as SignedUserTransaction2;
use starcoin_vm2_vm_types::vm_status::VMStatus as Vm2Status;
use std::sync::Arc;

pub struct VerifierPool {
    storage2: Arc<dyn Store2>,
    entries: Vec<Arc<Mutex<VerifierEntry>>>,
    vm_metrics: Option<VMMetrics>,
}

struct VerifierEntry {
    state_root1: Option<HashValue>,
    state_root2: Option<HashValue>,
    statedb2: Option<ChainStateDB2>,
    vm2: Option<StarcoinVM2>,
}

pub(crate) struct VerifierGuard<'a> {
    guard: MutexGuard<'a, VerifierEntry>,
}

impl VerifierPool {
    pub fn new(size: usize, storage2: Arc<dyn Store2>, vm_metrics: Option<VMMetrics>) -> Self {
        let pool_size = size.max(1);
        let entries = (0..pool_size)
            .map(|_| {
                Arc::new(Mutex::new(VerifierEntry::new(
                    storage2.clone(),
                    vm_metrics.clone(),
                )))
            })
            .collect();
        Self {
            storage2,
            entries,
            vm_metrics,
        }
    }

    pub(crate) fn checkout(
        &self,
        state_root1: HashValue,
        state_root2: HashValue,
    ) -> VerifierGuard<'_> {
        for entry in &self.entries {
            if let Some(guard) = entry.try_lock() {
                if guard.matches(state_root1, state_root2) {
                    return VerifierGuard { guard };
                }
            }
        }

        for entry in &self.entries {
            if let Some(mut guard) = entry.try_lock() {
                if !guard.matches(state_root1, state_root2) {
                    guard.refresh(
                        state_root1,
                        state_root2,
                        &self.storage2,
                        self.vm_metrics.clone(),
                    );
                }
                return VerifierGuard { guard };
            }
        }

        let mut guard = self.entries[0].lock();
        if !guard.matches(state_root1, state_root2) {
            guard.refresh(
                state_root1,
                state_root2,
                &self.storage2,
                self.vm_metrics.clone(),
            );
        }
        VerifierGuard { guard }
    }

    pub(crate) fn invalidate_all(&self) {
        for entry in &self.entries {
            if let Some(mut guard) = entry.try_lock() {
                guard.reset();
            }
        }
    }
}

impl VerifierEntry {
    fn new(_storage2: Arc<dyn Store2>, _vm_metrics: Option<VMMetrics>) -> Self {
        Self {
            state_root1: None,
            state_root2: None,
            statedb2: None,
            vm2: None,
        }
    }

    fn matches(&self, state_root1: HashValue, state_root2: HashValue) -> bool {
        self.state_root1 == Some(state_root1) && self.state_root2 == Some(state_root2)
    }

    fn refresh(
        &mut self,
        state_root1: HashValue,
        state_root2: HashValue,
        storage2: &Arc<dyn Store2>,
        vm_metrics: Option<VMMetrics>,
    ) {
        self.state_root1 = Some(state_root1);
        self.state_root2 = Some(state_root2);
        let statedb2 = ChainStateDB2::new(storage2.clone().into_super_arc(), Some(state_root2));
        let vm2 = StarcoinVM2::new(vm_metrics, &statedb2);
        self.statedb2 = Some(statedb2);
        self.vm2 = Some(vm2);
    }

    fn reset(&mut self) {
        self.state_root1 = None;
        self.state_root2 = None;
        self.statedb2 = None;
        self.vm2 = None;
    }

    fn verify_vm2(&mut self, txn: SignedUserTransaction2) -> Option<Vm2Status> {
        let vm2 = self
            .vm2
            .as_mut()
            .expect("verifier pool entry must be initialized before verify_vm2");
        let statedb2 = self
            .statedb2
            .as_ref()
            .expect("verifier pool entry must be initialized before verify_vm2");
        vm2.verify_transaction_cached_config(statedb2, txn)
    }
}

impl VerifierGuard<'_> {
    pub(crate) fn verify_vm2(&mut self, txn: SignedUserTransaction2) -> Option<Vm2Status> {
        self.guard.verify_vm2(txn)
    }

    pub(crate) fn state_roots(&self) -> (Option<HashValue>, Option<HashValue>) {
        (self.guard.state_root1, self.guard.state_root2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::prelude::*;
    use sp_utils::thread_pool::RAYON_EXEC_POOL;
    use starcoin_config::NodeConfig;
    use starcoin_genesis::Genesis;
    use starcoin_storage::Store;
    use starcoin_transaction_builder::vm2::build_transfer_from_association;
    use starcoin_vm2_vm_types::{
        account_address::AccountAddress as Vm2AccountAddress,
        transaction::Transaction as Transaction2,
    };
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn setup_pool_and_tx(
        pool_size: usize,
    ) -> (
        Arc<VerifierPool>,
        HashValue,
        HashValue,
        SignedUserTransaction2,
    ) {
        let config = NodeConfig::random_for_test();
        let net = config.net().clone();
        let (storage, storage2, chain_info, ..) =
            Genesis::init_storage_for_test(&net).expect("init storage for test");
        let multi_state = storage
            .get_vm_multi_state(chain_info.head().id())
            .expect("multi state from genesis");
        let pool = Arc::new(VerifierPool::new(pool_size, storage2.clone(), None));

        let receiver: Vm2AccountAddress = "0x2".parse().expect("valid vm2 address literal");
        let txn = build_transfer_from_association(
            receiver,
            0,
            10_000,
            net.time_service().now_secs() + 3_600,
            net.chain_id().id().into(),
            net.genesis_config2(),
        );
        let signed = match txn {
            Transaction2::UserTransaction(signed) => signed,
            _ => unreachable!("vm2 transfer should be user txn"),
        };

        (
            pool,
            multi_state.state_root1(),
            multi_state.state_root2(),
            signed,
        )
    }

    #[test]
    fn verifier_pool_lazy_init_and_verify() {
        let (pool, root1, root2, txn) = setup_pool_and_tx(1);
        {
            let entry = pool.entries[0].lock();
            assert!(entry.vm2.is_none());
            assert!(entry.statedb2.is_none());
        }

        let mut guard = pool.checkout(root1, root2);
        assert!(guard.guard.vm2.is_some());
        assert!(guard.guard.statedb2.is_some());
        assert_eq!(guard.state_roots(), (Some(root1), Some(root2)));
        assert!(guard.verify_vm2(txn).is_none());
    }

    #[test]
    fn verifier_pool_blocking_checkout_reuses_vm() {
        let (pool, root1, root2, txn) = setup_pool_and_tx(1);
        let first = pool.checkout(root1, root2);
        let first_ptr = first.guard.vm2.as_ref().unwrap() as *const _;

        let (tx, rx) = mpsc::channel();
        let pool_clone = Arc::clone(&pool);
        thread::spawn(move || {
            let guard = pool_clone.checkout(root1, root2);
            let ptr = guard.guard.vm2.as_ref().unwrap() as *const _ as usize;
            let roots = guard.state_roots();
            tx.send((ptr, roots)).unwrap();
        });

        thread::sleep(Duration::from_millis(50));
        drop(first);

        let (second_ptr, roots) = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(first_ptr as usize, second_ptr);
        assert_eq!(roots, (Some(root1), Some(root2)));

        let mut guard = pool.checkout(root1, root2);
        assert!(guard.verify_vm2(txn).is_none());
    }

    #[test]
    fn verifier_pool_refreshes_on_root_change() {
        let (pool, root1, root2, _) = setup_pool_and_tx(1);
        let guard = pool.checkout(root1, root2);
        assert_eq!(guard.state_roots(), (Some(root1), Some(root2)));
        drop(guard);

        let new_root1 = HashValue::random();
        let guard = pool.checkout(new_root1, root2);
        assert_eq!(guard.state_roots(), (Some(new_root1), Some(root2)));
        assert!(guard.guard.vm2.is_some());
    }

    #[test]
    fn verifier_pool_invalidate_rebuilds() {
        let (pool, root1, root2, _) = setup_pool_and_tx(1);
        {
            let guard = pool.checkout(root1, root2);
            assert!(guard.guard.vm2.is_some());
            assert_eq!(guard.state_roots(), (Some(root1), Some(root2)));
        }

        pool.invalidate_all();
        {
            let guard = pool.entries[0].lock();
            assert!(guard.vm2.is_none());
            assert!(guard.statedb2.is_none());
            assert_eq!(guard.state_root1, None);
            assert_eq!(guard.state_root2, None);
        }

        let guard = pool.checkout(root1, root2);
        assert!(guard.guard.vm2.is_some());
        assert!(guard.guard.statedb2.is_some());
        assert_eq!(guard.state_roots(), (Some(root1), Some(root2)));
    }

    #[test]
    fn verifier_pool_parallel_verify() {
        let (pool, root1, root2, txn) = setup_pool_and_tx(4);
        let txn = Arc::new(txn);
        RAYON_EXEC_POOL.install(|| {
            (0..8usize).into_par_iter().for_each(|_| {
                let mut guard = pool.checkout(root1, root2);
                assert!(guard.verify_vm2(txn.as_ref().clone()).is_none());
            });
        });
    }
}
