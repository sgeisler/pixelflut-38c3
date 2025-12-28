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
          buildInputs = with pkgs; [
            rustNightly
            rustAnalyzer
            pkg-config
            ffmpeg_7-full.dev
            ffmpeg_7-full
            llvmPackages.libclang
            stdenv.cc.libc.dev
          ];

          # Set LIBCLANG_PATH for bindgen and rust-analyzer
          LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

          # Set clang include paths for system headers
          BINDGEN_EXTRA_CLANG_ARGS = with pkgs; builtins.concatStringsSep " " [
            "-I${stdenv.cc.libc.dev}/include"
            "-I${llvmPackages.libclang.lib}/lib/clang/${llvmPackages.libclang.version}/include"
          ];

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath (with pkgs; [ ffmpeg_7-full stdenv.cc.cc.lib ])}:''${LD_LIBRARY_PATH:-}"
            echo "Rust Nightly Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo "rust-analyzer: $(rust-analyzer --version 2>/dev/null || echo 'available')"
            echo "LIBCLANG_PATH: $LIBCLANG_PATH"
            echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
          '';
        };
      }
    );
}

