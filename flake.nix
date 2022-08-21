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
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = { self, nixpkgs, flake-utils, nix-filter, naersk, fenix }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      fenix-lib = fenix.packages.${system}.complete;
      naersk-lib = naersk.lib.${system}.override { inherit (fenix-lib) cargo rustc; };
      darwin-support = if pkgs.stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.Security ] else [ ];
    in
    {
      devShell = pkgs.mkShell {
        buildInputs =
          [
            fenix-lib.toolchain
            pkgs.libiconv
            pkgs.sqlite
            pkgs.terraform
            pkgs.awscli2
          ] ++ darwin-support;

        RUST_SRC_PATH = "${fenix-lib.rust-src}/lib/rustlib/src/rust/library";
      };
      defaultPackage = naersk-lib.buildPackage {
        src = nix-filter.lib {
          root = ./.;
          include = [
            "Cargo.lock"
            "Cargo.toml"
            (nix-filter.lib.inDirectory "src")
          ];
        };
      };
    });
}
