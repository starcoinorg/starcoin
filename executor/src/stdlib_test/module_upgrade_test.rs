use crate::execute_readonly_function;
use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_state_api::StateView;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::{ModuleId, StructTag, TypeTag};
use starcoin_types::transaction::Script;
use starcoin_vm_types::account_config::{genesis_address, stc_type_tag};
use starcoin_vm_types::transaction::{Package, TransactionPayload};
use starcoin_vm_types::transaction_argument::TransactionArgument;
use starcoin_vm_types::values::VMValueCast;
use stdlib::transaction_scripts::{compiled_transaction_script, StdlibScript};
use test_helper::dao::dao_vote_test;
use test_helper::executor::*;
use test_helper::Account;

#[stest::test]
fn test_dao_upgrade_module() -> Result<()> {
    let alice = Account::new();
    let (chain_state, net) = prepare_genesis();

    let dao_action_type_tag = TypeTag::Struct(StructTag {
        address: genesis_address(),
        module: Identifier::new("UpgradeModuleDaoProposal").unwrap(),
        name: Identifier::new("UpgradeModule").unwrap(),
        type_params: vec![],
    });
    let module = compile_module_with_address(genesis_address(), TEST_MODULE);
    let package = Package::new_with_module(module)?;
    let package_hash = package.crypto_hash();
    let script1 =
        compiled_transaction_script(net.stdlib_version(), StdlibScript::ProposeModuleUpgrade)
            .into_vec();

    let vote_script = Script::new(
        script1,
        vec![stc_type_tag()],
        vec![
            TransactionArgument::Address(genesis_address()),
            TransactionArgument::U8Vector(package_hash.to_vec()),
            TransactionArgument::U64(0),
        ],
    );
    let script2 =
        compiled_transaction_script(net.stdlib_version(), StdlibScript::SubmitModuleUpgradePlan)
            .into_vec();
    let execute_script = Script::new(
        script2,
        vec![stc_type_tag()],
        vec![
            TransactionArgument::Address(*alice.address()),
            TransactionArgument::U64(0),
        ],
    );
    let chain_state = dao_vote_test(
        alice,
        chain_state,
        net.clone(),
        vote_script,
        dao_action_type_tag,
        execute_script,
    )?;
    association_execute(
        net.genesis_config(),
        &chain_state,
        TransactionPayload::Package(package),
    )?;

    assert_eq!(read_foo(&chain_state), 1);
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
