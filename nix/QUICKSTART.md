# Quick Start Guide - cosmic-ext-notifications NixOS Module

## 5-Minute Setup

### 1. Add to Your Flake

```nix
{
  inputs = {
    cosmic-ext-notifications.url = "github:username/cosmic-ext-notifications";
    cosmic-ext-notifications.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, cosmic-ext-notifications, ... }: {
    nixosConfigurations.your-host = nixpkgs.lib.nixosSystem {
      modules = [
        cosmic-ext-notifications.nixosModules.default
        ./configuration.nix
      ];
    };
  };
}
```

### 2. Enable in Configuration

```nix
# configuration.nix
{
  services.desktopManager.cosmic.enable = true;
  services.cosmic-ext-notifications.enable = true;
}
```

### 3. Apply Changes

```bash
sudo nixos-rebuild switch
# Log out and back in to COSMIC
```

### 4. Verify

```bash
notify-send "Success!" "cosmic-ext-notifications is working"
```

## Common Configurations

### Default (Recommended)

```nix
services.cosmic-ext-notifications.enable = true;
```

All features enabled with sensible defaults.

### Privacy Mode

```nix
services.cosmic-ext-notifications = {
  enable = true;
  settings = {
    show_images = false;
    enable_links = false;
    enable_animations = false;
  };
};
```

No external content loading.

### Performance Mode

```nix
services.cosmic-ext-notifications = {
  enable = true;
  settings = {
    max_image_size = 64;
    enable_animations = false;
  };
};
```

Minimal resource usage.

### Feature Complete

```nix
services.cosmic-ext-notifications = {
  enable = true;
  settings = {
    show_images = true;
    show_actions = true;
    max_image_size = 256;
    enable_links = true;
    enable_animations = true;
  };
};
```

All bells and whistles.

## Troubleshooting

### Service not running?

```bash
systemctl --user status cosmic-ext-notifications
journalctl --user -u cosmic-ext-notifications -f
```

### Notifications not showing?

```bash
# Test DBus
dbus-send --session --print-reply \
  --dest=org.freedesktop.Notifications \
  /org/freedesktop/Notifications \
  org.freedesktop.Notifications.GetServerInformation
```

### Wrong daemon running?

```bash
# Check what owns the DBus name
dbus-send --session --print-reply \
  --dest=org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus.GetNameOwner \
  string:org.freedesktop.Notifications
```

## Next Steps

- Read full documentation: `nix/README.md`
- View configuration examples: `nix/example-configuration.nix`
- Check integration details: `nix/INTEGRATION.md`
- Test in VM: `nixos-rebuild build-vm -I nixos-config=./nix/test.nix`

## Quick Reference

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enable` | bool | false | Enable the service |
| `settings.show_images` | bool | true | Display images |
| `settings.show_actions` | bool | true | Show action buttons |
| `settings.max_image_size` | int | 128 | Max image pixels |
| `settings.enable_links` | bool | true | Clickable URLs |
| `settings.enable_animations` | bool | true | GIF/APNG/WebP |
| `replaceSystemPackage` | bool | true | Replace system daemon |

## Support

- Issues: GitHub issue tracker
- Logs: `journalctl --user -u cosmic-ext-notifications`
- Status: `systemctl --user status cosmic-ext-notifications`
- Config: `~/.config/cosmic-ext-notifications/config.toml`
