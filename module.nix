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
    problems_interface = cfg.problems-interface;
    server = cfg.server;
  };

  # Systemd services
  service = lib.mkIf cfg.enable {
    ## User for our services
    # users.users = lib.mkIf (cfg.user == manifest.name) {
    #   ${manifest.name} = {
    #     description = "${manifest.name} Service";
    #     home = cfg.dataDir;
    #     useDefaultShell = true;
    #     group = cfg.group;
    #     isSystemUser = true;
    #   };
    # };

    # # Group to join our user
    # users.groups = mkIf (cfg.group == manifest.name) {
    #   ${manifest.name} = {};
    # };

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

    systemd.services."relago-daemon-config" = {
      wantedBy = [ "relago-daemon.target" ];
      partOf = [ "relago-daemon.target" ];

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        TimeoutSec = "infinity";
        Restart = "on-failure";
        WorkingDirectory = "${cfg.data-dir}";
        RemainAfterExit = true;

        ExecStartPre =
          let
            preStartFullPrivileges = ''
              echo "fuck you" > /tmp/msg

              set -o errexit -o pipefail -o nounset
              shopt -s dotglob nullglob inherit_errexit

              chown -R --no-dereference '${cfg.user}':'${cfg.group}' '${cfg.data-dir}'
              chmod -R u+rwX,g+rX,o-rwx '${cfg.data-dir}'
            '';
          in
          # "+${pkgs.writeShellScript "${packageName}-pre-start-full-privileges" preStartFullPrivileges}";
          "+${pkgs.writeShellScript "relago-pre-start-full-privileges" preStartFullPrivileges}";

        ExecStart = pkgs.writeShellScript "relago-config" ''
          set -o errexit -o pipefail -o nounset
          shopt -s inherit_errexit

          umask u=rwx,g=rx,o=

          # Write configuration file for server
          cp -f ${toml-config} ${cfg.data-dir}/config.toml
        '';
      };
    };

    systemd.services."relago-daemon" = {
      description = "Relago daemon";

      after = [
        # "network-online.target"
        "relago-daemon-config.service"
      ];
      # wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        # Type = "dbus";
        # BusName = "org.freedesktop.problems.daemon";

        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        ExecStart = ''
          pwd
          ls ${toml-config}
          ${lib.getBin fpkg}/bin/relago

        '';
        ExecReload = "${pkgs.coreutils}/bin/kill -s HUP $MAINPID";

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
        RestrictAddressFamilies = [
          "AF_NETLINK"
          "AF_INET"
          "AF_INET6"
          "AF_UNIX"
        ];
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
        default = "/var/lib/relago-daemon/tmp/";
        example = "/var/lib/relago-daemon/tmp/";
        description = "Temp folder for Relago";
      };

      data-dir = mkOption {
        type = types.str;
        default = "/var/lib/relago-daemon/";
        example = "/var/lib/relago-daemon/";
        description = "Folder for Relago";
      };

      nix-config = mkOption {
        type = types.str;
        default = "/etc/nixos/xinux-config";
        example = "/etc/nixos/xinux-config";
        description = "Path of Nixos config";
      };

      problems-interface = mkOption {
        type = types.str;
        default = "org.freedesktop.problems.daemon";
        example = "org.freedesktop.problems.daemon";
        description = "Notification daemon";
      };

      server = mkOption {
        type = types.str;
        default = "https://cocomelon.uz";
        example = "https://cocomelon.uz";
        description = "Relago-daemon server";
      };

      user = mkOption {
        type = types.str;
        default = "relago-daemon";
        example = "relago-daemon";
        description = "User for running system + access keys";
      };

      group = mkOption {
        type = types.str;
        default = "relago-daemon";
        example = "relago-daemon";
        description = "Group for running system + acess keys";
      };
    };
  };

  config = lib.mkMerge [ service ];
}
