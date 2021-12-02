use anyhow::Result;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_executor::execute_readonly_function;
use starcoin_state_api::{ChainStateReader, ChainStateWriter};
use starcoin_types::account_config::account_struct_tag;
use starcoin_types::identifier::Identifier;
use starcoin_types::language_storage::ModuleId;
use starcoin_vm_types::access_path::AccessPath;
use starcoin_vm_types::account_config::association_address;
use starcoin_vm_types::transaction::{Package, ScriptFunction, TransactionPayload};
use starcoin_vm_types::value::MoveValue;
use test_helper::executor::{
    association_execute_should_success, compile_modules_with_address, prepare_genesis,
};
#[stest::test]
fn test_starcoin_merkle() -> Result<()> {
    let (chain_state, net) = prepare_genesis();
    let ap = AccessPath::resource_access_path(association_address(), account_struct_tag());
    let state_with_proof = chain_state.get_with_proof(&ap)?;
    state_with_proof.proof.verify(
        chain_state.state_root(),
        ap,
        state_with_proof.state.as_deref(),
    )?;

    {
        let source = include_str!("../modules/StarcoinVerifier.move");
        let modules = compile_modules_with_address(association_address(), source);

        let package = Package::new(modules, None)?;
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::Package(package),
        )?;
        chain_state.commit()?;
        chain_state.flush()?;
    }

    let state_root = chain_state.state_root();

    {
        let script_function = ScriptFunction::new(
            ModuleId::new(
                association_address(),
                Identifier::new("StarcoinVerifierScripts").unwrap(),
            ),
            Identifier::new("create").unwrap(),
            vec![],
            vec![MoveValue::vector_u8(state_root.to_vec())
                .simple_serialize()
                .unwrap()],
        );
        association_execute_should_success(
            &net,
            &chain_state,
            TransactionPayload::ScriptFunction(script_function),
        )?;
        chain_state.commit()?;
        chain_state.flush()?;
    }

    {
        // change to previout state root.
        let old_chain_state = chain_state.fork_at(state_root);
        // let state_root = chain_state.state_root();
        let _expected_root = MoveValue::vector_u8(state_root.to_vec());

        let ap = AccessPath::resource_access_path(association_address(), account_struct_tag());
        let state_with_proof = old_chain_state.get_with_proof(&ap)?;
        let account_address = MoveValue::vector_u8(association_address().to_vec());
        let account_state_hash = MoveValue::vector_u8(
            state_with_proof
                .proof
                .account_state
                .unwrap()
                .crypto_hash()
                .to_vec(),
        );
        let proof = MoveValue::Vector(
            state_with_proof
                .proof
                .account_proof
                .siblings()
                .iter()
                .map(|h| MoveValue::vector_u8(h.to_vec()))
                .collect(),
        );

        let result = execute_readonly_function(
            &chain_state,
            &ModuleId::new(
                association_address(),
                Identifier::new("StarcoinVerifier").unwrap(),
            ),
            &Identifier::new("verify_on").unwrap(),
            vec![],
            vec![
                MoveValue::Address(association_address())
                    .simple_serialize()
                    .unwrap(),
                account_address.simple_serialize().unwrap(),
                account_state_hash.simple_serialize().unwrap(),
                proof.simple_serialize().unwrap(),
            ],
            None,
        )?
        .pop()
        .unwrap();

        let is_ok: bool = bcs_ext::from_bytes(result.as_slice()).unwrap();
        assert!(is_ok, "verify fail");
    }
    Ok(())
}
