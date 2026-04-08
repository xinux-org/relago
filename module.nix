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

  dpkgs = [
    # Policy for daemon
    (pkgs.writeTextDir "share/dbus-1/system.d/org.relago.DaemonService.conf" ''
      <!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
       "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
      <busconfig>
        <policy user="${cfg.user}">
          <allow own="org.relago.DaemonService"/>
        </policy>

        <policy group="${cfg.group}">
          <allow send_destination="org.relago.DaemonService"/>
          <allow receive_sender="org.relago.DaemonService"/>
        </policy>
      </busconfig>
    '')

    (pkgs.writeTextDir "share/dbus-1/system-services/org.relago.DaemonService.service" ''
      [D-BUS Service]
      Name=org.relago.DaemonService
      Exec=${fpkg}/bin/relago daemon
      User=relago
    '')
  ];

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
    services.dbus.packages = dpkgs;

    users.users.${cfg.user} = {
      description = "relago user";
      isSystemUser = true;
      group = cfg.group;
      home = cfg.data-dir;
      useDefaultShell = true;
      extraGroups = [
        "systemd-journal"
      ];
    };

    users.groups = {
      ${cfg.group} = { };
    };

    systemd.targets."${manifest.name}" = { };

    systemd.services."${manifest.name}-config" = {
      wantedBy = [ "multi-user.target" ];
      partOf = [ "${manifest.name}-daemon.target" ];
      path = with pkgs; [ jq ];

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        TimeoutSec = "infinity";
        Restart = "on-failure";
        WorkingDirectory = cfg.tmp-dir;
        RemainAfterExit = true;

        StateDirectory = cfg.user;
        StateDirectoryMode = "0755";

        ExecStartPre =
          let
            preStartFullPrivileges = ''
              set -o errexit -o pipefail -o nounset
              ${pkgs.coreutils}/bin/install -d -m 0755 -o ${cfg.user} -g ${cfg.group} ${cfg.data-dir}
              ${pkgs.coreutils}/bin/install -d -m 0755 -o ${cfg.user} -g ${cfg.group} ${cfg.tmp-dir}
              ${pkgs.coreutils}/bin/install -m 0644 ${toml-config} ${cfg.data-dir}/config.toml

              chmod 0640 ${cfg.data-dir}/config.toml
              chgrp ${cfg.group} ${cfg.data-dir}/config.toml
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

    systemd.user.services."${manifest.name}-gnome" = {
      description = "Relago GNOME UI Agent";
      wantedBy = [ "default.target" ];
      partOf = [ "graphical-session.target" ];
      after = [
        "network.target"
        "graphical-session.target"
      ];
      restartTriggers = [ fpkg ];

      serviceConfig = {
        # Type = "dbus";
        # BusName = "org.relago.ReportService";
        Type = "simple";
        ExecStart = "${lib.getBin fpkg}/bin/relago gnome-relago";
        Restart = "on-failure";
        RestartSec = 5;

        StateDirectory = cfg.user;
        StateDirectoryMode = "0755";

        PrivateDevices = false;
        PrivateTmp = true;
        PrivateUsers = false;
        ProtectClock = false;
        ProtectControlGroups = false;
        ProtectHome = false;
        ProtectHostname = false;
        ProtectKernelLogs = false;
        ProtectKernelTunables = false;
        ProtectSystem = "no";
        ReadOnlyPaths = [ "/" ];
        ReadWritePaths = [
          cfg.data-dir
          cfg.tmp-dir
        ];
        RemoveIPC = true;
        RestrictNamespaces = false;
        RestrictRealtime = false;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        UMask = "0022";
      };
    };

    systemd.services."${manifest.name}-daemon" = {
      description = "Relago daemon";

      after = [
        "network.target"
        "${manifest.name}-config.service"
      ];
      requires = [ "${manifest.name}-config.service" ];
      wantedBy = [
        "multi-user.target"
      ];
      wants = [
        "network-online.target"
      ];
      path = [ fpkg ];

      serviceConfig = {
        Type = "dbus";
        BusName = "org.relago.DaemonService";
        # Type = "simple";

        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        ExecStart = "${lib.getBin fpkg}/bin/relago daemon";
        ExecReload = "${pkgs.coreutils}/bin/kill -s HUP $MAINPID";
        ReadWritePaths = [
          cfg.data-dir
          cfg.tmp-dir
        ];

        StateDirectory = cfg.user;
        StateDirectoryMode = "0755";

        PrivateDevices = false;
        PrivateTmp = true;
        PrivateUsers = false;
        ProtectClock = false;
        ProtectControlGroups = false;
        ProtectHome = false;
        ProtectHostname = false;
        ProtectKernelLogs = false;
        ProtectKernelTunables = false;
        ProtectSystem = "no";
        ReadOnlyPaths = [ "/" ];
        RemoveIPC = true;
        RestrictNamespaces = false;
        RestrictRealtime = false;
        RestrictSUIDSGID = true;
        SystemCallArchitectures = "native";
        UMask = "0022";
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
