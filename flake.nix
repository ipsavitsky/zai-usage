{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      naersk,
      rust-overlay,
      treefmt-nix,
    }:
    flake-utils.lib.eachSystem
      [ flake-utils.lib.system.x86_64-linux flake-utils.lib.system.aarch64-linux ]
      (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          naersk-lib = pkgs.callPackage naersk {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
          treefmtModule = treefmt-nix.lib.evalModule pkgs ./treefmt.nix;
        in
        rec {
          formatter = treefmtModule.config.build.wrapper;

          checks = {
            formatting = treefmtModule.config.build.check self;
          };

          packages = rec {
            default = zai-usage;
            zai-usage = naersk-lib.buildPackage {
              src = ./.;
            };
          };

          devShells = {
            default = pkgs.mkShell {
              packages = with pkgs; [
                watchexec
                just
                rustToolchain
                nil
              ];
            };
          };
        }
      );
}
