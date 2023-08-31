use anyhow::Result;
use starcoin_vm_types::transaction::Package;
use std::{fs::File, io::Read, path::PathBuf};

#[test]
fn test_package_init_function() -> Result<()> {
    let path_list = [
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
    for (i, str) in path_list.iter().enumerate() {
        let path = PathBuf::from(*str).canonicalize()?;
        let mut bytes = vec![];
        File::open(path)?.read_to_end(&mut bytes)?;
        let package: Package = bcs_ext::from_bytes(&bytes)?;
        let init_fun = if let Some(init_script) = package.init_script() {
            format!("{}::{}", init_script.module(), init_script.function())
        } else {
            "".to_owned()
        };
        assert_eq!(init_fun, init_strs[i]);
    }
    Ok(())
}
