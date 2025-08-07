use anyhow::Result;
use starcoin_config::genesis_config::G_LATEST_GAS_PARAMS;
use starcoin_framework::get_metadata_from_compiled_module;
use starcoin_gas_schedule::LATEST_GAS_FEATURE_VERSION;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::normalized::Function;
use std::collections::{HashMap, HashSet};
use stdlib::StdLibOptions;

#[test]
pub fn test_native_function_matches() -> Result<()> {
    let modules = stdlib::stdlib_compiled_modules(StdLibOptions::Fresh);
    let runtime_metadata = modules
        .iter()
        .filter_map(|m| {
            get_metadata_from_compiled_module(m).map(|metadata| (m.self_id(), metadata))
        })
        .collect::<HashMap<_, _>>();
    let native_functions: Vec<_> = modules
        .iter()
        .flat_map(|m| {
            m.function_defs()
                .iter()
                .filter_map(|func_def| {
                    let func_name = Function::new(m, func_def).0;
                    if func_def.is_native()
                        && !runtime_metadata
                            .get(&m.self_id())
                            .and_then(|metadata| {
                                metadata.fun_attributes.get(&func_name.to_string())
                            })
                            .map(|attr| attr.iter().any(|attr| attr.is_bytecode_instruction()))
                            .unwrap_or_default()
                    {
                        Some(func_name)
                    } else {
                        None
                    }
                })
                .map(|func_name| {
                    (
                        *m.self_id().address(),
                        m.self_id().name().to_string(),
                        func_name.to_string(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect();

    let mut native_function_table = starcoin_vm_runtime::natives::starcoin_natives(
        LATEST_GAS_FEATURE_VERSION,
        G_LATEST_GAS_PARAMS.clone().natives,
        G_LATEST_GAS_PARAMS.clone().vm.misc,
        starcoin_vm_types::on_chain_config::TimedFeaturesBuilder::enable_all().build(),
        starcoin_vm_types::on_chain_config::Features::default(),
    )
    .iter()
    .map(|(addr, m_name, f_name, _)| (*addr, m_name.to_string(), f_name.to_string()))
    .collect::<HashSet<_>>();

    for f in native_functions {
        assert!(
            native_function_table.remove(&f),
            "native {:?} not exists in native function table",
            &f
        );
    }

    for f in native_function_table {
        println!("native {:?} is un-used in latest stdlib", f)
    }

    Ok(())
}
