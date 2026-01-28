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
    # users.users = lib.mkIf (cfg.user == manifest.name) {
    #   ${manifest.name} = {
    #     description = "${manifest.name} Service";
    #     home = cfg.dataDir;
    #     useDefaultShell = true;
    #     group = cfg.group;
    #     isSystemUser = true;
    #   };
    # };

    ## Group to join our user
    # users.groups = mkIf (cfg.group == manifest.name) {
    #   ${manifest.name} = {};
    # };

    systemd.services."${manifest.name}" = {
      description = "${manifest.name} Relago daemon";
      wantedBy = ["multi-user.target"];
      
      serviceConfig = {
        Type = "dbus";
        BusName = "org.freedesktop.problems.daemon";
        
        ExecStart = "${lib.getBin fpkg}/bin/relago";
        
        StandardInput = "null";
        StandardOutput = "journal";
        StandardError = "journal";
        
        # Restart = "always";

        DevicePolicy="closed";
        KeyringMode="private";
        LockPersonality="yes";
        MemoryDenyWriteExecute="yes";
        NoNewPrivileges="yes";
        PrivateDevices="yes";
        PrivateTmp="true";
        ProtectClock="yes";
        ProtectControlGroups="yes";
        ProtectHome="read-only";
        ProtectHostname="yes";
        ProtectKernelLogs="yes";
        ProtectKernelModules="yes";
        ProtectKernelTunables="yes";
        ProtectProc="invisible";
        ProtectSystem="full";
        RestrictNamespaces="yes";
        RestrictRealtime="yes";
        RestrictSUIDSGID="yes";
        SystemCallArchitectures="native";
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

      threads = mkOption {
        type = types.int;
        default = 1;
        description = "How many cores to use while pooling";
      };

      dataDir = mkOption {
        type = types.str;
        default = "/var/lib/${manifest.name}";
        description = lib.mdDoc ''
          The path where ${manifest.name} keeps its config, data, and logs.
        '';
      };

    };
  };

  config = mkMerge [service];
}
