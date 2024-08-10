{ pkgsCross }:
let
  crossPkgsFor = with pkgsCross; {
    x86_64 = musl64.pkgsStatic;
    aarch64 = aarch64-multiplatform;
  };
in
{ arch }:
let
  crossPkgs = crossPkgsFor.${arch};
in
(crossPkgs.callPackage ./package.nix { optimizeSize = true; }).overrideAttrs (old: {
  passthru = old.passthru or { } // {
    inherit crossPkgs;
  };
})
