# Nixpkgs overlay that turns the stock NixOS graphical installer (Calamares
# plus calamares-nixos-extensions) into the Tanix installer.
#
# What it changes:
#   - patches/tanix-installer.patch: the lines written for each choice on the
#     System page (KDE Plasma, Hyprland, the two servers, Security Edition),
#     the `./tanix` import in the generated configuration.nix, and copying
#     /etc/tanix from the live image to the target.
#   - config/packagechooser.conf: the choices shown to the user.
#   - branding/tanix: installer name, colours, logo and slideshow.
#
# The patch is tied to the pinned nixpkgs revision. If it stops applying after
# a `nix flake update`, see docs/installer.md for how to regenerate it.
final: prev:

let
  assets = ../modules/tanix/assets;

  # Preview tiles for the choices that upstream ships no screenshot for.
  tiles = {
    hyprland = {
      title = "Hyprland";
      subtitle = "on Tanix";
    };
    server = {
      title = "Server";
      subtitle = "Minimal";
    };
    server-web = {
      title = "Web Server";
      subtitle = "Caddy";
    };
    server-containers = {
      title = "Containers";
      subtitle = "Podman";
    };
    server-database = {
      title = "Database";
      subtitle = "PostgreSQL";
    };
    security = {
      title = "Security Edition";
      subtitle = "Minimal KDE Plasma";
    };
  };

  tile =
    { title, subtitle }:
    final.writeText "tile.svg" ''
      <svg xmlns="http://www.w3.org/2000/svg" width="960" height="540" viewBox="0 0 960 540">
      <defs>
      <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#0A1030"/><stop offset="1" stop-color="#1A379E"/></linearGradient>
      <linearGradient id="ft" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFFFFF"/><stop offset="1" stop-color="#E6EEFF"/></linearGradient>
      <linearGradient id="fl" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#C9D8FF"/><stop offset="1" stop-color="#9DB5F8"/></linearGradient>
      <linearGradient id="fr" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#8AA6F2"/><stop offset="1" stop-color="#5370D2"/></linearGradient>
      </defs>
      <rect width="960" height="540" fill="url(#bg)"/>
      <g transform="translate(480,200) scale(0.42)">
      <polygon points="-216.5,-125 0,0 0,250 -216.5,125" fill="url(#fl)"/>
      <polygon points="0,0 216.5,-125 216.5,125 0,250" fill="url(#fr)"/>
      <polygon points="0,-250 216.5,-125 0,0 -216.5,-125" fill="url(#ft)"/>
      </g>
      <text x="480" y="410" text-anchor="middle" font-family="DejaVu Sans" font-weight="bold" font-size="64" fill="#FFFFFF">${title}</text>
      <text x="480" y="462" text-anchor="middle" font-family="DejaVu Sans" font-size="24" fill="#C9D8FF">${subtitle}</text>
      </svg>
    '';
in
{
  # The installer program has the store path of its extension files built
  # in. Rebuilding it for every change to those files means recompiling
  # Calamares (about 10 minutes of full CPU). Instead, take the prebuilt
  # stock program from the binary cache and rewrite that one path in it.
  calamares-nixos = final.replaceDependencies {
    drv = prev.calamares-nixos.override {
      calamares-nixos-extensions = prev.calamares-nixos-extensions;
    };
    replacements = [
      {
        oldDependency = prev.calamares-nixos-extensions;
        newDependency = final.calamares-nixos-extensions;
      }
    ];
  };

  calamares-nixos-extensions = prev.calamares-nixos-extensions.overrideAttrs (old: {
    patches = (old.patches or [ ]) ++ [ ./patches/tanix-installer.patch ];

    nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ final.resvg ];

    postPatch = (old.postPatch or "") + ''
      # Branding: replace the NixOS component with the Tanix one, keeping the
      # upstream desktop screenshots and the "unfree software" page.
      mv branding/nixos branding/tanix
      rm branding/tanix/branding.desc branding/tanix/show.qml \
         branding/tanix/nix-snowflake.svg branding/tanix/white.png \
         branding/tanix/gfx-landing-*.png
      cp ${./branding/tanix}/* branding/tanix/
      # Screenshots of desktops Tanix does not offer.
      find branding/tanix/images -type f ! -name plasma6.jpg -delete
      resvg --width 512 ${assets}/tanix-logo.svg branding/tanix/tanix-logo.png
      resvg --width 256 ${assets}/tanix-mark.svg branding/tanix/tanix-mark.png

      ${final.lib.concatStringsSep "\n" (
        final.lib.mapAttrsToList (id: text: ''
          resvg --skip-system-fonts --use-fonts-dir ${final.dejavu_fonts}/share/fonts \
            ${tile text} branding/tanix/images/${id}.png
        '') tiles
      )}

      substituteInPlace "branding/tanix/notesqml@unfree.qml" \
        --replace-fail "NixOS is fully open source" "Tanix is built on NixOS, which is fully open source"

      # Desktop list and branding selection
      cp ${./config/packagechooser.conf} config/modules/packagechooser.conf
      substituteInPlace config/settings.conf \
        --replace-fail "branding: nixos" "branding: tanix"
    '';
  });
}
