{
  pkgs,
  lib,
  crane,
  rust-overlay,
  advisory-db,
  system,
  self,
  ...
}:
let
  rustToolchain = pkgs.rust-bin.stable.latest.default.override {
    targets = [ "x86_64-pc-windows-msvc" ];
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  src =
    let
      unfilteredRoot = ../cli/.;
    in
    lib.fileset.toSource {
      root = unfilteredRoot;
      fileset = lib.fileset.unions [
        (craneLib.fileset.commonCargoSources unfilteredRoot)
        (lib.fileset.fileFilter (file: file.hasExt "md") unfilteredRoot)
        (lib.fileset.maybeMissing (unfilteredRoot + /assets/.))
      ];
    };

  commonArgs = {
    inherit src;
    strictDeps = true;
    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.alsa-lib ] ++ lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
    CARGO_BUILD_JOBS = 2;
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  my-crate = craneLib.buildPackage (commonArgs // { inherit cargoArtifacts; });
in
{
  checks = {
    inherit my-crate;
    my-crate-clippy = craneLib.cargoClippy (
      commonArgs
      // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      }
    );
    my-crate-doc = craneLib.cargoDoc (
      commonArgs
      // {
        inherit cargoArtifacts;
        env.RUSTDOCFLAGS = "--deny warnings";
      }
    );
    my-crate-fmt = craneLib.cargoFmt { inherit src; };
    my-crate-toml-fmt = craneLib.taploFmt {
      src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
    };
    my-crate-audit = craneLib.cargoAudit { inherit src advisory-db; };
    my-crate-deny = craneLib.cargoDeny { inherit src; };
    my-crate-nextest = craneLib.cargoNextest (
      commonArgs
      // {
        inherit cargoArtifacts;
        partitions = 1;
        partitionType = "count";
        cargoNextestPartitionsExtraArgs = "--no-tests=pass";
      }
    );
  };

  package = my-crate;

  devShell = craneLib.devShell {
    checks = self.checks.${system};
    packages = with pkgs; [
      rustfmt
      rust-analyzer
      cargo-xwin
    ];
    shellHook = ''
      export REPO_ROOT=$(git rev-parse --show-toplevel)
      export PS1="\n\[\033[1;32m\][nix-shell:\w]\$\[\033[0m\] "
    '';
  };
}
