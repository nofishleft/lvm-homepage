self:
{ config, lib, pkgs, ... }:
let
  cfg = config.services.lvm-homepage;
in
{
  options.services.lvm-homepage = {
    enable = lib.mkEnableOption "lvm-homepage service";

    port = lib.mkOption {
      type = lib.types.port;
      default = 9000;
      description = "Port to listen on";
    };

    host = lib.mkOption {
      type = lib.types.str;
      default = "127.0.0.1";
      description = "Host address to bind to";
    };

    user = lib.mkOption {
      type = lib.types.str;
      default = "lvm-homepage";
      description = "User account under which lvm-homepage runs";
    };

    group = lib.mkOption {
      type = lib.types.str;
      default = "lvm-homepage";
      description = "Group under which lvm-homepage runs";
    };

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.lvm-homepage;
      defaultText = lib.literalExpression "pkgs.lvm-homepage";
      description = "The lvm-homepage package to use";
    };
  };

  config = lib.mkIf cfg.enable {
    nixpkgs.overlays = [ self.overlays.default ];

    users.users.${cfg.user} = {
      isSystemUser = true;
      group = cfg.group;
      description = "lvm-homepage service user";
    };

    users.groups.${cfg.group} = {};

    security.sudo.extraRules = [
      {
        users = [ cfg.user ];
        commands = [
          {
            command = "${pkgs.lvm2.bin}/bin/lvs";
            options = [ "NOPASSWD" ];
          }
          {
            command = "${pkgs.lvm2.bin}/bin/pvs";
            options = [ "NOPASSWD" ];
          }
          {
            command = "${pkgs.lvm2.bin}/bin/vgs";
            options = [ "NOPASSWD" ];
          }
        ];
      }
    ];

    systemd.services.lvm-homepage = {
      description = "LVM Homepage Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        User = cfg.user;
        Group = cfg.group;
        ExecStart = "${cfg.package}/bin/lvm-homepage";
        Restart = "on-failure";
        RestartSec = "5s";

        PrivateTmp = true;
        ProtectSystem = "struct";
        ProtectHome = true;
        ReadWritePaths = [ ];
      };

      environment = {
        PATH = lib.mkForce "/run/wrappers/bin:${lib.makeBinPath [ pkgs.lvm2 ]}";
        HOST = cfg.host;
        PORT = toString cfg.port;
      };
    };
  };
}