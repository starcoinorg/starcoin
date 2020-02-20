// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::TransactionExecutor;
use anyhow::{Error, Result};
use chain_state::ChainState;
use config::VMConfig;
use crypto::{ed25519::compat, ed25519::*, hash::CryptoHash, traits::SigningKey, HashValue};
use once_cell::sync::Lazy;
use state_tree::SparseMerkleTree;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_state::AccountState,
    contract_event::ContractEvent,
    language_storage::{ModuleId, StructTag, TypeTag},
    transaction::{
        RawUserTransaction, Script, SignedUserTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};

enum MockTransaction {
    Mint {
        sender: AccountAddress,
        amount: u64,
    },
    Payment {
        sender: AccountAddress,
        recipient: AccountAddress,
        amount: u64,
    },
}

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(VMStatus::new(StatusCode::ABORTED).with_sub_status(10))
});

const MOCK_GAS_AMOUNT: u64 = 140_000;
const MOCK_GAS_PRICE: u64 = 1;

fn empty_tree() {
    unimplemented!()
}

pub struct MockChainState {
    //    state_tree: SparseMerkleTree,
}

impl MockChainState {
    // create empty chain state
    pub fn new() -> Self {
        MockChainState {
//            state_tree: empty_tree(),
        }
    }
    /// Commit and calculate new state root
    pub fn commit(&self) -> Result<HashValue> {
        unimplemented!()
    }

    /// flush data to db.
    pub fn flush(&self) -> Result<()> {
        unimplemented!()
    }
}

impl ChainState for MockChainState {
    fn get_by_hash(
        &self,
        storage_root: HashValue,
        resource_key: HashValue,
    ) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn get_code(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>> {
        unimplemented!()
    }

    fn get_account_state(&self, address: AccountAddress) -> Result<Option<AccountState>> {
        Ok(None)
    }

    fn is_genesis(&self) -> bool {
        unimplemented!()
    }

    fn state_root(&self) -> HashValue {
        unimplemented!()
    }

    fn set(&self, access_path: &AccessPath, value: Vec<u8>) -> Result<()> {
        unimplemented!()
    }

    fn set_at(
        &self,
        account_state: &AccountState,
        struct_tag: &StructTag,
        value: Vec<u8>,
    ) -> Result<()> {
        unimplemented!()
    }

    fn delete(&self, access_path: &AccessPath) -> Result<()> {
        unimplemented!()
    }

    fn delete_at(&self, account_state: &AccountState, struct_tag: &StructTag) -> Result<()> {
        unimplemented!()
    }

    fn set_code(&self, module_id: &ModuleId) -> Result<()> {
        unimplemented!()
    }
}

pub struct MockExecutor {
    config: VMConfig,
}

impl MockExecutor {
    /// Creates an executor in which no genesis state has been applied yet.
    pub fn new() -> Self {
        MockExecutor {
            config: VMConfig::default(),
        }
    }
}

impl TransactionExecutor for MockExecutor {
    fn execute_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: Transaction,
    ) -> Result<TransactionOutput> {
        // ToDo: output_cache is not used currently.
        let mut output_cache = HashMap::new();
        let mut output;

        match decode_transaction(&txn.as_signed_user_txn().unwrap()) {
            MockTransaction::Mint { sender, amount } => {
                let old_balance = read_balance(&output_cache, chain_state, sender);
                let new_balance = old_balance + amount;
                let old_seqnum = read_seqnum(&output_cache, chain_state, sender);
                let new_seqnum = old_seqnum + 1;

                output_cache.insert(balance_ap(sender), new_balance);
                output_cache.insert(seqnum_ap(sender), new_seqnum);

                let write_set = gen_mint_writeset(sender, new_balance, new_seqnum);
                output = TransactionOutput::new(vec![], 0, KEEP_STATUS.clone());
            }
            MockTransaction::Payment {
                sender,
                recipient,
                amount,
            } => {
                let sender_old_balance = read_balance(&output_cache, chain_state, sender);
                let recipient_old_balance = read_balance(&output_cache, chain_state, recipient);
                if sender_old_balance < amount {
                    output = TransactionOutput::new(vec![], 0, DISCARD_STATUS.clone());
                } else {
                    let sender_old_seqnum = read_seqnum(&output_cache, chain_state, sender);
                    let sender_new_seqnum = sender_old_seqnum + 1;
                    let sender_new_balance = sender_old_balance - amount;
                    let recipient_new_balance = recipient_old_balance + amount;

                    output_cache.insert(balance_ap(sender), sender_new_balance);
                    output_cache.insert(seqnum_ap(sender), sender_new_seqnum);
                    output_cache.insert(balance_ap(recipient), recipient_new_balance);

                    let write_set = gen_payment_writeset(
                        sender,
                        sender_new_balance,
                        sender_new_seqnum,
                        recipient,
                        recipient_new_balance,
                    );
                    output = TransactionOutput::new(
                        vec![],
                        0,
                        TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
                    );
                }
            }
        }
        Ok(output)
    }

    fn validate_transaction(
        config: &VMConfig,
        chain_state: &dyn ChainState,
        txn: SignedUserTransaction,
    ) -> Option<VMStatus> {
        None
    }
}

