use cc::Build;
use std::env;

fn main() {
    let mut config = Build::new();
    config
        .include("ext/")
        .file("ext/aesb.c")
        .file("ext/c_blake256.c")
        .file("ext/c_groestl.c")
        .file("ext/c_jh.c")
        .file("ext/c_keccak.c")
        .file("ext/c_skein.c")
        .file("ext/cryptonight.c")
        .file("ext/hash-extra-blake.c")
        .file("ext/hash-extra-groestl.c")
        .file("ext/hash-extra-skein.c")
        .file("ext/hash-extra-jh.c")
        .file("ext/hash.c")
        .file("ext/oaes_lib.c")
        .file("ext/slow-hash.c");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is set by cargo.");

    let target = env::var("TARGET").expect("TARGET is set by cargo.");
    if target.contains("x86_64") {
        config.flag("-maes").flag("-msse2");
    }
    if target_os.contains("linux") || target_os.contains("macos") {
        let info = os_info::get();
        let opt_level = env::var("OPT_LEVEL").expect("fetching OPT_LEVEL");
        if info.os_type() == os_info::Type::Ubuntu
            && info.version() == &os_info::Version::Custom("22.04".into())
        {
            if opt_level == "3" {
                config.flag("-O2").flag("-fexceptions").flag("-std=gnu99");
            } else {
                config.flag("-fexceptions").flag("-std=gnu99");
            }
        } else {
            config
                .flag("-Ofast")
                .flag("-fexceptions")
                .flag("-std=gnu99");
        }
    }
    config.compile("cryptonight");
}
