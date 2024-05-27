{
  lib,
  arch,
  nixpkgs-tracker-bot,
  fenix,
  pkgsCross,
}: let
  crossTargetFor = with pkgsCross; {
    x86_64 = musl64.pkgsStatic;
    aarch64 = aarch64-multiplatform;
  };

  rustcTargetFor = lib.mapAttrs (lib.const (pkgs: pkgs.stdenv.hostPlatform.rust.rustcTarget)) crossTargetFor;
  rustStdFor = lib.mapAttrs (lib.const (rustcTarget: fenix.targets.${rustcTarget}.stable.rust-std)) rustcTargetFor;

  toolchain = with fenix;
    combine (
      [stable.cargo stable.rustc]
      ++ lib.attrValues rustStdFor
    );

  crossPlatformFor =
    lib.mapAttrs (
      lib.const (pkgs:
        pkgs.makeRustPlatform (
          lib.genAttrs ["cargo" "rustc"] (lib.const toolchain)
        ))
    )
    crossTargetFor;
in
  nixpkgs-tracker-bot.override {
    rustPlatform = crossPlatformFor.${arch};
    optimizeSize = true;
  }
