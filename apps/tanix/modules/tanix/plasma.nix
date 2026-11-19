# Tanix on KDE Plasma, beyond the layout in default.nix:
#
#   - Tanix wallpaper on the login screen (SDDM) and the lock screen
#   - the Tanix accent colour, on Plasma's stock Breeze theme
#   - the Tanix icon themes: "Tanix Minimal 3D" (default) and "Tanix Minimal"
#     (flat), both installed and switchable in System Settings > Icons
#   - a few extra everyday apps, and Flatpak with Flathub in Discover
#     (both left out of the Security Edition, see security.nix)
{
  config,
  lib,
  pkgs,
  ...
}:

let
  plasma = config.services.desktopManager.plasma6.enable;
  sddm = config.services.displayManager.sddm.enable;
  branding = config.tanix.branding;
  extras = config.tanix.desktop.extras;
  iconTheme = config.tanix.desktop.iconTheme;

  # Both Tanix icon themes. They cover the everyday apps and inherit Breeze
  # for every other icon. Theme folder names are the ids used in kdeglobals.
  iconThemes = pkgs.runCommand "tanix-icon-themes" { } ''
    mkdir -p $out/share/icons
    cp -r ${./assets/icons}/. $out/share/icons/
  '';

  # The Breeze login theme reads overrides from theme.conf.user next to it.
  sddmBackground = pkgs.writeTextDir "share/sddm/themes/breeze/theme.conf.user" ''
    [General]
    background=${branding.wallpaperPath}
    type=image
  '';
in
{
  options.tanix.desktop.iconTheme = lib.mkOption {
    type = lib.types.enum [
      "tanix-minimal-3d"
      "tanix-minimal"
      "breeze"
    ];
    default = "tanix-minimal-3d";
    description = ''
      Default icon theme on KDE Plasma: "tanix-minimal-3d" (Tanix Minimal 3D),
      "tanix-minimal" (Tanix Minimal, flat) or Plasma's stock "breeze". Both
      Tanix themes are always installed, so users can switch in System Settings.
    '';
  };

  options.tanix.desktop.extras = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Install the extra Tanix desktop apps (calculator, disk usage, partition manager).";
    };
    flatpak = lib.mkOption {
      type = lib.types.bool;
      default = extras.enable;
      defaultText = lib.literalExpression "config.tanix.desktop.extras.enable";
      description = "Enable Flatpak with the Flathub store, shown in Discover.";
    };
  };

  config = lib.mkIf (branding.enable && plasma) (
    lib.mkMerge [
      {
        environment.etc."xdg/kscreenlockerrc".text = ''
          [Greeter][Wallpaper][org.kde.image][General]
          Image=file://${branding.wallpaperPath}
        '';

        environment.etc."xdg/kdeglobals".text = ''
          [General]
          AccentColor=${branding.accentColor}

          [Icons]
          Theme=${iconTheme}
        '';

        environment.systemPackages = [ iconThemes ];
      }

      (lib.mkIf sddm {
        environment.systemPackages = [ sddmBackground ];
        environment.pathsToLink = [ "/share/sddm" ];
      })

      (lib.mkIf extras.enable {
        environment.systemPackages = with pkgs.kdePackages; [
          kcalc
          filelight
        ];
        programs.partition-manager.enable = true;
      })

      (lib.mkIf extras.flatpak {
        # With Flatpak on, the Plasma module also installs Discover. The
        # Flathub store is added in base.nix.
        services.flatpak.enable = true;
      })
    ]
  );
}