fn read_balance(
    output_cache: &HashMap<AccessPath, u64>,
    chain_state: &dyn ChainState,
    account: AccountAddress,
) -> u64 {
    let balance_access_path = balance_ap(account);
    match output_cache.get(&balance_access_path) {
        Some(balance) => *balance,
        None => read_balance_from_storage(chain_state, &balance_access_path),
    }
}

fn read_seqnum(
    output_cache: &HashMap<AccessPath, u64>,
    chain_state: &dyn ChainState,
    account: AccountAddress,
) -> u64 {
    let seqnum_access_path = seqnum_ap(account);
    match output_cache.get(&seqnum_access_path) {
        Some(seqnum) => *seqnum,
        None => read_seqnum_from_storage(chain_state, &seqnum_access_path),
    }
}

fn read_balance_from_storage(
    chain_state: &dyn ChainState,
    balance_access_path: &AccessPath,
) -> u64 {
    read_u64_from_storage(chain_state, &balance_access_path)
}

fn read_seqnum_from_storage(chain_state: &dyn ChainState, seqnum_access_path: &AccessPath) -> u64 {
    read_u64_from_storage(chain_state, &seqnum_access_path)
}

fn read_u64_from_storage(chain_state: &dyn ChainState, access_path: &AccessPath) -> u64 {
    chain_state
        .get(&access_path)
        .expect("Failed to query storage.")
        .map_or(0, |bytes| decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> u64 {
    let mut buf = [0; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

fn balance_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"balance".to_vec())
}

fn seqnum_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"seqnum".to_vec())
}

fn gen_mint_writeset(sender: AccountAddress, balance: u64, seqnum: u64) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(seqnum.to_le_bytes().to_vec()),
    ));
    write_set.freeze().expect("mint writeset should be valid")
}

fn gen_payment_writeset(
    sender: AccountAddress,
    sender_balance: u64,
    sender_seqnum: u64,
    recipient: AccountAddress,
    recipient_balance: u64,
) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(sender_balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(sender_seqnum.to_le_bytes().to_vec()),
    ));
    write_set.push((
        balance_ap(recipient),
        WriteOp::Value(recipient_balance.to_le_bytes().to_vec()),
    ));
    write_set
        .freeze()
        .expect("payment write set should be valid")
}

pub fn encode_mint_program(amount: u64) -> Script {
    let argument = TransactionArgument::U64(amount);
    Script::new(vec![], vec![argument])
}

pub fn encode_transfer_program(recipient: AccountAddress, amount: u64) -> Script {
    let argument1 = TransactionArgument::Address(recipient);
    let argument2 = TransactionArgument::U64(amount);
    Script::new(vec![], vec![argument1, argument2])
}

pub fn encode_mint_transaction(sender: AccountAddress, amount: u64) -> Transaction {
    encode_transaction(sender, encode_mint_program(amount))
}

pub fn encode_transfer_transaction(
    sender: AccountAddress,
    recipient: AccountAddress,
    amount: u64,
) -> Transaction {
    encode_transaction(sender, encode_transfer_program(recipient, amount))
}

fn encode_transaction(sender: AccountAddress, program: Script) -> Transaction {
    let raw_transaction =
        RawUserTransaction::new_script(sender, 0, program, 0, 0, std::time::Duration::from_secs(0));

    let (privkey, pubkey) = compat::generate_keypair(None);
    Transaction::UserTransaction(
        raw_transaction
            .sign(&privkey, pubkey)
            .expect("Failed to sign raw transaction.")
            .into_inner(),
    )
}

fn decode_transaction(txn: &SignedUserTransaction) -> MockTransaction {
    let sender = txn.sender();
    match txn.payload() {
        TransactionPayload::Script(script) => {
            assert!(script.code().is_empty(), "Code should be empty.");
            match script.args().len() {
                1 => match script.args()[0] {
                    TransactionArgument::U64(amount) => MockTransaction::Mint { sender, amount },
                    _ => unimplemented!(
                        "Only one integer argument is allowed for mint transactions."
                    ),
                },
                2 => match (&script.args()[0], &script.args()[1]) {
                    (TransactionArgument::Address(recipient), TransactionArgument::U64(amount)) => {
                        MockTransaction::Payment {
                            sender,
                            recipient: *recipient,
                            amount: *amount,
                        }
                    }
                    _ => unimplemented!(
                        "The first argument for payment transaction must be recipient address \
                         and the second argument must be amount."
                    ),
                },
                _ => unimplemented!("Transaction must have one or two arguments."),
            }
        }
        TransactionPayload::Module(_) => {
            unimplemented!("MockExecutor does not support Module transaction payload.")
        }
    }
}

pub fn get_signed_txn(
    sender: AccountAddress,
    sequence_number: u64,
    private_key: &Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    script: Script,
) -> SignedUserTransaction {
    let expiration_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 10; // 10 seconds from now.
    let raw_txn = RawUserTransaction::new_script(
        sender,
        sequence_number,
        script,
        MOCK_GAS_AMOUNT,
        MOCK_GAS_PRICE,
        Duration::from_secs(expiration_time),
    );

    let signature = private_key.sign_message(&raw_txn.crypto_hash());

    SignedUserTransaction::new(raw_txn, public_key, signature)
}
