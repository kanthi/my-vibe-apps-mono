# Tanix: branding, the Tanix KDE Plasma desktop layout, and editions.
#
# This one module is used in two places:
#   1. the live/installer ISO (imported by ../iso.nix), and
#   2. every installed system: the installer copies this whole folder to
#      /etc/nixos/tanix and the generated configuration.nix imports it.
#
# Because of (2) it must stay self-contained: only plain nixpkgs packages and
# the files in ./assets. No flake inputs, no overlays.
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.tanix.branding;
  desktopCfg = config.tanix.desktop;

  # Wallpapers are kept as SVG and rendered at build time.
  wallpapers =
    pkgs.runCommand "tanix-wallpapers"
      {
        nativeBuildInputs = [ pkgs.resvg ];
      }
      ''
        bg=$out/share/backgrounds/tanix
        mkdir -p $bg $out/share/gnome-background-properties

        # The Security Edition's wallpaper is the dark one in red.
        sed -e 's/#0A1030/#1A0608/g' -e 's/#1A379E/#7A1424/g' -e 's/#3F6BE0/#E03F4F/g' \
          -e 's/#C9D8FF/#FFD0D4/g' -e 's/#9DB5F8/#F2A0A8/g' \
          -e 's/#8AA6F2/#E0707C/g' -e 's/#5370D2/#B22234/g' \
          ${./assets/wallpapers}/tanix-dark.svg > tanix-security.svg

        for variant in dark light security; do
          src=${./assets/wallpapers}/tanix-$variant.svg
          [ $variant = security ] && src=tanix-security.svg
          resvg --width 3840 $src $bg/tanix-$variant.png

          # KDE Plasma wallpaper package
          name=Tanix-$variant
          kde=$out/share/wallpapers/$name
          mkdir -p $kde/contents/images
          ln -s $bg/tanix-$variant.png $kde/contents/images/3840x2160.png
          cat > $kde/metadata.json <<EOF
        {
          "KPlugin": {
            "Id": "$name",
            "Name": "Tanix ($variant)",
            "License": "CC-BY-SA-4.0"
          }
        }
        EOF
        done

        # GNOME "Background" settings entry
        cat > $out/share/gnome-background-properties/tanix.xml <<EOF
        <?xml version="1.0"?>
        <!DOCTYPE wallpapers SYSTEM "gnome-wp-list.dtd">
        <wallpapers>
          <wallpaper deleted="false">
            <name>Tanix</name>
            <filename>$bg/tanix-light.png</filename>
            <filename-dark>$bg/tanix-dark.png</filename-dark>
            <options>zoom</options>
            <shade_type>solid</shade_type>
            <pcolor>#1A379E</pcolor>
            <scolor>#0A1030</scolor>
          </wallpaper>
        </wallpapers>
        EOF
      '';

  wallpaper = "${wallpapers}/share/backgrounds/tanix/tanix-${cfg.wallpaper}.png";
  wallpaperLight = "${wallpapers}/share/backgrounds/tanix/tanix-light.png";
  wallpaperDark = "${wallpapers}/share/backgrounds/tanix/tanix-dark.png";

  # Icon named "tanix", referenced by LOGO= in /etc/os-release.
  logoIcon = pkgs.runCommand "tanix-logo-icon" { nativeBuildInputs = [ pkgs.resvg ]; } ''
    install -Dm644 ${./assets/tanix-logo.svg} $out/share/icons/hicolor/scalable/apps/tanix.svg
    for size in 48 128 256; do
      mkdir -p $out/share/icons/hicolor/''${size}x''${size}/apps
      resvg --width $size ${./assets/tanix-logo.svg} $out/share/icons/hicolor/''${size}x''${size}/apps/tanix.png
    done
    mkdir -p $out/share/pixmaps
    ln -s $out/share/icons/hicolor/256x256/apps/tanix.png $out/share/pixmaps/tanix.png
  '';

  # Boot splash (Plymouth) logo.
  plymouthLogo = pkgs.runCommand "tanix-plymouth-logo.png" { nativeBuildInputs = [ pkgs.resvg ]; } ''
    resvg --width 160 ${./assets/tanix-mark.svg} $out
  '';

  # The Tanix Plasma layout, written in Plasma's own desktop scripting
  # language and using only widgets that ship with Plasma:
  #   - a thin bar at the top: launcher, the active window's menu, tray, clock
  #   - a floating dock at the bottom: pinned and running apps as icons
  plasmaLayoutScript = pkgs.writeText "tanix-plasma-layout.js" ''
    var old = panels();
    for (var i = 0; i < old.length; i++) {
      old[i].remove();
    }

    var bar = new Panel;
    bar.location = "top";
    bar.height = 28;
    bar.floating = false;
    bar.addWidget("org.kde.plasma.kickoff");
    bar.addWidget("org.kde.plasma.appmenu");
    bar.addWidget("org.kde.plasma.panelspacer");
    bar.addWidget("org.kde.plasma.systemtray");
    bar.addWidget("org.kde.plasma.digitalclock");

    var dock = new Panel;
    dock.location = "bottom";
    dock.height = 56;
    dock.lengthMode = "fit";
    dock.alignment = "center";
    dock.floating = true;
    dock.hiding = "dodgewindows";
    dock.addWidget("org.kde.plasma.icontasks");
  '';

  # Plasma keeps panels and wallpaper in per-user state, so apply them once
  # per user at first login. Later changes made by the user are left alone.
  # The icon theme is applied the same way: a system-wide default in
  # /etc/xdg/kdeglobals is not enough, because at login Plasma writes the
  # Breeze look-and-feel defaults (Icons Theme=breeze) to
  # ~/.config/kdedefaults/kdeglobals, which takes precedence over /etc/xdg.
  plasmaWallpaperScript = pkgs.writeShellScript "tanix-plasma-setup" ''
    stamp="''${XDG_STATE_HOME:-$HOME/.local/state}/tanix/plasma-setup-done"
    [ -e "$stamp" ] && exit 0
    for attempt in $(seq 1 30); do
      sleep 2
      if ${pkgs.kdePackages.plasma-workspace}/bin/plasma-apply-wallpaperimage ${wallpaper}; then
        ${pkgs.kdePackages.plasma-workspace}/libexec/plasma-changeicons ${config.tanix.desktop.iconTheme} || true
        ${lib.optionalString desktopCfg.macLayout ''
          ${pkgs.systemd}/bin/busctl --user call org.kde.plasmashell /PlasmaShell \
            org.kde.PlasmaShell evaluateScript s "$(cat ${plasmaLayoutScript})" || true
        ''}
        mkdir -p "$(dirname "$stamp")"
        touch "$stamp"
        exit 0
      fi
    done
    exit 1
  '';

  plasmaWallpaperAutostart = pkgs.writeTextDir "etc/xdg/autostart/tanix-plasma-wallpaper.desktop" ''
    [Desktop Entry]
    Type=Application
    Name=Tanix desktop setup
    Exec=${plasmaWallpaperScript}
    OnlyShowIn=KDE;
    NoDisplay=true
    X-KDE-autostart-phase=2
  '';

  dm = config.services.desktopManager;
