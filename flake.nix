{
  description = "Tauri v2 app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    # crane-tauri declares no inputs of its own, so there is nothing to follow
    # or override (the previous follows clause warned about non-existent inputs).
    crane-tauri.url = "github:JPHutchins/crane-tauri";
    flake-utils.url = "github:numtide/flake-utils";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      crane,
      crane-tauri,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        inherit (pkgs) lib;
        craneLib = crane.mkLib pkgs;

        package = (lib.importTOML ./src-tauri/Cargo.toml).package;
        pname = package.name;
        version = package.version;

        frontend = pkgs.buildNpmPackage {
          pname = pname;
          version = version;
          src = lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./package.json
              ./package-lock.json
              ./vite.config.js
              ./index.html
              ./src
              ./public
            ];
          };
          npmDeps = pkgs.importNpmLock {
            npmRoot = ./.;
          };
          npmConfigHook = pkgs.importNpmLock.npmConfigHook;
          installPhase = ''
            runHook preInstall
            cp -r dist $out
            runHook postInstall
          '';
        };

        tauri = crane-tauri.lib.buildTauriApp { inherit pkgs craneLib; } {
          pname = pname;
          version = version;
          src = ./.;
          inherit frontend;
          craneArgs = {
            CARGO_BUILD_JOBS = "4";
          };
        };

        pkgsCli = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

      in
      {
        packages = {
          inherit frontend;
          default = tauri.app;
          cli =
            (import ./nix/cli.nix (
              {
                inherit system self;
                pkgs = pkgsCli;
                lib = pkgs.lib;
              }
              // inputs
            )).package;
        };

        checks = {
          inherit (tauri) app;

          clippy = craneLib.cargoClippy (
            tauri.commonArgs
            // {
              cargoArtifacts = tauri.cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- -D warnings";
              TAURI_CONFIG = tauri.tauriConfig;
            }
          );

          fmt = craneLib.cargoFmt { src = tauri.commonArgs.src; };
        };

        devShells = {

          pomotimer = pkgs.mkShell {
            packages = [ tauri.app ];
            shellHook = ''
              export REPO_ROOT=$(git rev-parse --show-toplevel)
              export PS1="Pomotimer $"
              export PS1="\[\e[38;5;141m\]❯\[\e[0m\] "
              clear
            '';
          };

          default = craneLib.devShell {
            checks = self.checks.${system};

            buildInputs = with pkgs; [
              librsvg
              webkitgtk_4_1
            ];

            packages = [
            ]
            ++ (with pkgs; [
              rustfmt
              rust-analyzer
              cargo-xwin
              typescript-language-server
              cargo-tauri
              vite
              nodejs
              pkg-config
              wrapGAppsHook4
              rustc
              prettier
            ]);

            shellHook = ''
              export REPO_ROOT=$(git rev-parse --show-toplevel)
              export PS1="\n\[\033[1;32m\][nix-shell:\w]\$\[\033[0m\] "
              cargo() {
              case "$1" in
              build|run) echo "use nix to build/run instead" ;;
              *) command cargo "$@" ;;
              esac
              }
              export XDG_DATA_DIRS="$GSETTINGS_SCHEMAS_PATH" # Needed on Wayland to report the correct display scale
            '';
          };
        };

      }
    );
}
