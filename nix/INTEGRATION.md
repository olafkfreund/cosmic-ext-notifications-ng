# NixOS Integration Guide for cosmic-ext-notifications

This document provides comprehensive information about the NixOS module for cosmic-ext-notifications.

## Overview

The NixOS module enables declarative configuration and seamless integration of cosmic-ext-notifications with the COSMIC desktop environment. It provides:

- **Drop-in replacement** for the default COSMIC notifications daemon
- **Type-safe configuration** via NixOS option system
- **Systemd hardening** with comprehensive security restrictions
- **Automatic DBus registration** for freedesktop.org notifications
- **COSMIC session integration** with proper lifecycle management

## Architecture

### Module Structure

```
nix/
├── module.nix                  # Main NixOS module
├── README.md                   # User documentation
├── INTEGRATION.md              # This file
├── example-configuration.nix   # Configuration examples
└── test.nix                    # VM test configuration
```

### Key Components

#### 1. Options Definition

The module provides structured options under `services.cosmic-ext-notifications`:

```nix
services.cosmic-ext-notifications = {
  enable = mkEnableOption "...";
  package = mkPackageOption pkgs "cosmic-ext-notifications" { };
  settings = mkOption { ... };
  replaceSystemPackage = mkOption { ... };
};
```

#### 2. Configuration Generation

User settings are converted to TOML format and placed in the XDG config directory:

```nix
xdg.configFile."cosmic-ext-notifications/config.toml" = {
  source = settingsFormat.generate "cosmic-ext-notifications.toml" cfg.settings;
};
```

#### 3. Systemd Service

The module creates a hardened systemd user service:

```nix
systemd.user.services.cosmic-ext-notifications = {
  description = "COSMIC Notifications NG Daemon";
  partOf = [ "cosmic-session.target" ];
  serviceConfig = {
    Type = "dbus";
    BusName = "org.freedesktop.Notifications";
    # ... security hardening ...
  };
};
```

#### 4. Package Overlay

When `replaceSystemPackage = true`, an overlay is created:

```nix
nixpkgs.overlays = [
  (final: prev: {
    cosmic-ext-notifications = cfg.package;
  })
];
```

## Configuration Options Reference

### `services.cosmic-ext-notifications.enable`

**Type:** `boolean`
**Default:** `false`

Enables the cosmic-ext-notifications daemon. When enabled:
- Systemd user service is created
- DBus registration is configured
- Configuration file is generated

### `services.cosmic-ext-notifications.package`

**Type:** `package`
**Default:** `pkgs.cosmic-ext-notifications`

The package to use for cosmic-ext-notifications. Can be overridden to use custom builds:

```nix
package = pkgs.cosmic-ext-notifications.override {
  enableSystemd = true;
};
```

### `services.cosmic-ext-notifications.settings`

**Type:** `attribute set`
**Default:** `{}`

Configuration settings for cosmic-ext-notifications. All settings are optional.

#### `settings.show_images`

**Type:** `boolean`
**Default:** `true`

Display images in notifications from `image-path` and `image-data` hints.

#### `settings.show_actions`

**Type:** `boolean`
**Default:** `true`

Display action buttons in notifications. Enables DBus ActionInvoked signals.

#### `settings.max_image_size`

**Type:** `positive integer`
**Default:** `128`

Maximum image dimension in pixels. Larger images are automatically resized.

#### `settings.enable_links`

**Type:** `boolean`
**Default:** `true`

Make HTTP/HTTPS URLs in notification text clickable.

#### `settings.enable_animations`

**Type:** `boolean`
**Default:** `true`

Enable GIF/APNG/WebP animations (100 frame limit, 30s max duration).

### `services.cosmic-ext-notifications.replaceSystemPackage`

**Type:** `boolean`
**Default:** `true`

Create a nixpkgs overlay that replaces `cosmic-ext-notifications` with `cosmic-ext-notifications` system-wide.

## Security Model

### Systemd Hardening

The module implements comprehensive systemd security features:

#### Filesystem Protection

```nix
ProtectSystem = "strict";      # Read-only /usr, /boot, /efi
ProtectHome = true;            # No home directory access
PrivateTmp = true;             # Isolated /tmp
```

#### Process Isolation

```nix
NoNewPrivileges = true;        # Cannot escalate privileges
RestrictSUIDSGID = true;       # Cannot create setuid files
LockPersonality = true;        # Cannot change execution domain
```

