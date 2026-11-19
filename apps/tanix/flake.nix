{
  description = "Tanix: a custom Linux distro built on NixOS";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      mkIso =
        module: system:
        nixpkgs.lib.nixosSystem {
          modules = [
            module
            { nixpkgs.hostPlatform = system; }
          ];
        };
    in
    {
      # Branding module, also copied onto installed systems by the installer.
      nixosModules.tanix = ./modules/tanix;

      # Overlay that turns the NixOS Calamares installer into the Tanix one.
      overlays.installer = import ./installer;

      nixosConfigurations = {
        iso-aarch64 = mkIso ./modules/iso.nix "aarch64-linux";
        iso-x86_64 = mkIso ./modules/iso.nix "x86_64-linux";
        # Tanix Server: small text installer with a role checklist.
        server-iso-aarch64 = mkIso ./modules/iso-server.nix "aarch64-linux";
        server-iso-x86_64 = mkIso ./modules/iso-server.nix "x86_64-linux";
      };

      # `nix build .#iso` builds the live/installer ISO for the build machine.
      packages = forAllSystems (
        system:
        let
          arch = nixpkgs.lib.head (nixpkgs.lib.splitString "-" system);
          isoSystem = self.nixosConfigurations."iso-${arch}";
        in
        {
          iso = isoSystem.config.system.build.isoImage;
          # `nix build .#iso-server`: the Tanix Server ISO.
          iso-server = self.nixosConfigurations."server-iso-${arch}".config.system.build.isoImage;
          # The patched installer files on their own, for quick inspection.
          installer-extensions = isoSystem.pkgs.calamares-nixos-extensions;
          default = self.packages.${system}.iso;
        }
      );

      formatter = forAllSystems (system: nixpkgs.legacyPackages.${system}.nixfmt-rfc-style);
    };
}
