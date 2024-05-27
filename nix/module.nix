self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.nixpkgs-tracker-bot;

  inherit
    (lib)
    getExe
    literalExpression
    mkEnableOption
    mkIf
    mkOption
    mkPackageOption
    types
    ;

  inherit (pkgs.stdenv.hostPlatform) system;
in {
  options.services.nixpkgs-tracker-bot = {
    enable = mkEnableOption "nixpkgs-tracker-bot";
    package = mkPackageOption (
      self.packages.${system} or (throw "${system} is not supported!")
    ) "nixpkgs-tracker-bot" {};

    environmentFile = mkOption {
      description = ''
        Environment file as defined in {manpage}`systemd.exec(5)`
      '';
      type = types.nullOr types.path;
      default = null;
      example = literalExpression ''
        "/run/agenix.d/1/nixpkgs-tracker-bot"
      '';
    };
  };

  config = mkIf cfg.enable {
    systemd.services.nixpkgs-tracker-bot = {
      enable = true;
      wantedBy = ["multi-user.target"];
      after = ["network.target"];

      script = ''
        ${getExe cfg.package}
      '';

      serviceConfig = {
        Type = "simple";
        Restart = "on-failure";

        EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;

        # hardening
        DynamicUser = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        RestrictNamespaces = "uts ipc pid user cgroup";
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@resources"
          "~@privileged"
        ];
        Umask = "0007";
      };
    };
  };
}
