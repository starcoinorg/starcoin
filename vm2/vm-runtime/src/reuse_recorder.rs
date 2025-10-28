use starcoin_crypto::HashValue;
use starcoin_vm_types::state_store::state_key::StateKey;
use starcoin_vm_types::state_store::state_value::StateValue;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct ReadDescriptor {
    pub key: StateKey,
    pub from_storage: bool,
    pub existed: bool,
    pub value_hash: HashValue,
}

#[derive(Default)]
struct Recorder {
    per_txn: Vec<Vec<ReadDescriptor>>,
    current: Vec<ReadDescriptor>,
}

thread_local! {
    static RECORDER: RefCell<Option<Recorder>> = RefCell::new(None);
}

pub fn start() {
    RECORDER.with(|cell| {
        *cell.borrow_mut() = Some(Recorder::default());
    });
}

pub fn is_active() -> bool {
    RECORDER.with(|cell| cell.borrow().is_some())
}

pub fn reset_session() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
            rec.per_txn.clear();
        }
    });
}

pub fn begin_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
        }
    });
}

pub fn abort_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            rec.current.clear();
        }
    });
}

pub fn end_transaction() {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            let mut taken = Vec::new();
            std::mem::swap(&mut rec.current, &mut taken);
            rec.per_txn.push(taken);
        }
    });
}

pub fn record_read(state_key: &StateKey, from_storage: bool, value: Option<&StateValue>) {
    RECORDER.with(|cell| {
        if let Some(rec) = cell.borrow_mut().as_mut() {
            let (existed, value_hash) = match value {
                Some(val) => (true, HashValue::sha3_256_of(val.bytes().as_ref())),
                None => (false, HashValue::zero()),
            };

            rec.current.push(ReadDescriptor {
                key: state_key.clone(),
                from_storage,
                existed,
                value_hash,
            });
        }
    });
}

pub fn finish() -> Vec<Vec<ReadDescriptor>> {
    RECORDER.with(|cell| {
        cell.borrow_mut()
            .take()
            .map(|mut rec| {
                if !rec.current.is_empty() {
                    rec.per_txn.push(std::mem::take(&mut rec.current));
                }
                rec.per_txn
            })
            .unwrap_or_default()
    })
}
