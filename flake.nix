{
  description = "Dev shell for Dioxus + ESP + WASM with nightly Rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {inherit system overlays;};
      in {
        packages = {
          default = pkgs.hello;
        };

        devShells.default = pkgs.mkShell {
          name = "dev-shell-esp-nightly";

          buildInputs = [
            pkgs.espup
            pkgs.espflash
            pkgs.esp-generate
            pkgs.rustup
            pkgs.just
            pkgs.lld
            pkgs.llvm
          ];
        };
      }
    );
}
