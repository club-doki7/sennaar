{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        libclang = pkgs.llvmPackages_18.libclang;
        rust = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src" "rust-analyzer"
          ];
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            rust

            # libclang
            libclang
          ];

          LIBCLANG_PATH = lib.makeLibraryPath [ libclang.lib ];
        };
      }
    );
}
