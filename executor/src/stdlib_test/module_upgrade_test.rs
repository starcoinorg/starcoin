use crate::execute_readonly_function;
use crate::test_helper::*;
use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_crypto::HashValue;
use starcoin_functional_tests::account::Account;
use starcoin_state_api::StateView;
use starcoin_transaction_builder::encode_create_account_script;
use starcoin_types::account_address::AccountAddress;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::Script;
use starcoin_vm_types::account_config::genesis_address;
use starcoin_vm_types::block_metadata::BlockMetadata;
use starcoin_vm_types::token::stc::stc_type_tag;
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::transaction_argument::TransactionArgument;
use starcoin_vm_types::values::{VMValueCast, Value};
use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};

const PENDING: u8 = 1;
const ACTIVE: u8 = 2;
#[allow(unused)]
const DEFEATED: u8 = 3;
const AGREED: u8 = 4;
const QUEUED: u8 = 5;
const EXECUTABLE: u8 = 6;
const EXTRACTED: u8 = 7;
fn proposal_state(
    state_view: &dyn StateView,
    token: TypeTag,
    action_ty: TypeTag,
    proposer_address: AccountAddress,
    proposal_id: u64,
) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("proposal_state").unwrap(),
        vec![token, action_ty],
        vec![Value::address(proposer_address), Value::u64(proposal_id)],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

fn quorum_vote(state_view: &dyn StateView, token: TypeTag) -> u128 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("quorum_votes").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

fn voting_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_delay").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}
fn voting_period(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("voting_period").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

fn min_action_delay(state_view: &dyn StateView, token: TypeTag) -> u64 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("Dao").unwrap()),
        &Identifier::new("min_action_delay").unwrap(),
        vec![token],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}

#[stest::test]
fn test_dao_upgrade_module() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let alice = Account::new();
    let bob = Account::new();
    let pre_mint_amount = net.genesis_config().pre_mine_amount;
    let one_day: u64 = 60 * 60 * 24 * 1000;
    // Block 1
    let block_number = 1;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                genesis_address(),
                None,
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let script = encode_create_account_script(
            net.stdlib_version(),
            stc_type_tag(),
            alice.address(),
            alice.pubkey.to_bytes().to_vec(),
            pre_mint_amount / 4,
        );
        association_execute(
            net.genesis_config(),
            &chain_state,
            TransactionPayload::Script(script),
        )?;

        let script = encode_create_account_script(
            net.stdlib_version(),
            stc_type_tag(),
            bob.address(),
            bob.pubkey.to_bytes().to_vec(),
            pre_mint_amount / 4,
        );
        association_execute(
            net.genesis_config(),
            &chain_state,
            TransactionPayload::Script(script),
        )?;
    }

    let module = compile_module_with_address(genesis_address(), TEST_MODULE);
    let package = Package::new_with_module(module)?;
    let package_hash = package.crypto_hash();

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });
    // block 2
    let block_number = 2;
    let block_timestamp = net.time_service().now_millis() + one_day * block_number;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;

        let script =
            compiled_transaction_script(net.stdlib_version(), StdlibScript::ProposeModuleUpgrade)
                .into_vec();

        let script = Script::new(
            script,
            vec![stc_type_tag()],
            vec![
                TransactionArgument::Address(genesis_address()),
                TransactionArgument::U8Vector(package_hash.to_vec()),
                TransactionArgument::U64(0),
            ],
        );
        account_execute(&alice, &chain_state, TransactionPayload::Script(script))?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, PENDING);
    }

    // block 3
    let block_number = 3;
    let block_timestamp =
        block_timestamp + voting_delay(&chain_state, stc_type_tag()) * 1000 + 10000;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let script =
            compiled_transaction_script(net.stdlib_version(), StdlibScript::CastVote).into_vec();
        let proposer_address = *alice.address();
        let proposer_id = 0;
        let voting_power = get_balance(*bob.address(), &chain_state);
        println!("alice voting power: {}", voting_power);
        let script = Script::new(
            script,
            vec![stc_type_tag(), dao_action_type_tag.clone()],
            vec![
                TransactionArgument::Address(proposer_address),
                TransactionArgument::U64(proposer_id),
                TransactionArgument::Bool(true),
                TransactionArgument::U128(voting_power),
            ],
        );
        // vote first.
        account_execute(&alice, &chain_state, TransactionPayload::Script(script))?;

        let quorum = quorum_vote(&chain_state, stc_type_tag());
        println!("quorum: {}", quorum);

        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, ACTIVE);
    }

    // block 4
    let block_number = 4;
    let block_timestamp =
        block_timestamp + voting_period(&chain_state, stc_type_tag()) * 1000 - 10 * 1000;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, ACTIVE);
    }

    // block 5
    let block_number = 5;
    let block_timestamp = block_timestamp + 20 * 1000;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, AGREED);

        let script =
            compiled_transaction_script(net.stdlib_version(), StdlibScript::QueueProposalAction)
                .into_vec();
        let script = Script::new(
            script,
            vec![stc_type_tag(), dao_action_type_tag.clone()],
            vec![
                TransactionArgument::Address(*alice.address()),
                TransactionArgument::U64(0),
            ],
        );
        account_execute(&alice, &chain_state, TransactionPayload::Script(script))?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, QUEUED);
    }

    // block 6
    let block_number = 6;
    let block_timestamp = block_timestamp + min_action_delay(&chain_state, stc_type_tag()) * 1000;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag.clone(),
            *alice.address(),
            0,
        );
        assert_eq!(state, EXECUTABLE);

        let script = compiled_transaction_script(
            net.stdlib_version(),
            StdlibScript::SubmitModuleUpgradePlan,
        )
        .into_vec();
        let script = Script::new(
            script,
            vec![stc_type_tag()],
            vec![
                TransactionArgument::Address(*alice.address()),
                TransactionArgument::U64(0),
            ],
        );
        account_execute(&alice, &chain_state, TransactionPayload::Script(script))?;
    }

    // block 7
    let block_number = 7;
    let block_timestamp = block_timestamp + 1000;
    {
        blockmeta_execute(
            &chain_state,
            BlockMetadata::new(
                HashValue::zero(),
                block_timestamp,
                *alice.address(),
                Some(alice.pubkey.clone()),
                0,
                block_number,
                net.chain_id(),
                0,
            ),
        )?;
        let state = proposal_state(
            &chain_state,
            stc_type_tag(),
            dao_action_type_tag,
            *alice.address(),
            0,
        );
        assert_eq!(state, EXTRACTED);

        association_execute(
            net.genesis_config(),
            &chain_state,
            TransactionPayload::Package(package),
        )?;

        assert_eq!(read_foo(&chain_state), 1);
    }

    Ok(())
}

fn read_foo(state_view: &dyn StateView) -> u8 {
    let mut ret = execute_readonly_function(
        state_view,
        &ModuleId::new(genesis_address(), Identifier::new("M").unwrap()),
        &Identifier::new("foo").unwrap(),
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(ret.len(), 1);
    ret.pop().unwrap().1.cast().unwrap()
}
