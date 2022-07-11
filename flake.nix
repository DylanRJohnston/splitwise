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
  };

  outputs = { self, nixpkgs, flake-utils, naersk, fenix }: flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
      fenix-lib = fenix.packages.${system}.complete;
      naersk-lib = naersk.lib.${system}.override { inherit (fenix-lib) cargo rustc; };
      darwin-support = if pkgs.stdenv.isDarwin then [ pkgs.darwin.apple_sdk.frameworks.Security ] else [ ];
    in
    {
      devShell = pkgs.mkShell {
        buildInputs = with fenix-lib;
          [
            cargo
            rustc
            rustfmt
            clippy
            pkgs.libiconv
            pkgs.sqlite
            pkgs.lldb
          ] ++ darwin-support;

        RUST_SRC_PATH = "${fenix-lib.rust-src}/lib/rustlib/src/rust/library";
      };
    });
}
