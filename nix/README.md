# NixOS Module for COSMIC Ext Notifications

This directory contains the NixOS module for integrating `cosmic-ext-notifications` with your NixOS system.

## Features

- **Drop-in replacement** for the default COSMIC notifications daemon
- **Declarative configuration** via NixOS options
- **Systemd hardening** with proper security restrictions
- **DBus integration** with automatic service registration
- **COSMIC session integration** with proper lifecycle management

## Quick Start

### Using the Flake

Add this flake to your NixOS configuration:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    cosmic-ext-notifications = {
      url = "github:username/cosmic-ext-notifications";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, cosmic-ext-notifications, ... }: {
    nixosConfigurations.your-hostname = nixpkgs.lib.nixosSystem {
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

### Importing Directly

If you're not using flakes, import the module directly:

```nix
{ config, pkgs, ... }:

{
  imports = [
    /path/to/cosmic-ext-notifications/nix/module.nix
  ];

  services.desktopManager.cosmic.enable = true;
  services.cosmic-ext-notifications.enable = true;
}
```

## Configuration Options

### Basic Configuration

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;

    settings = {
      show_images = true;
      show_actions = true;
      max_image_size = 128;
      enable_links = true;
      enable_animations = true;
    };
  };
}
```

### Custom Package

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;
    package = pkgs.cosmic-ext-notifications.override {
      enableSystemd = true;
    };
  };
}
```

### Privacy-Focused Configuration

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;

    settings = {
      show_images = false;          # Don't load external images
      enable_links = false;          # Disable clickable links
      enable_animations = false;     # Static images only
      show_actions = true;           # Keep action buttons
    };
  };
}
```

### Performance-Optimized Configuration

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;

    settings = {
      max_image_size = 64;          # Smaller images
      enable_animations = false;     # No animations
      show_images = true;
      show_actions = true;
      enable_links = true;
    };
  };
}
```

## Available Options

### `services.cosmic-ext-notifications.enable`
**Type:** `boolean`
**Default:** `false`

Enable the COSMIC Notifications NG daemon.

### `services.cosmic-ext-notifications.package`
**Type:** `package`
**Default:** `pkgs.cosmic-ext-notifications`

The package to use for cosmic-ext-notifications.

### `services.cosmic-ext-notifications.settings`
**Type:** `attribute set`
**Default:** `{}`

Configuration settings for cosmic-ext-notifications.

#### `settings.show_images`
**Type:** `boolean`
**Default:** `true`

Show images in notifications. Supports `image-path` and `image-data` hints.

#### `settings.show_actions`
**Type:** `boolean`
**Default:** `true`

Show action buttons in notifications.

#### `settings.max_image_size`
**Type:** `positive integer`
**Default:** `128`

Maximum image size in pixels. Images are automatically resized.

#### `settings.enable_links`
**Type:** `boolean`
**Default:** `true`

Enable clickable HTTP/HTTPS links in notification body text.

#### `settings.enable_animations`
**Type:** `boolean`
**Default:** `true`

Enable GIF/APNG/WebP animations (max 100 frames, 30s duration).

### `services.cosmic-ext-notifications.replaceSystemPackage`
**Type:** `boolean`
**Default:** `true`

Replace the system `cosmic-notifications` package with `cosmic-ext-notifications` via overlay.

## Security

The module implements comprehensive systemd hardening:

- **Filesystem Protection:**
  - `ProtectSystem = "strict"` - Read-only system directories
  - `ProtectHome = true` - No access to user home directories
  - `PrivateTmp = true` - Isolated temporary directory

- **Process Isolation:**
  - `NoNewPrivileges = true` - Cannot gain privileges
  - `RestrictSUIDSGID = true` - Cannot create SUID/SGID files
  - `LockPersonality = true` - Cannot change execution domain

- **Kernel Protection:**
  - `ProtectKernelTunables = true` - Read-only kernel tunables
  - `ProtectControlGroups = true` - Read-only cgroup hierarchy
  - `RestrictRealtime = true` - No realtime scheduling

- **Resource Limits:**
  - `MemoryMax = "512M"` - Maximum 512MB memory
  - `TasksMax = 256` - Maximum 256 tasks

- **System Call Filtering:**
  - `SystemCallFilter = "@system-service ~@privileged"` - Limited syscalls
  - `CapabilityBoundingSet = ""` - No capabilities

## Troubleshooting

### Check Service Status

```bash
systemctl --user status cosmic-ext-notifications
```

### View Logs

```bash
journalctl --user -u cosmic-ext-notifications -f
```

### Test Notification

```bash
notify-send "Test" "This is a test notification"
```

### Verify DBus Registration

```bash
dbus-send --session --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.ListNames | grep Notifications
```

### Configuration File Location

The generated configuration file is located at:
```
~/.config/cosmic-ext-notifications/config.toml
```

## Integration with COSMIC

The module automatically integrates with the COSMIC session:

- **Session Lifecycle:** Starts with `cosmic-session.target`
- **DBus Activation:** Automatically registered with session bus
- **Package Replacement:** Can replace system `cosmic-notifications` transparently

## Migration from Default COSMIC Notifications

1. Enable the module with `replaceSystemPackage = true` (default)
2. Rebuild your system: `sudo nixos-rebuild switch`
3. Log out and log back in to COSMIC
4. Verify with: `systemctl --user status cosmic-ext-notifications`

No additional steps needed - the overlay ensures COSMIC uses `cosmic-ext-notifications` automatically.

## Advanced Usage

### Override Without Full Replacement

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;
    replaceSystemPackage = false;  # Don't create overlay
  };

  # Manually add to packages
  environment.systemPackages = with pkgs; [
    cosmic-ext-notifications
  ];
}
```

### Multiple Monitor Configuration

```nix
{
  services.cosmic-ext-notifications = {
    enable = true;
    settings = {
      # Configuration is per-user and respects COSMIC panel settings
      show_images = true;
    };
  };
}
```

## Contributing

To improve this module:

1. Test changes with `nixos-rebuild test`
2. Verify systemd hardening with `systemd-analyze security cosmic-ext-notifications`
3. Check for warnings: review NixOS build output
4. Submit pull request with documentation updates

## License

This module follows the same license as cosmic-ext-notifications.
