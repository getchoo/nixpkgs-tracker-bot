{
  lib,
  stdenv,
  openssl,
  pkg-config,
  rustPlatform,

  self,
  nix-filter,
  lto ? true,
  optimizeSize ? false,
}:

rustPlatform.buildRustPackage {
  pname = "nixpkgs-tracker-bot";
  version = self.shortRev or self.dirtyShortRev or "unknown";

  src = nix-filter.lib.filter {
    root = self;
    include = [
      "crates"
      "Cargo.toml"
      "Cargo.lock"
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl ];

  env =
    let
      rustFlags =
        lib.optionalAttrs lto {
          lto = "thin";
        }
        // lib.optionalAttrs optimizeSize {
          codegen-units = 1;
          opt-level = "s";
          panic = "abort";
          strip = "symbols";
        };
    in
    {
      CARGO_BUILD_RUSTFLAGS = toString (
        lib.mapAttrsToList (name: value: "-C ${name}=${toString value}") rustFlags
      );
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
