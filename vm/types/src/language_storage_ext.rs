use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct FunctionId {
    pub module: ModuleId,
    pub function: Identifier,
}

impl std::fmt::Display for FunctionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", &self.module, &self.function)
    }
}

impl FromStr for FunctionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let splits: Vec<&str> = s.rsplitn(2, "::").collect();
        if splits.len() != 2 {
            anyhow::bail!("invalid script function id");
        }
        let module_id = parse_module_id(splits[1])?;
        let function = Identifier::new(splits[0])?;
        Ok(FunctionId {
            module: module_id,
            function,
        })
    }
}

pub fn parse_module_id(s: &str) -> Result<ModuleId, anyhow::Error> {
    let parts: Vec<_> = s.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("invalid module id");
    }
    let module_addr = parts[0].parse::<AccountAddress>()?;
    let module_name = Identifier::new(parts[1])?;
    Ok(ModuleId::new(module_addr, module_name))
}
