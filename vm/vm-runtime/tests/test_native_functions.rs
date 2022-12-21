use anyhow::Result;
use starcoin_vm_types::access::ModuleAccess;
use starcoin_vm_types::normalized::Function;
use std::collections::HashSet;
use stdlib::load_latest_compiled_modules;

#[test]
pub fn test_native_function_matches() -> Result<()> {
    let modules = load_latest_compiled_modules();
    let native_functions: Vec<_> = modules
        .iter()
        .flat_map(|m| {
            m.function_defs()
                .iter()
                .filter_map(|func_def| {
                    if func_def.is_native() {
                        Some(Function::new(m, func_def).0)
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

    let mut native_function_table = starcoin_vm_runtime::natives::starcoin_natives()
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
