# Tanix Security Edition.
#
# The installer's "Security Edition" choice installs KDE Plasma and sets
# `tanix.security.enable = true`. This file decides what that changes:
#
#   - Plasma cut down to a minimal base (see `minimalPlasma` below)
#   - hardening: AppArmor, random MAC addresses, no SSH server
#   - its own wallpaper and a red accent, so it is easy to tell apart
#   - security tools, grouped by category the way Kali Linux's menu is
#
# The tool lists are deliberately empty for now (decided 2026-11-17: start
# from a minimal base, add tools per category later). Add nixpkgs package
# names to a category's `packages` (search at https://search.nixos.org).
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.tanix.security;

  # Kali's menu categories, in Kali's order.
  categories = {
    information-gathering = "Information gathering";
    vulnerability-analysis = "Vulnerability analysis";
    web-application = "Web application analysis";
    database-assessment = "Database assessment";
    password-attacks = "Password attacks";
    wireless-attacks = "Wireless attacks";
    reverse-engineering = "Reverse engineering";
    exploitation = "Exploitation tools";
    sniffing-spoofing = "Sniffing and spoofing";
    post-exploitation = "Post exploitation";
    forensics = "Forensics";
    reporting = "Reporting tools";
    social-engineering = "Social engineering tools";
  };

  categoryOption =
    name: title:
    lib.mkOption {
      type = lib.types.submodule {
        options = {
          enable = lib.mkOption {
            type = lib.types.bool;
            default = true;
            description = "Install the ${title} tools.";
          };
          packages = lib.mkOption {
            type = lib.types.listOf lib.types.package;
            default = [ ];
            description = "Tools in the ${title} category.";
          };
        };
      };
      default = { };
      description = "${title} tools.";
    };

  # Tools from every enabled category. Tools that do not build for this CPU
  # (some are x86 only) are skipped instead of breaking the install.
  tools = lib.filter (lib.meta.availableOn pkgs.stdenv.hostPlatform) (
    lib.concatMap (name: lib.optionals cfg.categories.${name}.enable cfg.categories.${name}.packages) (
      lib.attrNames categories
    )
  );

  # Optional Plasma parts left out of the Security Edition. Kept: Konsole,
  # Dolphin, Kate, Ark, Spectacle (screenshots for reports), the browser.
  minimalPlasma = with pkgs.kdePackages; [
    elisa
    gwenview
    okular
    khelpcenter
    krdp
    plasma-browser-integration
    plasma-workspace-wallpapers
    plasma-keyboard
    qtvirtualkeyboard
    ffmpegthumbs
    dolphin-plugins
  ];
in
{
  options.tanix.security = {
    enable = lib.mkEnableOption "the Tanix Security Edition";

    categories = lib.mapAttrs categoryOption categories;

    extraPackages = lib.mkOption {
      type = lib.types.listOf lib.types.package;
      default = [ ];
      description = "Tools that fit no category.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = tools ++ cfg.extraPackages;

    # Shown as VARIANT in /etc/os-release.
    system.nixos.variantName = lib.mkDefault "Security Edition";
    system.nixos.variant_id = lib.mkDefault "security";

    # Look: its own wallpaper and a red accent.
    tanix.branding.wallpaper = lib.mkDefault "security";
    tanix.branding.accentColor = lib.mkDefault "178,34,52";

    # Minimal Plasma.
    environment.plasma6.excludePackages = minimalPlasma;
    tanix.desktop.extras.enable = lib.mkDefault false;
    # Plasma switches these on with mkDefault; 900 wins over that and still
    # loses to a plain setting in configuration.nix.
    services.orca.enable = lib.mkOverride 900 false;
    programs.kde-pim.enable = lib.mkOverride 900 false;
    # No file indexing in the background.
    environment.etc."xdg/baloofilerc".text = ''
      [Basic Settings]
      Indexing-Enabled=false
    '';

    # Hardening.
    security.apparmor.enable = true;
    networking.networkmanager.wifi.macAddress = lib.mkDefault "random";
    networking.networkmanager.ethernet.macAddress = lib.mkDefault "random";
    # Nothing listens on the network unless asked for.
    services.openssh.enable = lib.mkDefault false;
  };
}
