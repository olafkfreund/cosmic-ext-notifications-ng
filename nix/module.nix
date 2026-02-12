{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.cosmic-ext-notifications;

  settingsFormat = pkgs.formats.toml { };

  configFile = settingsFormat.generate "cosmic-ext-notifications.toml" cfg.settings;
in
{
  options.services.cosmic-ext-notifications = {
    enable = mkEnableOption "COSMIC Notifications NG daemon as a replacement for the default COSMIC notifications";

    package = mkPackageOption pkgs "cosmic-ext-notifications" {
      default = [ "cosmic-ext-notifications" ];
      example = literalExpression "pkgs.cosmic-ext-notifications.override { enableSystemd = true; }";
    };

    settings = mkOption {
      type = types.submodule {
        freeformType = settingsFormat.type;

        options = {
          show_images = mkOption {
            type = types.bool;
            default = true;
            description = ''
              Show images in notifications.

              Supports image-path and image-data hints from the freedesktop.org
              Notification Specification. Images are automatically resized.
            '';
          };

          show_actions = mkOption {
            type = types.bool;
            default = true;
            description = ''
              Show action buttons in notifications.

              Enables multiple action buttons per notification with proper
              DBus ActionInvoked signal emission.
            '';
          };

          max_image_size = mkOption {
            type = types.ints.positive;
            default = 128;
            example = 256;
            description = ''
              Maximum image size in pixels.

              Images larger than this will be automatically resized while
              preserving aspect ratio.
            '';
          };

          enable_links = mkOption {
            type = types.bool;
            default = true;
            description = ''
              Enable clickable links in notification body text.

              Automatically detects URLs and makes them clickable.
              Only http:// and https:// URLs are enabled for security.
            '';
          };

          enable_animations = mkOption {
            type = types.bool;
            default = true;
            description = ''
              Enable animated images in notifications.

              Supports GIF, APNG, and WebP animations with memory-safe
              limits (100 frames max, 30s max duration).
            '';
          };
        };
      };
      default = { };
      example = literalExpression ''
        {
          show_images = true;
          show_actions = true;
          max_image_size = 256;
          enable_links = true;
          enable_animations = false;
        }
      '';
      description = ''
        Configuration for COSMIC Notifications NG.

        See <https://github.com/cosmic-ext-notifications> for available options.
      '';
    };

    replaceSystemPackage = mkOption {
      type = types.bool;
      default = true;
      description = ''
        Replace the system cosmic-notifications package with cosmic-ext-notifications.

        This creates an overlay that substitutes the default COSMIC notifications
        daemon with this enhanced version across the entire system.
      '';
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = cfg.settings.max_image_size > 0;
        message = "services.cosmic-ext-notifications.settings.max_image_size must be positive";
      }
    ];

    warnings = optional (!cfg.settings.enable_animations) [
      "Animated images are disabled in cosmic-ext-notifications. GIF/APNG/WebP animations will display as static images."
    ] ++ optional (!cfg.settings.enable_links) [
      "Clickable links are disabled in cosmic-ext-notifications. URLs in notifications will not be interactive."
    ];

    nixpkgs.overlays = mkIf cfg.replaceSystemPackage [
      (final: prev: {
        cosmic-notifications = cfg.package;
      })
    ];

    environment.systemPackages = mkIf (!cfg.replaceSystemPackage) [ cfg.package ];

    # Config setup script - runs before the service starts
    # Creates the config directory and writes the config file
    systemd.user.services.cosmic-ext-notifications = {
      description = "COSMIC Notifications NG Daemon";
      documentation = [ "https://github.com/cosmic-ext-notifications" ];

      partOf = [ "cosmic-session.target" ];
      after = [ "cosmic-session.target" ];
      requisite = [ "cosmic-session.target" ];

      serviceConfig = {
        Type = "dbus";
        BusName = "org.freedesktop.Notifications";
        ExecStartPre = mkIf (cfg.settings != { }) (
          let
            setupScript = pkgs.writeShellScript "cosmic-ext-notifications-setup" ''
              mkdir -p "$HOME/.config/cosmic-ext-notifications"
              cp -f ${configFile} "$HOME/.config/cosmic-ext-notifications/config.toml"
              chmod 644 "$HOME/.config/cosmic-ext-notifications/config.toml"
            '';
          in
          "${setupScript}"
        );
        ExecStart = "${cfg.package}/bin/cosmic-ext-notifications";
        Restart = "on-failure";
        RestartSec = 3;

        Slice = "session.slice";

        # Security hardening
        ProtectSystem = "strict";
        ProtectHome = "read-only";
        ReadWritePaths = [ "%h/.config/cosmic-ext-notifications" ];
        PrivateTmp = true;
        NoNewPrivileges = true;
        RestrictSUIDSGID = true;
        ProtectKernelTunables = true;
        ProtectControlGroups = true;
        RestrictNamespaces = true;
        LockPersonality = true;
        RestrictRealtime = true;

        MemoryMax = "512M";
        TasksMax = 256;

        CapabilityBoundingSet = "";
        SystemCallFilter = [ "@system-service" "~@privileged" ];
        SystemCallErrorNumber = "EPERM";
      };
    };

    services.dbus.packages = [ cfg.package ];

    meta.maintainers = with maintainers; [ ];
  };
}
