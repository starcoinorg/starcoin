{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rustup
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.protobuf
            pkgs.zlib
            pkgs.openssl
            pkgs.llvmPackages.clang
            pkgs.llvmPackages.libclang
            pkgs.stdenv.cc.cc.lib
            pkgs.mold
            pkgs.sccache
          ];

          shellHook = ''
            # Set SCCACHE_DIR with proper home directory expansion
            export SCCACHE_DIR="''${~/.cache}/sccache"
            export SCCACHE_CACHE_SIZE="100G"

            if [ -f rust-toolchain.toml ]; then
              rust_version=$(grep 'channel' rust-toolchain.toml | cut -d '"' -f 2)
              rustup override set "$rust_version"
              rustup component add rust-src --toolchain "$rust_version" 2>/dev/null || true
              rustup component add rust-analyzer --toolchain "$rust_version" 2>/dev/null || true
            fi
          '';

          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          RUSTC_WRAPPER = "sccache";
          CARGO_INCREMENTAL = "0";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = "${pkgs.zlib}/lib:${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.openssl.out}/lib";
          OPENSSL_NO_VENDOR = "1";
        };
      });
}
