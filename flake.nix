{
  description = "Enhanced notifications daemon for COSMIC desktop with rich content support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, nix-filter, crane, fenix }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        # Use crane.mkLib (new API) with nixpkgs Rust toolchain
        craneLib = crane.mkLib pkgs;
        pkgDef = {
          src = nix-filter.lib.filter {
            root = ./.;
            exclude = [
              ./.gitignore
              ./flake.nix
              ./flake.lock
              ./LICENSE
              ./debian
              ./nix
            ];
          };
          nativeBuildInputs = with pkgs; [
            just
            pkg-config
            autoPatchelfHook
          ];
          buildInputs = with pkgs; [
            libxkbcommon
            wayland
            freetype
            fontconfig
            expat
            lld
            desktop-file-utils
            stdenv.cc.cc.lib
            desktop-file-utils
            # Audio support for notification sounds
            alsa-lib
           ];
          runtimeDependencies = with pkgs; [
            wayland
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly pkgDef;
        cosmic-notifications-daemon= craneLib.buildPackage (pkgDef // {
          inherit cargoArtifacts;
        });
      in {
        checks = {
          inherit cosmic-notifications-daemon;
        };

        packages.default = cosmic-notifications-daemon.overrideAttrs (oldAttrs: rec {
          buildPhase= ''
            just prefix=$out build-release
          '';
          installPhase = ''
            just prefix=$out install
          '';
        });

        apps.default = flake-utils.lib.mkApp {
          drv = cosmic-notifications-daemon;
        };

        devShells.default = pkgs.mkShell rec {
          inputsFrom = builtins.attrValues self.checks.${system};
          LD_LIBRARY_PATH = pkgs.lib.strings.makeLibraryPath (builtins.concatMap (d: d.runtimeDependencies) inputsFrom);
        };
      }) // {
        nixosModules = {
          default = import ./nix/module.nix;
          cosmic-notifications-ng = import ./nix/module.nix;
        };

        overlays = {
          default = final: prev: {
            cosmic-notifications = self.packages.${prev.system}.default;
            cosmic-notifications-ng = self.packages.${prev.system}.default;
          };
        };
      };

  nixConfig = {
    # Cache for the Rust toolchain in fenix
    extra-substituters = [ "https://nix-community.cachix.org" ];
    extra-trusted-public-keys = [ "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs=" ];
  };
}