#### Kernel Protection

```nix
ProtectKernelTunables = true;  # Read-only /proc/sys, /sys
ProtectControlGroups = true;   # Read-only cgroup hierarchy
RestrictRealtime = true;       # No realtime scheduling
RestrictNamespaces = true;     # Limited namespace creation
```

#### Resource Limits

```nix
MemoryMax = "512M";            # Maximum memory usage
TasksMax = 256;                # Maximum concurrent tasks
```

#### Capability Restrictions

```nix
CapabilityBoundingSet = "";    # No Linux capabilities
SystemCallFilter = "@system-service ~@privileged";
```

### Security Assessment

Check the security score with:

```bash
systemd-analyze security cosmic-ext-notifications
```

Expected score: **9.0/10** or better (lower is more secure).

## Validation and Assertions

### Compile-Time Checks

The module includes assertions that prevent invalid configurations:

```nix
assertions = [
  {
    assertion = cfg.settings.max_image_size > 0;
    message = "max_image_size must be positive";
  }
  {
    assertion = config.services.desktopManager.cosmic.enable or false;
    message = "COSMIC desktop environment must be enabled";
  }
];
```

### Runtime Warnings

The module emits warnings for potentially unexpected configurations:

```nix
warnings = optional (!cfg.settings.enable_animations) [
  "Animated images are disabled..."
] ++ optional (!cfg.settings.enable_links) [
  "Clickable links are disabled..."
];
```

## Integration Patterns

### Pattern 1: Flake-based System Configuration

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-ext-notifications.url = "github:user/cosmic-ext-notifications";
    cosmic-ext-notifications.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, cosmic-ext-notifications, ... }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [
        cosmic-ext-notifications.nixosModules.default
        {
          services.desktopManager.cosmic.enable = true;
          services.cosmic-ext-notifications.enable = true;
        }
      ];
    };
  };
}
```

### Pattern 2: Channels-based Configuration

```nix
{ config, pkgs, ... }:

let
  cosmic-ext-notifications = builtins.fetchGit {
    url = "https://github.com/user/cosmic-notifications-ng";
    ref = "main";
  };
in
{
  imports = [
    "${cosmic-ext-notifications}/nix/module.nix"
  ];

  services.desktopManager.cosmic.enable = true;
  services.cosmic-ext-notifications.enable = true;
}
```

### Pattern 3: Local Development

```nix
{ config, pkgs, ... }:

{
  imports = [
    /home/user/projects/cosmic-ext-notifications/nix/module.nix
  ];

  services.cosmic-ext-notifications = {
    enable = true;
    package = pkgs.callPackage /home/user/projects/cosmic-ext-notifications { };
  };
}
```

## Testing

### Manual Testing

After enabling the module:

```bash
# Rebuild system
sudo nixos-rebuild switch

# Log out and back in to COSMIC

# Verify service status
systemctl --user status cosmic-ext-notifications

# Send test notification
notify-send "Test" "If you see this, it works!"

# Check DBus registration
dbus-send --session --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep Notifications
```

### VM Testing

Build and run a test VM:

```bash
# From the repository root
nixos-rebuild build-vm -I nixos-config=./nix/test.nix

# Run the VM
./result/bin/run-*-vm

