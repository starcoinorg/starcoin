use cc::Build;

fn main() {
    Build::new()
        .include("ext/")
        .file("ext/cryptonight.c")
        .file("ext/aesb.c")
        .file("ext/blake256.c")
        .file("ext/groestl.c")
        .file("ext/hash-extra-blake.c")
        .file("ext/hash-extra-groestl.c")
        .file("ext/hash-extra-skein.c")
        .file("ext/hash-extra-jh.c")
        .file("ext/hash.c")
        .file("ext/jh.c")
        .file("ext/keccak.c")
        .file("ext/oaes_lib.c")
        .file("ext/skein.c")
        .file("ext/slow-hash.c")
        .flag("-maes")
        .flag("-msse2")
        .flag("-Ofast")
        .flag("-fexceptions")
        .compile("cryptonight")
}
