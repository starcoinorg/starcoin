use anyhow::{bail, Result};
use starcoin_config::ChainNetwork;
use starcoin_vm2_executor::executor2::{do_execute_block_transactions, execute_readonly_function};
use starcoin_vm2_genesis::{build_genesis_transaction, execute_genesis_transaction};
use starcoin_vm2_state_api::AccountStateReader;
use starcoin_vm2_statedb::{ChainStateDB, ChainStateReader, ChainStateWriter};
use starcoin_vm2_transaction_builder::DEFAULT_MAX_GAS_AMOUNT;
use starcoin_vm2_types::{
    account::Account,
    account_address::AccountAddress,
    account_config::{association_address, genesis_address},
    block_metadata::BlockMetadata,
    contract_event::ContractEvent,
    identifier::Identifier,
    language_storage::ModuleId,
    transaction::{
        authenticator::AccountPrivateKey, Module, RawUserTransaction, SignedUserTransaction,
        Transaction, TransactionOutput, TransactionPayload, TransactionStatus,
    },
};
use starcoin_vm2_vm_types::{
    move_resource::MoveResource, state_view::StateReaderExt, vm_status::KeptVMStatus, StateView,
};

//TODO warp to A MockTxnExecutor

pub const TEST_MODULE: &str = r#"
    module {{sender}}::M {
        struct Foo { a: u8 }
        public fun foo(): u8 { 1 }
    }
    "#;
pub const TEST_MODULE_1: &str = r#"
    module {{sender}}::M {
        struct Foo { a: address }
        public fun foo(): u8 { 2 }
    }
    "#;
pub const TEST_MODULE_2: &str = r#"
    module {{sender}}::M {
        struct Foo { a: u8 }
        public fun foo(): u8 { 1 }
        public fun bar(): u8 { 2 }
    }
    "#;

pub fn prepare_genesis() -> anyhow::Result<(ChainStateDB, ChainNetwork)> {
    let net = ChainNetwork::new_test();
    let chain_state = ChainStateDB::mock();
    let genesis_txn = build_genesis_transaction(&net).unwrap();
    // execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))?;
    Ok((chain_state, net))
}

pub fn prepare_customized_genesis(net: &ChainNetwork) -> Result<ChainStateDB> {
    let chain_state = ChainStateDB::mock();
    let genesis_txn = build_genesis_transaction(net).unwrap();
    // execute_genesis_txn(&chain_state, genesis_txn).unwrap();
    execute_genesis_transaction(&chain_state, Transaction::UserTransaction(genesis_txn))?;
    Ok(chain_state)
}

pub fn execute_and_apply(chain_state: &ChainStateDB, txn: Transaction) -> TransactionOutput {
    let output = do_execute_block_transactions(chain_state, vec![txn], None, None)
        .unwrap()
        .pop()
        .expect("Output must exist.");
    if let TransactionStatus::Keep(_) = output.status() {
        chain_state
            .apply_write_set(output.write_set().clone())
            .expect("apply write_set should success.");
        chain_state.commit().expect("commit should success.");
    }

    output
}

pub fn current_block_number<S: StateView>(state_view: &S) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("stc_block").unwrap()),
        &Identifier::new("get_current_block_number").unwrap(),
        vec![],
        vec![],
        None,
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    bcs_ext::from_bytes(ret.pop().unwrap().as_slice()).unwrap()
}

pub fn get_sequence_number<S: ChainStateReader>(addr: AccountAddress, chain_state: &S) -> u64 {
    // if account not exist, return 0
    chain_state
        .get_account_resource(addr)
        .map(|r| r.sequence_number())
        .unwrap_or_default()
}

pub fn get_balance<S: ChainStateReader>(address: AccountAddress, chain_state: &S) -> u128 {
    AccountStateReader::new(chain_state)
        .get_balance(&address)
        .expect("read balance resource should ok")
}

pub fn compile_modules_with_address(address: AccountAddress, code: &str) -> Vec<Module> {
    compile_modules_with_address_ext(address, code, &stdlib_files())
}

fn stdlib_files() -> Vec<String> {
    starcoin_vm2_framework::testnet_release_bundle()
        .files()
        .unwrap()
        .clone()
}

pub fn compile_modules_with_address_ext(
    address: AccountAddress,
    code: &str,
    libs: &[String],
) -> Vec<Module> {
    let (_, compiled_result) =
        starcoin_vm2_move_compiler::compile_source_string(code, libs, address)
            .expect("compile fail");

    compiled_result
        .into_iter()
        .map(|m| Module::new(m.serialize(None)))
        .collect()
}

pub fn compile_script(code: impl AsRef<str>) -> Result<Vec<u8>> {
    let mut compile_unit = starcoin_vm2_move_compiler::compile_source_string_no_report(
        code.as_ref(),
        &stdlib_files(),
        genesis_address(),
    )?
    .1
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(compile_unit
        .0
        .pop()
        .expect("at least contain one script")
        .into_compiled_unit()
        .serialize(None))
}

