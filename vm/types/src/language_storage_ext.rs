use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;

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
