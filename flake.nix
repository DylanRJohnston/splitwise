{
  inputs = {
    cargo2nix.url = "github:cargo2nix/cargo2nix/release-0.11.0";
    flake-utils.follows = "cargo2nix/flake-utils";
    nixpkgs.follows = "cargo2nix/nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, cargo2nix, ... }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs { inherit system; overlays = [ cargo2nix.overlays.default ]; };
      rustpkgs = pkgs.rustBuilder.makePackageSet {
        rustVersion = "2022-04-29";
        rustChannel = "nightly";
        packageFun = import ./Cargo.nix;
        extraRustComponents = [ "clippy" ];
      };
    in
    rec {
      devShell = rustpkgs.workspaceShell {
        packages = [ pkgs.terraform ];
      };
      packages = {
        default = packages.lambda-binary;
        binary = (rustpkgs.workspace.splitwise-ynab { }).bin;
        zip = pkgs.runCommand "lambda" { } ''
          cp ${packages.binary}/bin/lambda bootstrap
          ${pkgs.zip}/bin/zip bootstrap.zip bootstrap
          mv bootstrap.zip $out
        '';
      };
    }
  );
}