pub fn compile_ir_script(_code: impl AsRef<str>) -> Result<Vec<u8>> {
    // TODO(BobOng): [dual-vm] genesis for stdlib
    // use starcoin_vm2_move_ir_compiler::Compiler as IRCompiler;
    // let modules = starcoin_vm2_move_compiler::stdlib_compiled_modules(
    //     starcoin_vm2_transaction_builder::StdLibOptions::Compiled(StdlibVersion::Latest),
    // );
    // let (script, _) = IRCompiler::new(modules.iter().collect())
    //     .into_compiled_script_and_source_map(code.as_ref())?;
    // let mut bytes = vec![];
    // script.serialize(&mut bytes)?;
    let bytes = Vec::new();
    Ok(bytes)
}

pub fn association_execute(
    net: &ChainNetwork,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_raw_txn(association_address(), state, payload, None);
    let txn = net
        .genesis_config2()
        .as_ref()
        .unwrap()
        .sign_with_association(txn)?;
    execute_signed_txn(state, txn)
}

pub fn association_execute_should_success(
    net: &ChainNetwork,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_raw_txn(association_address(), state, payload, None);
    let txn = net
        .genesis_config2()
        .as_ref()
        .unwrap()
        .sign_with_association(txn)?;
    execute_signed_txn_should_success(state, txn)
}

pub fn account_execute(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    user_execute(*account.address(), account.private_key(), state, payload)
}

pub fn account_execute_should_success(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    user_execute_should_success(*account.address(), account.private_key(), state, payload)
}

pub fn account_execute_with_output(
    account: &Account,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> TransactionOutput {
    let txn = build_signed_txn(*account.address(), account.private_key(), state, payload);
    execute_and_apply(state, Transaction::UserTransaction(txn))
}

pub fn blockmeta_execute(state: &ChainStateDB, meta: BlockMetadata) -> Result<TransactionOutput> {
    let txn = Transaction::BlockMetadata(meta);
    let output = execute_and_apply(state, txn);
    if let TransactionStatus::Discard(s) = output.status() {
        bail!("txn discard, status: {:?}", s);
    }

    Ok(output)
}

pub fn build_raw_txn(
    user_address: AccountAddress,
    state: &ChainStateDB,
    payload: TransactionPayload,
    expiration_timestamp_secs: Option<u64>,
) -> RawUserTransaction {
    let chain_id = state.get_chain_id().unwrap();
    let seq_number = get_sequence_number(user_address, state);

    let now_seconds: u64 = state.get_timestamp().unwrap().microseconds / 1000000;
    let expiration_timestamp_secs = expiration_timestamp_secs.unwrap_or(now_seconds + 60 * 60);
    RawUserTransaction::new_with_default_gas_token(
        user_address,
        seq_number,
        payload,
        DEFAULT_MAX_GAS_AMOUNT,
        1,
        expiration_timestamp_secs,
        chain_id,
    )
}

fn user_execute(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_signed_txn(user_address, prikey, state, payload);
    execute_signed_txn(state, txn)
}

fn user_execute_should_success(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> Result<TransactionOutput> {
    let txn = build_signed_txn(user_address, prikey, state, payload);
    execute_signed_txn_should_success(state, txn)
}

fn build_signed_txn(
    user_address: AccountAddress,
    prikey: &AccountPrivateKey,
    state: &ChainStateDB,
    payload: TransactionPayload,
) -> SignedUserTransaction {
    let txn = build_raw_txn(user_address, state, payload, None);
    // It's ok to unwrap here, we just build the txn, and this function is only used for testing purpose.
    let signature = prikey.sign(&txn).unwrap();
    SignedUserTransaction::new(txn, signature)
}

#[allow(clippy::unnecessary_wraps)]
fn execute_signed_txn(
    state: &ChainStateDB,
    txn: SignedUserTransaction,
) -> Result<TransactionOutput> {
    let txn = Transaction::UserTransaction(txn);
    Ok(execute_and_apply(state, txn))
}

fn execute_signed_txn_should_success(
    state: &ChainStateDB,
    txn: SignedUserTransaction,
) -> Result<TransactionOutput> {
    let txn = Transaction::UserTransaction(txn);
    let output = execute_and_apply(state, txn);

    match output.status() {
        TransactionStatus::Discard(s) => {
            bail!("txn discard, status: {:?}", s);
        }
        TransactionStatus::Keep(s) => {
            if s != &KeptVMStatus::Executed {
                bail!("txn executing error, {:?}", s)
            }
        }
        TransactionStatus::Retry => {
            bail!("impossible txn retry");
        }
    }
    Ok(output)
}

pub fn move_abort_code(status: KeptVMStatus) -> Option<u64> {
    match status {
        KeptVMStatus::MoveAbort(_, code) => Some(code),
        _ => None,
    }
}

pub fn expect_event<Event: MoveResource>(output: &TransactionOutput) -> ContractEvent {
    output
        .events()
        .iter()
        .filter(|event| event.is_typed::<Event>())
        .last()
        .cloned()
        .unwrap_or_else(|| panic!("Expect event: {}", Event::struct_tag()))
}

pub fn expect_decode_event<Event: MoveResource>(output: &TransactionOutput) -> Event {
    output
        .events()
        .iter()
        .filter(|event| event.is_typed::<Event>())
        .last()
        .cloned()
        .and_then(|event| event.decode_event::<Event>().ok())
        .unwrap_or_else(|| panic!("Expect event: {}", Event::struct_tag()))
}
