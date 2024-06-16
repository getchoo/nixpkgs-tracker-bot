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

      environment = {
        # using `/var/lib/private` as we have `DynamicUser` enabled
        BOT_NIXPKGS_PATH = "/var/lib/private/${config.systemd.services.nixpkgs-tracker-bot.serviceConfig.StateDirectory}/nixpkgs";
      };

      serviceConfig = {
        Type = "simple";
        Restart = "on-failure";

        EnvironmentFile = mkIf (cfg.environmentFile != null) cfg.environmentFile;

        StateDirectory = "nixpkgs-tracker-bot";

        # hardening settings
        DynamicUser = true;
        LockPersonality = true;
        MemoryDenyWriteExecute = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateIPC = true;
        PrivateTmp = true;
        PrivateUsers = true;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectProc = "invisible";
        ProtectSystem = "strict";
        RestrictNamespaces = "uts ipc pid user cgroup";
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
        ];
        UMask = "0077";
      };
    };
  };
}
