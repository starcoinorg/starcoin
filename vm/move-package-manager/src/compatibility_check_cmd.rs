// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::release::module;
use clap::Parser;
use itertools::Itertools;
use move_binary_format::CompiledModule;
use move_cli::sandbox::utils::PackageContext;
use move_cli::Move;
use move_core_types::resolver::ModuleResolver;
use starcoin_config::BuiltinNetworkID;
use starcoin_move_compiler::check_compiled_module_compat;
use starcoin_transactional_test_harness::remote_state::RemoteStateView;
use std::collections::BTreeMap;
use stdlib::{load_compiled_modules, load_latest_stable_compiled_modules, StdlibVersion};

#[derive(Parser)]
pub struct CompatibilityCheckCommand {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: Option<String>,
    #[clap(long = "block-number", requires("rpc"))]
    /// block number to read state from. default to latest block number.
    block_number: Option<u64>,

    #[clap(long = "network", short, conflicts_with("rpc"))]
    /// genesis with the network
    network: Option<BuiltinNetworkID>,

    #[clap(long = "pre-version", short)]
    /// use to check pre-version compatibility.
    pre_version: Option<u64>,
}

pub fn handle_compatibility_check(
    move_args: &Move,
    cmd: CompatibilityCheckCommand,
) -> anyhow::Result<()> {
    let pkg_ctx = PackageContext::new(&move_args.package_path, &move_args.build_config)?;
    let pkg = pkg_ctx.package();

    let rpc = cmd.rpc.unwrap_or_else(|| {
        format!(
            "http://{}:{}",
            cmd.network
                .unwrap_or(BuiltinNetworkID::Main)
                .boot_nodes_domain(),
            9850
        )
    });

    let remote_view = RemoteStateView::from_url(&rpc, cmd.block_number)?;

    let mut incompatible_module_ids = vec![];
    for m in pkg.modules()? {
        let m = module(&m.unit)?;
        let old_module = remote_view
            .get_module(&m.self_id())
            .map_err(|e| e.into_vm_status())?;
        if let Some(old) = old_module {
            let old_module = CompiledModule::deserialize(&old)?;
            let compatibility = check_compiled_module_compat(&old_module, m);
            if !compatibility.is_fully_compatible() {
                incompatible_module_ids.push((m.self_id(), compatibility));
            }
        }
    }

    if !incompatible_module_ids.is_empty() {
        eprintln!(
            "Modules {} is incompatible with remote chain: {}!",
            incompatible_module_ids
                .into_iter()
                .map(|(module_id, compat)| format!(
                    "{}(struct_layout:{},struct_and_function_linking:{})",
                    module_id, compat.struct_layout, compat.struct_and_function_linking
                ))
                .join(","),
            &rpc
        );
    } else {
        eprintln!(
            "All modules in {} is full compatible with remote chain: {}!",
            pkg.compiled_package_info.package_name, &rpc
        );
    }

    println!("pre_version number: {}", cmd.pre_version.unwrap());
    handle_pre_version_compatibility_check(cmd.pre_version);
    Ok(())
}

fn handle_pre_version_compatibility_check(pre_version: Option<u64>) {
    // referece: vm/stdlib/src/main.rs line 302, test later.
    let sources = &stdlib::STARCOIN_FRAMEWORK_SOURCES;
    let new_modules = stdlib::build_stdlib(&sources.files);

    if let Some((pre_stable_version, pre_stable_modules)) = pre_version
        .map(StdlibVersion::Version)
        .map(|v| (v, load_compiled_modules(v)))
        .or_else(load_latest_stable_compiled_modules)
    {
        println!(
            "Check compat with pre stable version: {}",
            pre_stable_version
        );
        let pre_stable_modules = pre_stable_modules
            .into_iter()
            .map(|module| (module.self_id(), module))
            .collect::<BTreeMap<_, _>>();
        let incompatible_module_ids = new_modules
            .values()
            .into_iter()
            .filter_map(|module| {
                let module_id = module.self_id();
                if let Some(old_module) = pre_stable_modules.get(&module_id) {
                    let compatibility =
                        check_compiled_module_compat(old_module, module).is_fully_compatible();
                    if !compatibility {
                        Some(module_id)
                    } else {
                        None
                    }
                } else {
                    println!("Module {:?} is new module.", module_id);
                    None
                }
            })
            .collect::<Vec<_>>();
        if !incompatible_module_ids.is_empty() {
            eprintln!(
                "Modules {} is incompatible with version: {}!",
                incompatible_module_ids
                    .into_iter()
                    .map(|module_id| module_id.to_string())
                    .join(","),
                pre_stable_version
            );
            std::process::exit(1);
        }
    }
}
