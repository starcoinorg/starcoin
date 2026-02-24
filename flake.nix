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
            pkgs.postgresql
            pkgs.llvmPackages.clang
            pkgs.llvmPackages.libclang
            pkgs.mold
            pkgs.gcc.cc.lib
          ];

          shellHook = ''
            if [ -f rust-toolchain.toml ]; then
              rust_version=$(grep 'channel' rust-toolchain.toml | cut -d '"' -f 2)
              rustup override set "$rust_version"
              rustup component add rust-src --toolchain "$rust_version" 2>/dev/null || true
              rustup component add rust-analyzer --toolchain "$rust_version" 2>/dev/null || true
            fi
          '';

          RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
          CARGO_INCREMENTAL = "1";
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
          LD_LIBRARY_PATH = "${pkgs.zlib}/lib:${pkgs.gcc.cc.lib}/lib:${pkgs.openssl.out}/lib:${pkgs.postgresql.lib}/lib";
          OPENSSL_NO_VENDOR = "1";
        };
      });
}
