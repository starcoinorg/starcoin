use serde::Deserialize;
use serde::Serialize;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::language_storage::TypeTag;
use starcoin_vm_types::transaction_argument::TransactionArgument;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContractCall {
    pub module_address: AccountAddress,
    pub module_name: String,
    pub func: String,
    pub type_args: Vec<TypeTag>,
    pub args: Vec<TransactionArgument>,
}
