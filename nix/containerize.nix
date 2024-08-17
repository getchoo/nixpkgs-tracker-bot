{ lib, dockerTools }:
let
  containerize =
    nixpkgs-tracker-bot:
    let
      inherit (nixpkgs-tracker-bot.passthru) crossPkgs;
      architecture = crossPkgs.go.GOARCH;
    in
    dockerTools.buildLayeredImage {
      name = "nixpkgs-tracker-bot";
      tag = "latest-${architecture}";
      config.Cmd = [ (lib.getExe nixpkgs-tracker-bot) ];
      inherit architecture;
    };
in
containerize
