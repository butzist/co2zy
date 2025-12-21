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

        # Use the latest nightly toolchain from rust-overlay
        rustNightly = pkgs.rust-bin.nightly.latest.default.override {
          extensions = ["rust-src"];
          targets = ["riscv32imac-unknown-none-elf"];
        };
      in {
        packages = {
          default = pkgs.hello;
        };

        devShells.default = pkgs.mkShell {
          name = "dev-shell-esp-nightly";

          buildInputs = [
            rustNightly
            pkgs.espup
            pkgs.espflash
            pkgs.esp-generate
            pkgs.rustup
            pkgs.just
            pkgs.lld
            pkgs.llvm
          ];

          shellHook = ''
            echo "🦀 Entered dev shell with nightly Rust"
            flake_root=$(git rev-parse --show-toplevel 2>/dev/null || echo "$PWD")


            # Use rustup shims
            export PATH=${pkgs.rustup}/bin:$PATH
            export RUSTUP_HOME="$flake_root/.rustup"

            # Setup nix-installed toolchain in rustup
            rustup toolchain link nix-nightly ${rustNightly}
            rustup default nix-nightly
          '';
        };
      }
    );
}
