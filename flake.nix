{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/git-hooks.nix";
    v-utils.url = "github:valeratrades/.github?ref=v1.2";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, pre-commit-hooks, v-utils, ... }:
    let
      manifest = (nixpkgs.lib.importTOML ./Cargo.toml).package;
      pname = manifest.name;
    in
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = builtins.trace "flake.nix sourced" [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rust = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer" "rust-docs" "rustc-codegen-cranelift-preview" ];
          });
          pre-commit-check = pre-commit-hooks.lib.${system}.run (v-utils.files.preCommit { inherit pkgs; });
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;

          github = v-utils.github {
            inherit pkgs pname;
            lastSupportedVersion = "nightly-2025-10-12";
            jobsErrors = [ "rust-tests" ];
            jobsWarnings = [ "rust-doc" "rust-clippy" "rust-machete" "rust-sorted" "rust-sorted-derives" "tokei" ];
            jobsOther = [ "loc-badge" ];
            langs = [ "rs" ];
          };
          readme = v-utils.readme-fw { inherit pkgs pname; lastSupportedVersion = "nightly-1.92"; rootDir = ./.; licenses = [{ name = "Blue Oak 1.0.0"; outPath = "LICENSE"; }]; badges = [ "msrv" "crates_io" "docs_rs" "loc" "ci" ]; };
        in
        {
          packages =
            let
              rustc = rust;
              cargo = rust;
              rustPlatform = pkgs.makeRustPlatform {
                inherit rustc cargo stdenv;
              };
            in
            {
              default = rustPlatform.buildRustPackage rec {
                inherit pname;
                version = manifest.version;

                buildInputs = with pkgs; [
                  openssl.dev
                ];
                nativeBuildInputs = with pkgs; [ pkg-config ];

                cargoLock.lockFile = ./Cargo.lock;
                src = pkgs.lib.cleanSource ./.;
              };
            };

          devShells.default = with pkgs; mkShell {
            inherit stdenv;
            shellHook =
              pre-commit-check.shellHook +
              github.shellHook +
              ''
                cp -f ${v-utils.files.licenses.blue_oak} ./LICENSE

                mkdir -p ./.cargo
                cp -f ${(v-utils.files.treefmt) {inherit pkgs;}} ./.treefmt.toml
                cp -f ${(v-utils.files.rust.rustfmt {inherit pkgs;})} ./rustfmt.toml
                cp -f ${(v-utils.files.rust.config {inherit pkgs;})} ./.cargo/config.toml

                cp -f ${readme} ./README.md
              '';

            env = {
              RUST_BACKTRACE = 1;
              RUST_LIB_BACKTRACE = 0;
            };

            packages = [
              mold
              openssl
              pkg-config
              rust
            ] ++ pre-commit-check.enabledPackages ++ github.enabledPackages;
          };
        }
      );
}
