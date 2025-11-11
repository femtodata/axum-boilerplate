# clanky: to build static, uncomment glibc.static, then
# RUSTFLAGS="-C target-feature=+crt-static" cargo build --release --target x86_64-unknown-linux-gnu
{
  description = "axum-boilerplate";

  inputs = {

    rust-overlay.url = "github:oxalica/rust-overlay/stable";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.12";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [cargo2nix.overlays.default];
        };

        cargo2nixPackages = cargo2nix.packages.${system};

        rustPkgs = pkgs.rustBuilder.makePackageSet {
          rustVersion = "1.91.0";
          packageFun = import ./Cargo.nix;
        };
      in rec
      {
        packages = {
          axum-boilerplate = (rustPkgs.workspace.axum-boilerplate {});
          default = packages.axum-boilerplate;
        };

        devShells.default = pkgs.mkShell {
          packages = [
            cargo2nixPackages.cargo2nix
            # openssl
            # postgresql
            # sqlite
          ];
          nativeBuildInputs = with pkgs; [
            duckdb
            postgresql
            # gcc
            # glibc.static
          ];
        };
      });
}
