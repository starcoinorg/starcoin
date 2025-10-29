use move_vm2_core_types::account_address::AccountAddress;
use move_vm2_core_types::identifier::Identifier;
use move_vm2_core_types::language_storage::{StructTag, CORE_CODE_ADDRESS};
use move_vm2_core_types::move_resource::MoveStructType;
use starcoin_vm2_test_helper::executor::prepare_genesis;
use starcoin_vm2_vm_types::account_config::ObjectGroupResource;
use starcoin_vm2_vm_types::state_store::state_key::StateKey;
use starcoin_vm2_vm_types::state_view::StateReaderExt;

#[stest::test]
pub fn test_read_resource_group() -> anyhow::Result<()> {
    let (chain_state, _net) = prepare_genesis()?;
    let txn_fee_bytes = chain_state.get_resource_group_struct_tag_bytes(
        &AccountAddress::ONE,
        &StateKey::resource_group(&AccountAddress::ONE, &ObjectGroupResource::struct_tag()),
        &StructTag {
            address: CORE_CODE_ADDRESS,
            module: Identifier::new("transaction_fee")?,
            name: Identifier::new("TransactionFeePod")?,
            type_args: vec![],
        },
    )?;
    assert!(txn_fee_bytes.is_some(), "should be present");
    Ok(())
}
