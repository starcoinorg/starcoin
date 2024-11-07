// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::release::module;
use anyhow::{ensure, Ok};
use clap::Parser;
use itertools::Itertools;
use move_binary_format::CompiledModule;
use move_cli::Move;
use move_core_types::resolver::ModuleResolver;
use move_package::compilation::compiled_package::CompiledUnitWithSource;
use starcoin_cmd::dev::dev_helper::{self};
use starcoin_config::BuiltinNetworkID;
use starcoin_move_compiler::check_compiled_module_compat;
use starcoin_transactional_test_harness::remote_state::RemoteViewer;
use starcoin_types::transaction::Package;
use std::{collections::BTreeMap, path::PathBuf};

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

    #[clap(long = "pre-modules")]
    /// use to check pre modules compatibility.
    pre_modules: Option<PathBuf>,
}

pub fn handle_compatibility_check(
    move_args: &Move,
    cmd: CompatibilityCheckCommand,
) -> anyhow::Result<()> {
    let package_path = match move_args.package_path {
        Some(_) => move_args.package_path.clone(),
        None => Some(std::env::current_dir()?),
    };
    let pkg = move_args
        .build_config
        .clone()
        .compile_package(package_path.as_ref().unwrap(), &mut std::io::stdout())?;

    let rpc = cmd.rpc.unwrap_or_else(|| {
        format!(
            "http://{}:{}",
            cmd.network
                .unwrap_or(BuiltinNetworkID::Main)
                .boot_nodes_domain(),
            9850
        )
    });

    let remote_view = RemoteViewer::from_url(&rpc, cmd.block_number)?;

    let mut incompatible_module_ids = vec![];
    for m in pkg.root_compiled_units.as_slice() {
        let m = module(&m.unit)?;
        let old_module = remote_view
            .get_module(&m.self_id())
            .map_err(|e| e.into_vm_status())?;
        if let Some(old) = old_module {
            let old_module = CompiledModule::deserialize(&old)?;
            let compatibility = check_compiled_module_compat(&old_module, m);
            if compatibility.is_err() {
                incompatible_module_ids.push((m.self_id(), compatibility));
            }
        }
    }

    if !incompatible_module_ids.is_empty() {
        eprintln!(
            "Modules {} is incompatible with remote chain: {}!",
            incompatible_module_ids
                .into_iter()
                // XXX FIXME YSG
                .map(|(module_id, compat)| format!("{} {}", module_id, compat.is_err()))
                .join(","),
            &rpc
        );
    } else {
        eprintln!(
            "All modules in {} is full compatible with remote chain: {}!",
            pkg.compiled_package_info.package_name, &rpc
        );
    }

    if cmd.pre_modules.is_none() || !cmd.pre_modules.clone().unwrap().as_path().exists() {
        return Ok(());
    }

    handle_pre_version_compatibility_check(
        cmd.pre_modules.unwrap(),
        pkg.all_modules().collect_vec(),
    )?;
    Ok(())
}

fn handle_pre_version_compatibility_check(
    pre_modules: PathBuf,
    new_modules: Vec<&CompiledUnitWithSource>,
) -> anyhow::Result<()> {
    ensure!(
        pre_modules.as_path().exists(),
        "pre modules path: {} not exists",
        pre_modules.as_path().to_str().unwrap()
    );

    let mut pre_stable_modules = vec![];
    let pkg: Package = if pre_modules.as_path().is_dir() {
        dev_helper::load_package_from_dir(pre_modules.as_path())?
    } else {
        dev_helper::load_package_from_file(pre_modules.as_path())?
    };

    for module in pkg.modules() {
        let pre_stable_module = CompiledModule::deserialize(module.code())?;
        pre_stable_modules.push(pre_stable_module);
    }

    let pre_stable_modules = pre_stable_modules
        .into_iter()
        .map(|module| (module.self_id(), module))
        .collect::<BTreeMap<_, _>>();

    let incompatible_module_ids = new_modules
        .into_iter()
        .filter_map(|m| {
            let new_module = module(&m.unit).unwrap();
            let module_id = new_module.self_id();
            if let Some(old_module) = pre_stable_modules.get(&module_id) {
                let compatibility = check_compiled_module_compat(old_module, new_module);
                if compatibility.is_err() {
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
            "Modules {} is incompatible with previous version: {}!",
            incompatible_module_ids
                .into_iter()
                .map(|module_id| module_id.to_string())
                .join(","),
            pre_modules.to_str().unwrap()
        );
        std::process::exit(1);
    } else {
        eprintln!("All previous modules is full compatible with current modules!",);
    }

    Ok(())
}
