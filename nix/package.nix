{
  lib,
  rustPlatform,
  version,
  lto ? true,
  optimizeSize ? false,
}:
rustPlatform.buildRustPackage {
  pname = "nixpkgs-tracker-bot";
  inherit version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../src
      ../Cargo.toml
      ../Cargo.lock
    ];
  };

  cargoLock = {
    lockFile = ../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  env = let
    toRustFlags = lib.mapAttrs' (
      name:
        lib.nameValuePair
        "CARGO_BUILD_RELEASE_${lib.toUpper (builtins.replaceStrings ["-"] ["_"] name)}"
    );
  in
    lib.optionalAttrs lto (toRustFlags {
      lto = "thin";
    })
    // lib.optionalAttrs optimizeSize (toRustFlags {
      codegen-units = 1;
      opt-level = "s";
      panic = "abort";
      strip = "symbols";
    });

  meta = {
    description = "A Discord app for tracking nixpkgs pull requests";
    homepage = "https://github.com/getchoo/nixpkgs-tracker-bot";
    mainProgram = "nixpkgs-tracker-bot";
    license = lib.licenses.mit;
    maintainers = [lib.maintainers.getchoo];
  };
}
