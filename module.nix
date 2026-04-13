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

    systemd.tmpfiles.rules = [
      "d ${cfg.data-dir} 0770 ${cfg.user} ${cfg.group} -"
      "d ${cfg.tmp-dir}  0770 ${cfg.user} ${cfg.group} -"
    ];

    systemd.targets."${manifest.name}" = { };

    systemd.services."${manifest.name}-config" = {
      wantedBy = [ "multi-user.target" ];
      after = [ "systemd-tmpfiles-setup.service" ];
      requires = [ "systemd-tmpfiles-setup.service" ];

      serviceConfig = {
        Type = "oneshot";
        User = cfg.user;
        Group = cfg.group;
        Restart = "on-failure";
        RestartSec = "2s";
        RemainAfterExit = true;

        ReadWritePaths = [
          cfg.data-dir
        ];

        ExecStartPre =
          let
            preStartFullPrivileges = ''
              set -o errexit -o pipefail -o nounset
              mkdir -p ${cfg.data-dir} ${cfg.tmp-dir}
              ${pkgs.coreutils}/bin/install -d -m 0770 -o ${cfg.user} -g ${cfg.group} ${cfg.data-dir}
              ${pkgs.coreutils}/bin/install -d -m 0770 -o ${cfg.user} -g ${cfg.group} ${cfg.tmp-dir}
            '';
          in
          "+${pkgs.writeShellScript "${manifest.name}-pre-start-full-privileges" preStartFullPrivileges}";

        ExecStart = pkgs.writeShellScript "${manifest.name}-config" ''
          set -o errexit -o pipefail -o nounset
          shopt -s inherit_errexit
          umask u=rwx,g=rx,o=
          ${pkgs.coreutils}/bin/install -m 0640 -o ${cfg.user} -g ${cfg.group} \
            ${toml-config} ${cfg.data-dir}/config.toml
        '';
      };
    };

    systemd.user.services."${manifest.name}-gui" = {
      description = "Relago GNOME UI Agent";
      wantedBy = [ "default.target" ];
      partOf = [ "graphical-session.target" ];
      after = [
        "graphical-session.target"
        "network.target"
        "dbus.socket"
      ];
      restartTriggers = [ fpkg ];

      serviceConfig = {
        # Type = "dbus";
        # BusName = "org.relago.ReportService";
        Type = "simple";
        ExecStart = "${lib.getBin fpkg}/bin/relago gui";
        Restart = "on-failure";
        RestartSec = 5;
        TimeoutStartSec = 30;

        ReadWritePaths = [
          cfg.data-dir
          cfg.tmp-dir
        ];

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
        # FIXME: change to whatever you think needeed
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
      restartTriggers = [
        fpkg
        toml-config
      ];

      serviceConfig = {
        Type = "dbus";
        BusName = "org.relago.DaemonService";
        # Type = "simple";

        User = cfg.user;
        Group = cfg.group;
        Restart = "on-failure";
        RestartSec = "5s";
        ExecStart = "${lib.getBin fpkg}/bin/relago daemon";
        ExecReload = "${pkgs.coreutils}/bin/kill -s HUP $MAINPID";
        ReadWritePaths = [
          cfg.data-dir
          cfg.tmp-dir
        ];

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
        # FIXME: check UMask later
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
