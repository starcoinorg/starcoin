use crate::COMPILED_MOVE_CODE_DIR;
use anyhow::{bail, ensure, format_err};
use log::info;
use starcoin_crypto::hash::PlainCryptoHash;
use starcoin_types::stdlib::StdlibVersion;
use starcoin_vm2_cached_packages::starcoin_stdlib::stdlib_upgrade_scripts_dummy_upgrade;
use starcoin_vm2_framework::ReleaseBundle;
use starcoin_vm2_vm_types::file_format::CompiledModule;
use starcoin_vm2_vm_types::transaction::{EntryFunction, Module, Package, TransactionPayload};
use std::collections::BTreeMap;
use std::path::Path;

/// read release bundles from dir.
pub fn read_released_bundles(stdlib_version: StdlibVersion) -> Vec<(String, Vec<Vec<u8>>)> {
    let sub_dir = stdlib_version.to_string();
    COMPILED_MOVE_CODE_DIR
        .get_dir(Path::new(sub_dir.as_str()))
        .expect("read release bundles dir should be ok")
        .files()
        .iter()
        .filter(|file| file.path().extension().is_some_and(|ext| ext == "mrb"))
        .map(|file| {
            let name = file
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let content = file.contents();
            let bundle = bcs_ext::from_bytes::<ReleaseBundle>(content).expect("bcs succeeds");
            (name, bundle.legacy_copy_code())
        })
        .collect()
}

pub fn stdlib_dummy_upgrade_script_function() -> EntryFunction {
    match stdlib_upgrade_scripts_dummy_upgrade() {
        TransactionPayload::EntryFunction(e) => e,
        _ => panic!("stdlib_dummy_upgrade_script_function should be some"),
    }
}

pub fn stdlib_upgrade_init_script(
    current: StdlibVersion,
    new: StdlibVersion,
) -> Option<EntryFunction> {
    use StdlibVersion::*;
    if current >= new {
        return None;
    }
    match (current, new) {
        (Version(12), Latest) => Some(stdlib_dummy_upgrade_script_function()),
        //(Version(12), Version(13)) => Some(stdlib_upgrade_scripts_upgrade_from_v12_to_v13()),
        _ => None,
    }
}

fn load_compiled_modules(version: StdlibVersion, package_name: &str) -> Vec<CompiledModule> {
    let package_file = format!("{version}/{package_name}.mrb");
    let package = COMPILED_MOVE_CODE_DIR
        .get_file(package_file)
        .map(|file| {
            let mrb = bcs_ext::from_bytes::<ReleaseBundle>(file.contents())
                .expect("Decode release bundle should success");
            mrb.compiled_modules()
        })
        .expect("Can not find package");

    package
}

pub fn modules_diff(
    first_modules: &[CompiledModule],
    second_modules: &[CompiledModule],
) -> Vec<CompiledModule> {
    let mut update_modules = vec![];
    let first_modules = first_modules
        .iter()
        .map(|module| (module.self_id(), module.clone()))
        .collect::<BTreeMap<_, _>>();
    for module in second_modules {
        let module_id = module.self_id();
        let is_new = if let Some(old_module) = first_modules.get(&module_id) {
            old_module != module
        } else {
            true
        };
        if is_new {
            update_modules.push(module.clone());
        }
    }
    update_modules
}

pub fn load_upgrade_package(
    current_version: StdlibVersion,
    new_version: StdlibVersion,
    current_package_name: &str,
    upgrade_package_name: &str,
) -> anyhow::Result<Option<Package>> {
    let init_script =
        stdlib_upgrade_init_script(current_version, new_version).ok_or(format_err!(
            "No upgrade script between version {} and {}",
            current_version,
            new_version
        ))?;
    let package = match (current_version, new_version) {
        (StdlibVersion::Version(previous_version), StdlibVersion::Version(new_version)) => {
            ensure!(
                previous_version < new_version,
                "previous version should < new version"
            );

            let package_file = format!("{new_version}/{upgrade_package_name}.mrb");
            let package = COMPILED_MOVE_CODE_DIR
                .get_file(package_file)
                .map(|file| {
                    let mrb = bcs_ext::from_bytes::<ReleaseBundle>(file.contents())
                        .expect("Decode release bundle should success");
                    let modules = mrb.legacy_copy_code();
                    Package::new(
                        modules.into_iter().map(Module::new).collect(),
                        Some(init_script),
                    )
                    .expect("Create package should success")
                })
                .ok_or_else(|| {
                    format_err!(
                        "Can not find upgrade package between version {} and {}",
                        current_version,
                        new_version
                    )
                })?;
            Some(package)
        }
        (current_version @ StdlibVersion::Version(_), StdlibVersion::Latest) => {
            let current_modules = load_compiled_modules(current_version, current_package_name);
            let latest_modules = load_compiled_modules(StdlibVersion::Latest, upgrade_package_name);
            let diff = modules_diff(&current_modules, &latest_modules);
            let modules: Vec<_> = diff
                .into_iter()
                .map(|m| {
                    let mut blob = vec![];
                    m.serialize(&mut blob).unwrap();
                    blob
                })
                .collect();
            Package::new(
                modules.into_iter().map(Module::new).collect(),
                Some(init_script),
            )
            .ok()
        }
        (StdlibVersion::Latest, _) => {
            bail!("Current version is latest, can not upgrade.");
        }
    };
    info!(
        "load_upgrade_package({:?},{:?}), hash: {:?}",
        current_version,
        new_version,
        package.as_ref().map(|package| package.crypto_hash())
    );
    Ok(package)
}
