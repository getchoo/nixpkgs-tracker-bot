{
  lib,
  stdenv,
  openssl,
  pkg-config,
  removeReferencesTo,
  rustPlatform,

  self,
  lto ? true,
  optimizeSize ? false,
}:

let
  inherit (self) lastModifiedDate;
  date =
    let
      year = lib.substring 0 4 lastModifiedDate;
      month = lib.substring 4 2 lastModifiedDate;
      day = lib.substring 6 2 lastModifiedDate;
    in
    if (self ? "lastModifiedDate") then
      lib.concatStringsSep "-" [
        year
        month
        day
      ]
    else
      "0";
in

rustPlatform.buildRustPackage {
  pname = "nixpkgs-tracker-bot";
  version = "0.2.0-unstable-${date}";

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.lock
      ../Cargo.toml

      ../crates
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    removeReferencesTo
  ];

  buildInputs = [ openssl ];

  # `-C panic="abort"` breaks checks
  doCheck = !optimizeSize;

  postFixup = lib.optionalString stdenv.hostPlatform.isStatic ''
    find "$out" \
      -type f \
      -exec remove-references-to -t ${stdenv.cc.libc} -t ${openssl.dev} '{}' +
  '';

  env =
    let
      rustFlags =
        lib.optionalAttrs lto {
          lto = "thin";
          embed-bitcode = "yes";
        }
        // lib.optionalAttrs optimizeSize {
          codegen-units = 1;
          opt-level = "s";
          panic = "abort";
          strip = "symbols";
        };
    in
    {
      RUSTFLAGS = toString (lib.mapAttrsToList (name: value: "-C ${name}=${toString value}") rustFlags);
    }
    // lib.optionalAttrs stdenv.hostPlatform.isStatic {
      OPENSSL_STATIC = 1;
    };

  meta = {
    description = "Discord app for tracking Nixpkgs pull requests";
    homepage = "https://github.com/getchoo/nixpkgs-tracker-bot";
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.getchoo ];
    mainProgram = "nixpkgs-tracker-bot";
  };
}
