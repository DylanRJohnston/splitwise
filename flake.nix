{
  inputs = {
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.11.0";
      inputs.rust-overlay.follows = "rust-overlay";
    };
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, cargo2nix, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      rustVersion = "2022-08-29";
      rustChannel = "nightly";
      packageFun = import ./Cargo.nix;

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ cargo2nix.overlays.default ];
      };
      pkgsCross = import nixpkgs {
        inherit system;
        pkgsCross.config = "aarch64-unknown-linux-musl";
        overlays = [ cargo2nix.overlays.default ];
      };
      rustpkgs = pkgs.rustBuilder.makePackageSet {
        inherit
          rustVersion
          rustChannel
          packageFun;

        extraRustComponents = [ "clippy" ];
      };
      rustpkgs-lambda = pkgsCross.rustBuilder.makePackageSet {
        inherit
          rustVersion
          rustChannel
          packageFun;

        packageOverrides = pkgs: pkgs.rustBuilder.overrides.all ++ [
          (pkgs.rustBuilder.rustLib.makeOverride {
            name = "splitwise-ynab";
            overrideAttrs = drv: {
              CC = "${cc}/bin/zig-cc";
            };
          })
        ];

        target = "aarch64-unknown-linux-musl";
      };
      cc = pkgs.writeShellApplication {
        name = "zig-cc";
        runtimeInputs = [ pkgs.zig ];
        text = ''
          zig cc -target aarch64-linux-musl "$@"
        '';
      };
    in
    rec {
      devShell = rustpkgs.workspaceShell {
        packages = [ pkgs.terraform ];
      };
      packages = {
        default = packages.binary;
        binary = (rustpkgs.workspace.splitwise-ynab { }).bin;
        binary-lambda = (rustpkgs-lambda.workspace.splitwise-ynab { }).bin;
        zip = pkgs.runCommand "lambda" { } ''
          cp ${packages.binary-lambda}/bin/lambda bootstrap
          ${pkgs.zip}/bin/zip bootstrap.zip bootstrap
          mv bootstrap.zip $out
        '';
      };
    }
  );
}
