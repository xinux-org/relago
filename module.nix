flake:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  inherit (lib)
    mkEnableOption
    mkOption
    mkIf
    mkMerge
    types
    ;

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
    tmp_dir = cfg.data-dir;
    nix_config = cfg.nix-config;
    problems_interface = cfg.problems-interface;
  };

  # Systemd services
  service = mkIf cfg.enable {
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
              set -o errexit -o pipefail -o nounset
              shopt -s dotglob nullglob inherit_errexit

              chown -R --no-dereference '${cfg.user}':'${cfg.group}' '${cfg.data-dir}'
              chmod -R u+rwX,g+rX,o-rwx '${cfg.data-dir}'
            '';
          in
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

    systemd.services."${manifest.name}" = {
      description = "${manifest.name} Relago daemon";

      after = [
        "network-online.target"
        "relago-daemon-config.service"
      ];
      wants = [ "network-online.target" ];
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Type = "dbus";
        BusName = "org.freedesktop.problems.daemon";

        ExecStart = "${lib.getBin fpkg}/bin/relago";

        StandardInput = "null";
        StandardOutput = "journal";
        StandardError = "journal";

        # Restart = "always";

        DevicePolicy = "closed";
        KeyringMode = "private";
        LockPersonality = "yes";
        MemoryDenyWriteExecute = "yes";
        NoNewPrivileges = "yes";
        PrivateDevices = "yes";
        PrivateTmp = "true";
        ProtectClock = "yes";
        ProtectControlGroups = "yes";
        ProtectHome = "read-only";
        ProtectHostname = "yes";
        ProtectKernelLogs = "yes";
        ProtectKernelModules = "yes";
        ProtectKernelTunables = "yes";
        ProtectProc = "invisible";
        ProtectSystem = "full";
        RestrictNamespaces = "yes";
        RestrictRealtime = "yes";
        RestrictSUIDSGID = "yes";
        SystemCallArchitectures = "native";
      };
    };
  };

in
{
  # Available user options
  options = with lib; {
    services.${manifest.name} = {
      enable = mkEnableOption ''
        ${manifest.name}, actix + diesel server on rust.
      '';

      parallel-compression = mkOption {
        type = types.int;
        default = 4;
        example = 4;
        description = "How many cores to use while pooling";
      };

      data-dir = mkOption {
        type = types.str;
        default = "/var/lib/${manifest.name}/tmp/";
        example = "/var/lib/${manifest.name}/tmp/";
        description = "Temp folder for Relago";
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
        description = "Relago-support server";
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

  config = mkMerge [ service ];
}
