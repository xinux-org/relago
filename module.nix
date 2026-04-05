flake:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  # Manifest via Cargo.toml
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;

  # Options
  cfg = config.services.${manifest.name};

  # Flake shipped default binary
  fpkg = flake.packages.${pkgs.stdenv.hostPlatform.system}.default;

  # Toml management
  toml = pkgs.formats.toml { };

  # The digesting configuration of server
  toml-config = toml.generate "config.toml" {
    parallel_compression = cfg.parallel-compression;
    tmp_dir = cfg.tmp-dir;
    data_dir = cfg.data-dir;
    nix_config = cfg.nix-config;
    server = cfg.server;
  };

  # Systemd services
  service = lib.mkIf cfg.enable {
    services.dbus.packages = [ fpkg ];

    users.users.${cfg.user} = {
      description = "relago user";
      isSystemUser = true;
      group = cfg.group;
      home = cfg.data-dir;
      useDefaultShell = true;
    };

    users.groups = {
      ${cfg.group} = { };
    };

    systemd.targets."${manifest.name}" = { };

    systemd.services."${manifest.name}-config" = {
      wantedBy = [ "multi-user.target" ];
      partOf = [ "${manifest.name}.target" ];
      path = with pkgs; [ jq ];

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        TimeoutSec = "infinity";
        Restart = "on-failure";
        WorkingDirectory = "/tmp";
        RemainAfterExit = true;

        StateDirectory = "relago";
        StateDirectoryMode = "0750";

        ExecStartPre =
          let
            preStartFullPrivileges = ''
              set -o errexit -o pipefail -o nounset
              shopt -s dotglob nullglob inherit_errexit

              mkdir -p '${cfg.data-dir}'

              chown -R --no-dereference '${cfg.user}':'${cfg.group}' '${cfg.data-dir}'
              chmod -R u+rwX,g+rX,o-rwx '${cfg.data-dir}'
            '';
          in
          "+${pkgs.writeShellScript "${manifest.name}-pre-start-full-privileges" preStartFullPrivileges}";

        ExecStart = pkgs.writeShellScript "${manifest.name}-config" ''
          set -o errexit -o pipefail -o nounset
          shopt -s inherit_errexit

          umask u=rwx,g=rx,o=

          # Write configuration file for server
          cp -f ${toml-config} ${cfg.data-dir}/config.toml
        '';
      };
    };

    systemd.services."${manifest.name}" = {
      description = "Relago daemon";

      after = [
        "network.target"
        "${manifest.name}-config.service"
      ];
      requires = [ "${manifest.name}-config.service" ];
      wantedBy = [ "multi-user.target" ];
      wants = [ "network-online.target" ];
      path = [ fpkg ];

      serviceConfig = {
        # Type = "dbus";
        # BusName = "org.freedesktop.problems.daemon";

        Type = "exec";

        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        ExecStart = "${lib.getBin fpkg}/bin/relago daemon";
        ExecReload = "${pkgs.coreutils}/bin/kill -s HUP $MAINPID";
        ReadWritePaths = [
          cfg.data-dir
        ];

        StateDirectory = cfg.user;
        StateDirectoryMode = "0750";

        CapabilityBoundingSet = [
          "AF_NETLINK"
          "AF_INET"
          "AF_INET6"
        ];
        DeviceAllow = [ "/dev/stdin r" ];
        DevicePolicy = "strict";
        # IPAddressAllow = "localhost";
        LockPersonality = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = false;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        ReadOnlyPaths = [ "/" ];
        RemoveIPC = true;
        RestrictNamespaces = true;
        RestrictRealtime = true;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        SystemCallFilter = [
          "@system-service"
          "~@privileged"
          "~@resources"
          "@pkey"
        ];
        UMask = "0027";

        # # StandardInput = "null";
        # # StandardOutput = "journal";
        # # StandardError = "journal";

        # # Restart = "always";

        # DevicePolicy = "closed";
        # KeyringMode = "private";
        # LockPersonality = "yes";
        # MemoryDenyWriteExecute = "yes";
        # NoNewPrivileges = "yes";
        # PrivateDevices = "yes";
        # PrivateTmp = "true";
        # ProtectClock = "yes";
        # ProtectControlGroups = "yes";
        # # ProtectHome = "read-only";
        # ProtectHostname = "yes";
        # ProtectKernelLogs = "yes";
        # ProtectKernelModules = "yes";
        # ProtectKernelTunables = "yes";
        # ProtectProc = "invisible";
        # ProtectSystem = "full";
        # RestrictNamespaces = "yes";
        # RestrictRealtime = "yes";
        # RestrictSUIDSGID = "yes";
        # SystemCallArchitectures = "native";
      };
    };
  };

in
{
  # Available user options
  options = with lib; {
    services.relago = {
      enable = mkEnableOption ''
        Relago
      '';

      parallel-compression = mkOption {
        type = types.int;
        default = 4;
        example = 4;
        description = "How many cores to use while pooling";
      };

      tmp-dir = mkOption {
        type = types.str;
        default = "/var/lib/relago/tmp";
        example = "/var/lib/relago/tmp";
        description = "Temp folder for Relago";
      };

      data-dir = mkOption {
        type = types.str;
        default = "/var/lib/relago";
        example = "/var/lib/relago";
        description = "Folder for Relago";
      };

      nix-config = mkOption {
        type = types.str;
        default = "/etc/nixos/xinux-config";
        example = "/etc/nixos/xinux-config";
        description = "Path of Nixos config";
      };

      server = mkOption {
        type = types.str;
        default = "https://cocomelon.uz";
        example = "https://cocomelon.uz";
        description = "Relago-daemon server";
      };

      user = mkOption {
        type = types.str;
        default = "relago";
        example = "relago";
        description = "User for running system + access keys";
      };

      group = mkOption {
        type = types.str;
        default = "relago";
        example = "relago";
        description = "Group for running system + acess keys";
      };
    };
  };

  config = lib.mkMerge [ service ];
}
