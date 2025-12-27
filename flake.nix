{
  description = "Pixelflut development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, fenix, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

        # Get nightly Rust toolchain
        rustNightly = fenix.packages.${system}.latest.toolchain;

        # Get rust-analyzer from nixpkgs (or use fenix's rust-analyzer)
        rustAnalyzer = fenix.packages.${system}.latest.rust-analyzer;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustNightly
            rustAnalyzer
          ];

          shellHook = ''
            echo "Rust Nightly Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "rust-analyzer: $(rust-analyzer --version 2>/dev/null || echo 'available')"
          '';
        };
      }
    );
}

