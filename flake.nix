{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        fontconfigLib = if pkgs.fontconfig ? lib then pkgs.fontconfig.lib else pkgs.fontconfig;
        opensslLib = if pkgs.openssl ? out then pkgs.openssl.out else pkgs.openssl;
        ldLibraryPath = pkgs.lib.makeLibraryPath [
          pkgs.zlib
          pkgs.stdenv.cc.cc.lib
          opensslLib
          fontconfigLib
          pkgs.freetype
        ];
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.rustup
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.fontconfig
            pkgs.freetype
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
            export SCCACHE_DIR="$HOME/.cache/sccache"
            export SCCACHE_CACHE_SIZE="100G"

            if [ -f rust-toolchain.toml ]; then
              rust_version=$(grep 'channel' rust-toolchain.toml | cut -d '"' -f 2)
              rustup override set "$rust_version"
              rustup component add rust-src --toolchain "$rust_version" 2>/dev/null || true
              rustup component add rust-analyzer --toolchain "$rust_version" 2>/dev/null || true
            fi
          '';

          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          CARGO_INCREMENTAL = "0";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = ldLibraryPath;
          OPENSSL_NO_VENDOR = "1";
        };
      });
}