in
{
  imports = [
    ./base.nix
    ./plasma.nix
    ./hyprland.nix
    ./server.nix
    ./security.nix
  ];

  options.tanix.desktop.macLayout = lib.mkOption {
    type = lib.types.bool;
    default = true;
    description = ''
      On KDE Plasma, give new users the Tanix layout: a menu bar at the top,
      a dock at the bottom and window buttons on the left. Uses only parts
      that ship with Plasma.
    '';
  };

  options.tanix.branding = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Apply Tanix naming, logo, boot splash and wallpapers.";
    };

    wallpaper = lib.mkOption {
      type = lib.types.enum [
        "dark"
        "light"
        "security"
      ];
      default = "dark";
      description = "Which Tanix wallpaper is the default for desktops that take a single image.";
    };

    accentColor = lib.mkOption {
      type = lib.types.str;
      default = "26,55,158";
      description = "Plasma accent colour as R,G,B. Default: Tanix blue (#1A379E).";
    };

    wallpaperPath = lib.mkOption {
      type = lib.types.str;
      readOnly = true;
      internal = true;
      default = wallpaper;
      description = "The chosen wallpaper image, for the other Tanix modules.";
    };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        # Name of the system: /etc/os-release, boot menu entries, login banner.
        system.nixos.distroName = "Tanix";
        system.nixos.distroId = "tanix";
        system.nixos.vendorName = "Tanix";
        system.nixos.vendorId = "tanix";
        system.nixos.extraOSReleaseArgs.LOGO = "tanix";

        # Wallpapers only where there is a desktop to show them; servers and
        # the Tanix Server ISO stay without them.
        environment.systemPackages = [
          logoIcon
        ]
        ++ lib.optional (dm.plasma6.enable || config.programs.hyprland.enable) wallpapers;
        environment.pathsToLink = [
          "/share/backgrounds"
          "/share/wallpapers"
          "/share/gnome-background-properties"
          "/share/icons"
          "/share/pixmaps"
        ];

        # Only takes effect where Plymouth is enabled.
        boot.plymouth.logo = plymouthLogo;
      }

      # KDE Plasma: wallpaper and (optionally) the Tanix layout
      (lib.mkIf dm.plasma6.enable {
        environment.systemPackages = [ plasmaWallpaperAutostart ];
      })

      # Window buttons on the left: close, minimise, maximise.
      (lib.mkIf (dm.plasma6.enable && desktopCfg.macLayout) {
        environment.etc."xdg/kwinrc".text = ''
          [org.kde.kdecoration2]
          ButtonsOnLeft=XIA
          ButtonsOnRight=
        '';
      })

      # Hyprland draws no wallpaper itself; run swaybg in the session.
      (lib.mkIf config.programs.hyprland.enable {
        systemd.user.services.tanix-wallpaper = {
          description = "Tanix wallpaper";
          partOf = [ "graphical-session.target" ];
          after = [ "graphical-session.target" ];
          wantedBy = [ "graphical-session.target" ];
          serviceConfig = {
            ExecStart = "${pkgs.swaybg}/bin/swaybg --image ${wallpaper} --mode fill";
            Restart = "on-failure";
            RestartSec = 2;
          };
        };
      })
    ]
  );
}
