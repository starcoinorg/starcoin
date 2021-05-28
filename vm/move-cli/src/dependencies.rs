use anyhow::Result;
use itertools::Itertools;
use move_lang::parser::ast::{Definition, ModuleDefinition, ModuleMember, Use};
use move_lang::shared::Address;
pub fn get_uses(move_files: &[String]) -> Result<Vec<(Address, String)>> {
    fn get_module_uses(m: &ModuleDefinition) -> Vec<(Address, String)> {
        m.members
            .iter()
            .filter_map(|m| {
                if let ModuleMember::Use(u) = m {
                    Some(match u {
                        Use::Module(mi, _) => mi.value.clone(),
                        Use::Members(mi, _) => mi.value.clone(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    let (files, parsed) = move_lang::move_parse(move_files, &[], None, None)?;

    let (_, _address, program) = move_lang::unwrap_or_report_errors!(files, parsed);

    let used_deps = program
        .source_definitions
        .into_iter()
        .flat_map(|d| match d {
            Definition::Module(m) => get_module_uses(&m),
            Definition::Address(_, _, ms) => ms.iter().flat_map(|m| get_module_uses(m)).collect(),
            Definition::Script(s) => s
                .uses
                .iter()
                .map(|u| match u {
                    Use::Module(mi, _) => mi.value.clone(),
                    Use::Members(mi, _) => mi.value.clone(),
                })
                .collect(),
        })
        .unique()
        .collect();

    Ok(used_deps)
}
