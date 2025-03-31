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

rustPlatform.buildRustPackage {
  pname = "nixpkgs-tracker-bot";
  version = self.shortRev or self.dirtyShortRev or "unknown";

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
    description = "A Discord app for tracking nixpkgs pull requests";
    homepage = "https://github.com/getchoo/nixpkgs-tracker-bot";
    license = lib.licenses.mit;
    maintainers = [ lib.maintainers.getchoo ];
    mainProgram = "nixpkgs-tracker-bot";
  };
}
