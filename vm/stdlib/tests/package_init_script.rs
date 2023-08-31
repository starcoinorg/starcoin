use anyhow::Result;
use starcoin_vm_types::transaction::Package;
use std::{fs::File, io::Read, path::PathBuf};

#[test]
fn test_compiled_package_blob() -> Result<()> {
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
    for str in path_list {
        let path = PathBuf::from(str).canonicalize()?;
        let mut bytes = vec![];
        File::open(path)?.read_to_end(&mut bytes)?;
        let package: Package = bcs_ext::from_bytes(&bytes)?;
        if let Some(init_script) = package.init_script() {
            println!("{:#?}", init_script);
        }
    }

    Ok(())
}