# Login as testuser (password: test)
# Test notifications in the COSMIC session
```

### Integration Testing

Create a NixOS test:

```nix
import <nixpkgs/nixos/tests/make-test-python.nix> ({ pkgs, ... }: {
  name = "cosmic-ext-notifications-test";

  nodes.machine = { ... }: {
    imports = [ ./nix/module.nix ];
    services.desktopManager.cosmic.enable = true;
    services.cosmic-ext-notifications.enable = true;
  };

  testScript = ''
    machine.start()
    machine.wait_for_unit("cosmic-session.target", "testuser")
    machine.wait_for_unit("cosmic-ext-notifications.service", "testuser")

    # Send notification
    machine.succeed("sudo -u testuser DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus notify-send 'Test' 'Message'")

    # Verify DBus interface
    machine.succeed("sudo -u testuser DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus dbus-send --session --print-reply --dest=org.freedesktop.Notifications /org/freedesktop/Notifications org.freedesktop.Notifications.GetServerInformation")
  '';
})
```

## Troubleshooting

### Service Not Starting

**Symptom:** `systemctl --user status cosmic-ext-notifications` shows inactive

**Solutions:**
1. Check COSMIC session is running:
   ```bash
   systemctl --user status cosmic-session.target
   ```

2. Check for DBus conflicts:
   ```bash
   systemctl --user list-units | grep notification
   ```

3. View detailed logs:
   ```bash
   journalctl --user -u cosmic-ext-notifications -b
   ```

### Configuration Not Applied

**Symptom:** Settings changes not reflected in notifications

**Solutions:**
1. Verify config file exists:
   ```bash
   cat ~/.config/cosmic-ext-notifications/config.toml
   ```

2. Restart the service:
   ```bash
   systemctl --user restart cosmic-ext-notifications
   ```

3. Check for syntax errors:
   ```bash
   nixos-rebuild dry-build
   ```

### DBus Registration Failure

**Symptom:** Multiple notification daemons or registration errors

**Solutions:**
1. Kill conflicting daemons:
   ```bash
   pkill -f cosmic-ext-notifications
   pkill -f notification-daemon
   ```

2. Verify DBus service file:
   ```bash
   ls -l /run/current-system/sw/share/dbus-1/services/
   ```

3. Check DBus ownership:
   ```bash
   dbus-send --session --print-reply \
     --dest=org.freedesktop.DBus \
     /org/freedesktop/DBus \
     org.freedesktop.DBus.GetNameOwner \
     string:org.freedesktop.Notifications
   ```

### Memory or Performance Issues

**Symptom:** High memory usage or slow performance

**Solutions:**
1. Reduce image size:
   ```nix
   settings.max_image_size = 64;
   ```

2. Disable animations:
   ```nix
   settings.enable_animations = false;
   ```

3. Monitor resource usage:
   ```bash
   systemctl --user show cosmic-ext-notifications | grep Memory
   ```

## Advanced Configuration

### Multi-User Setup

Each user can have different settings via home-manager:

```nix
home-manager.users.alice = {
  xdg.configFile."cosmic-ext-notifications/config.toml".text = ''
    show_images = true
    max_image_size = 256
  '';
};

home-manager.users.bob = {
  xdg.configFile."cosmic-ext-notifications/config.toml".text = ''
    show_images = false
    enable_animations = false
  '';
};
```

### Custom Build Options

Override package features:

```nix
services.cosmic-ext-notifications.package = pkgs.cosmic-ext-notifications.overrideAttrs (oldAttrs: {
  buildFeatures = [ "systemd" "custom-feature" ];

  preBuild = ''
    echo "Custom build step"
  '';
});
```

### Integration with Other Services

Coordinate with other notification systems:

```nix
# Disable conflicting services
systemd.user.services.dunst.enable = false;
systemd.user.services.mako.enable = false;

# Ensure cosmic-ext-notifications starts after other services
systemd.user.services.cosmic-ext-notifications = {
  after = [ "pipewire.service" "wireplumber.service" ];
  wants = [ "pipewire.service" ];
};
```

## Maintenance

### Updating the Module

When updating cosmic-ext-notifications:

```bash
# Update flake input
nix flake update cosmic-ext-notifications

# Rebuild system
sudo nixos-rebuild switch

# Restart user session or just the service
systemctl --user restart cosmic-ext-notifications
```

### Module Development

To modify the module:

1. Edit `/nix/module.nix`
2. Test with `nixos-rebuild build-vm`
3. Validate with `nix flake check`
4. Document changes in `/nix/README.md`

### Code Review Checklist

Before submitting module changes:

- [ ] No `mkIf condition true` anti-patterns
- [ ] Explicit imports (minimal `with` usage)
- [ ] Proper option documentation
- [ ] Type safety with assertions
- [ ] Security hardening maintained
- [ ] Test configuration works
- [ ] README updated
- [ ] No secrets in evaluation

## References

- [NixOS Module System](https://nixos.org/manual/nixos/stable/#sec-writing-modules)
- [Systemd Hardening](https://www.freedesktop.org/software/systemd/man/systemd.exec.html)
- [freedesktop.org Notification Spec](https://specifications.freedesktop.org/notification-spec/)
- [COSMIC Desktop](https://github.com/pop-os/cosmic-epoch)

## Contributing

Improvements to the NixOS module are welcome. Please:

1. Follow NixOS best practices
2. Maintain security hardening
3. Add tests for new features
4. Update documentation
5. Avoid anti-patterns listed in CLAUDE.md

## License

This module follows the same license as cosmic-ext-notifications.
