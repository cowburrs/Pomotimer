{
  description = "Tauri v2 app";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    # crane-tauri declares no inputs of its own, so there is nothing to follow
    # or override (the previous follows clause warned about non-existent inputs).
    crane-tauri.url = "github:JPHutchins/crane-tauri";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      crane-tauri,
      flake-utils,
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
          pname = pname; # TODO: change
          version = version; # TODO: change
          src = ./.;
          inherit frontend;
        };
      in
      {
        packages = {
          inherit frontend;
          default = tauri.app;
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

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
        };
      }
    );
}
