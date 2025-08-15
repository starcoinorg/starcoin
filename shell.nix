{ pkgs ? import <nixpkgs> {} }:

let
  rustup = pkgs.rustup;
in
pkgs.mkShell {
  buildInputs = [
    pkgs.openssl
    pkgs.pkg-config
    pkgs.protobuf
    pkgs.llvmPackages.libcxxClang
    pkgs.rust-analyzer
    rustup
    pkgs.mold     
    pkgs.sccache
    pkgs.stdenv.cc.cc.lib
  ];

  # setgup rustup, read rust version from rust-toolchain.toml
  shellHook = ''
    # if  rust-toolchain.toml exist，use rustup to set Rust version
    if [ -f rust-toolchain.toml ]; then
      rust_version=$(grep 'channel' rust-toolchain.toml | cut -d '"' -f 2)
      rustup override set "$rust_version"
      rustup component add rustfmt --toolchain "$rust_version"
      rustup component add rust-analysis --toolchain "$rust_version"
    fi
    
    # use mold as linker (let mold auto-detect optimal thread count)
    export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
    
    # use incremental compilation for faster rebuilds
    #export CARGO_INCREMENTAL=1
    
    # Set LD_LIBRARY_PATH for runtime linking
    export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.openssl.out}/lib:$LD_LIBRARY_PATH"
    
    # Let cargo/rustc auto-detect optimal parallelism based on CPU cores
    # They will use nproc by default
    
    # Use tmpfs for temporary compilation files (smaller, frequently accessed)
    export TMPDIR="/dev/shm/tmp"
    mkdir -p "$TMPDIR"
    
    # sccache config (disabled, conflicts with incremental compilation)
    # to enable sccache, comment out CARGO_INCREMENTAL=1 above
    # and uncomment the following:
    export RUSTC_WRAPPER=sccache
    export SCCACHE_DIR="$HOME/.cache/sccache"
    export SCCACHE_CACHE_SIZE="100G"
    export CARGO_INCREMENTAL=0
  '';
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
}
