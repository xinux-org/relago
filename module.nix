flake: {
  config,
  lib,
  pkgs,
  ...
}: let
  inherit (lib) mkEnableOption mkOption mkIf mkMerge types;

  # Manifest via Cargo.toml
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;

  # Options
  cfg = config.services.${manifest.name};

  # Flake shipped default binary
  fpkg = flake.packages.${pkgs.stdenv.hostPlatform.system}.default;

  # Toml management
  toml = pkgs.formats.toml {};


  # The digesting configuration of server
  toml-config = toml.generate "config.toml" {
    port = cfg.port;
    url = cfg.address;
    threads = cfg.threads;
    database_url = "#databaseUrl#";
  };


  # Systemd services
  service = mkIf cfg.enable {
    ## User for our services
    users.users = lib.mkIf (cfg.user == manifest.name) {
      ${manifest.name} = {
        description = "${manifest.name} Service";
        home = cfg.dataDir;
        useDefaultShell = true;
        group = cfg.group;
        isSystemUser = true;
      };
    };

    ## Group to join our user
    users.groups = mkIf (cfg.group == manifest.name) {
      ${manifest.name} = {};
    };


    ## Main server service
    systemd.services."${manifest.name}" = {
      description = "${manifest.name} Relago daemonr";
      documentation = [manifest.homepage];

      # after = ["network.target" "${manifest.name}-config.service" "${manifest.name}-migration.service"] ++ lib.optional local-database "postgresql.service";
      # requires = lib.optional local-database "postgresql.service";
      wants = ["network-online.target"];
      wantedBy = ["multi-user.target"];
      path = [cfg.package];

      serviceConfig = {
        User = cfg.user;
        Group = cfg.group;
        Restart = "always";
        ExecStart = "${lib.getBin cfg.package}/bin/relago daemon";
        ExecReload = "${pkgs.coreutils}/bin/kill -s HUP $MAINPID";
        StateDirectory = cfg.user;
        StateDirectoryMode = "0750";
        Type = "dbus";
        # Access write directories
        # ReadWritePaths = [cfg.dataDir "/run/postgresql"];
        CapabilityBoundingSet = [
          "AF_NETLINK"
          "AF_INET"
          "AF_INET6"
        ];
        DeviceAllow = ["/dev/stdin r"];
        DevicePolicy = "strict";
        IPAddressAllow = "localhost";
        LockPersonality = true;
        NoNewPrivileges = true;
        PrivateDevices = true;
        PrivateTmp = true;
        PrivateUsers = false;
        ProtectClock = true;
        ProtectControlGroups = true;
        ProtectHome = true;
        ProtectHostname = true;
        ProtectKernelLogs = true;
        ProtectKernelModules = true;
        ProtectKernelTunables = true;
        ProtectSystem = "strict";
        ReadOnlyPaths = ["/"];
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
      };
    };
  };

in {
  # Available user options
  options = with lib; {
    services.${manifest.name} = {
      enable = mkEnableOption ''
        ${manifest.name}, actix + diesel server on rust.
      '';

      address = mkOption {
        type = types.str;
        default = "127.0.0.1";
        description = "Port to use for passing over proxy";
      };

   

      threads = mkOption {
        type = types.int;
        default = 1;
        description = "How many cores to use while pooling";
      };

      proxy-reverse = {
        enable = mkEnableOption ''
          Enable proxy reversing via nginx/caddy.
        '';
      };


      user = mkOption {
        type = types.str;
        default = "${manifest.name}";
        description = "User for running system + accessing keys";
      };

      group = mkOption {
        type = types.str;
        default = "${manifest.name}";
        description = "Group for running system + accessing keys";
      };

      dataDir = mkOption {
        type = types.str;
        default = "/var/lib/${manifest.name}";
        description = lib.mdDoc ''
          The path where ${manifest.name} keeps its config, data, and logs.
        '';
      };

      package = mkOption {
        type = types.package;
        default = fpkg;
        description = ''
          Compiled ${manifest.name} actix server package to use with the service.
        '';
      };
    };
  };

  config = mkMerge [service];
}
