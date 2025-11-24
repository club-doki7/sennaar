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
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
            # rust-overlay
            openssl pkg-config eza fd 
            rust-bin.stable.latest.default

            # libclang
            libclang
          ];

          LIBCLANG_PATH = lib.makeLibraryPath [ libclang.lib ];

          shellHook = ''
            alias ls=eza
            alias find=fd
          '';
        };
      }
    );
}