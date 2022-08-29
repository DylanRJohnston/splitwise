{
  inputs = {
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    # https://github.com/numtide/nix-filter/issues/28
    nix-filter.url = "github:numtide/nix-filter/3e1fff9ec0112fe5ec61ea7cc6d37c1720d865f8";
  };

  outputs = { self, nixpkgs, flake-utils, nix-filter, naersk, fenix }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      complete-toolchain = fenix.packages.${system}.complete;
      lambda-toolchain = with fenix.packages.${system}; combine [
        minimal.cargo
        minimal.rustc
        targets.aarch64-unknown-linux-gnu.latest.rust-std
      ];
      rust = naersk.lib.${system}.override {
        cargo = complete-toolchain.cargo;
        rustc = complete-toolchain.rustc;
      };
      rust-lambda = naersk.lib.${system}.override {
        cargo = lambda-toolchain;
        rustc = lambda-toolchain;
      };
      darwin-support = if pkgs.stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.Security ] else [ ];
    in
    rec {
      devShell = pkgs.mkShell {
        buildInputs =
          [
            complete-toolchain.toolchain
            pkgs.libiconv
            pkgs.terraform
            pkgs.zig
          ] ++ darwin-support;

        RUST_SRC_PATH = "${complete-toolchain.rust-src}/lib/rustlib/src/rust/library";
      };
      defaultPacakge = packages.cmd;
      packages = {
        cmd = rust.buildPackage {
          strictDeps = true;
          depsBuildBuild = [ pkgs.pkgsStatic.stdenv.cc ];
          nativeBuildInputs = [ pkgs.pkgsStatic.stdenv.cc ];
          src = nix-filter.lib {
            root = ./.;
            include = [
              "Cargo.lock"
              "Cargo.toml"
              (nix-filter.lib.inDirectory "own")
              (nix-filter.lib.inDirectory "splitwise")
            ];
          };
        };
        lambda-binary = rust-lambda.buildPackage {
          CARGO_BUILD_TARGET = "aarch64-unknown-linux-gnu";
          src = nix-filter.lib {
            root = ./.;
            include = [
              "Cargo.lock"
              "Cargo.toml"
              (nix-filter.lib.inDirectory "own")
              (nix-filter.lib.inDirectory "splitwise")
            ];
          };
        };
        lambda = pkgs.runCommand "lambda" { } ''
          cp ${packages.lambda-binary}/bin/lambda bootstrap
          ${pkgs.zip}/bin/zip bootstrap.zip bootstrap
          mv bootstrap.zip $out
        '';
      };
    }
  );
}
