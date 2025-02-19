use anyhow::{format_err, Result};
use starcoin_vm_types::account_config::core_code_address;
use starcoin_vm_types::identifier::Identifier;
use starcoin_vm_types::language_storage::ModuleId;
use starcoin_vm_types::transaction::{Package, ScriptFunction};
use stdlib::COMPILED_MOVE_CODE_DIR;

#[test]
fn test_package_init_function() -> Result<()> {
    let _path_list = [
        "./compiled/2/1-2/stdlib.blob",
        "./compiled/3/2-3/stdlib.blob",
        "./compiled/4/3-4/stdlib.blob",
        "./compiled/5/4-5/stdlib.blob",
        "./compiled/6/5-6/stdlib.blob",
        "./compiled/7/6-7/stdlib.blob",
        "./compiled/8/7-8/stdlib.blob",
        "./compiled/9/8-9/stdlib.blob",
        "./compiled/10/9-10/stdlib.blob",
        "./compiled/11/10-11/stdlib.blob",
        "./compiled/12/11-12/stdlib.blob",
    ];

    let init_strs = [
        "0x00000000000000000000000000000001::PackageTxnManager::convert_TwoPhaseUpgrade_to_TwoPhaseUpgradeV2",
        "0x00000000000000000000000000000001::StdlibUpgradeScripts::upgrade_from_v2_to_v3",
        "",
        "",
        "0x00000000000000000000000000000001::StdlibUpgradeScripts::upgrade_from_v5_to_v6",
        "0x00000000000000000000000000000001::StdlibUpgradeScripts::upgrade_from_v6_to_v7",
        "0x00000000000000000000000000000001::StdlibUpgradeScripts::upgrade_from_v7_to_v8",
        "",
        "",
        "",
        "0x00000000000000000000000000000001::StdlibUpgradeScripts::upgrade_from_v11_to_v12",
    ];
    for (i, version) in (2..=12).collect::<Vec<usize>>().into_iter().enumerate() {
        let package_file = format!("{}/{}-{}/stdlib.blob", version, version - 1, version);
        let package = COMPILED_MOVE_CODE_DIR
            .get_file(package_file)
            .map(|file| {
                bcs_ext::from_bytes::<Package>(file.contents())
                    .expect("Decode package should success")
            })
            .ok_or_else(|| {
                format_err!(
                    "Can not find upgrade package between version {} and {}",
                    version - 1,
                    version
                )
            })?;
        let init_fun = if let Some(init_script) = package.init_script() {
            format!("{}::{}", init_script.module(), init_script.function())
        } else {
            "".to_owned()
        };
        assert_eq!(init_fun, init_strs[i]);
        let package = COMPILED_MOVE_CODE_DIR
            .get_file("12/11-12/stdlib.blob")
            .map(|file| {
                bcs_ext::from_bytes::<Package>(file.contents())
                    .expect("Decode package should success")
            })
            .ok_or_else(|| format_err!("Can not find upgrade package 12/11-12/stdlib.blob"))?;
        let init_script = ScriptFunction::new(
            ModuleId::new(
                core_code_address(),
                Identifier::new("StdlibUpgradeScripts").unwrap(),
            ),
            Identifier::new("upgrade_from_v11_to_v12").unwrap(),
            vec![],
            vec![
                bcs_ext::to_bytes(&23182155u64).unwrap(), // TODO(BobOng): [force-upgrade] to confirm main burn block
                bcs_ext::to_bytes(&16083000u64).unwrap(),
                bcs_ext::to_bytes(&5u64).unwrap(),
                bcs_ext::to_bytes(&1000u64).unwrap(),
            ],
        );

        assert_eq!(package.init_script().unwrap(), &init_script);
    }
    Ok(())
}
