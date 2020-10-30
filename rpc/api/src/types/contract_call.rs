pub use starcoin_resource_viewer::AnnotatedMoveValue;

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

#[cfg(test)]
mod test {
    use crate::types::ContractCall;
    #[test]
    fn test_deserialize() {
        let s = r#"
{
  "module_address": "0CC02653F9D7A62D07754D859B066BDE",
  "module_name": "T",
  "func": "A",
  "type_args": [
    {
      "Struct": {
        "address": "42C4DDA17CC39AF459C20D09F6A82EDF",
        "module": "T",
        "name": "T",
        "type_params": []
      }
    }
  ],
  "args": [
    {
      "Address": "D6F8FAF8FA976104B8BA8C6F85DCF9E4"
    }
  ]
}        
        "#;
        let v = serde_json::from_str::<ContractCall>(s).unwrap();
        println!("{:?}", v);
    }
}
