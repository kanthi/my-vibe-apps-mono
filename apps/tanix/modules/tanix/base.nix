# Tanix base: what every Tanix system gets, whichever choice was installed.
#
#   - a small set of everyday command-line tools
#   - flakes switched on
#   - weekly clean-up of old system generations, and store de-duplication
#   - compressed swap in RAM (zram)
#   - zsh with the starship prompt as the default shell, fastfetch on login
#   - optional automatic upgrades (on by default for servers only)
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.tanix;
in
{
  options.tanix = {
    base.enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Apply the Tanix base: tools, shell, clean-up, zram, flakes.";
    };

    autoUpgrade.enable = lib.mkOption {
      type = lib.types.bool;
      default = cfg.server.enable;
      defaultText = lib.literalExpression "config.tanix.server.enable";
      description = ''
        Upgrade the system automatically every night from its NixOS channel.
        Never reboots by itself; a new kernel takes effect at the next reboot.
        On by default for servers, off for desktops.
      '';
    };
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.base.enable {
      environment.systemPackages = with pkgs; [
        git
        curl
        wget
        htop
        btop
        ripgrep
        fd
        jq
        tree
        file
        unzip
        zip
        vim
        helix
        fastfetch
      ];

      nix.settings.experimental-features = [
        "nix-command"
        "flakes"
      ];

      # Old generations otherwise pile up until the disk is full.
      nix.gc = {
        automatic = true;
        dates = "weekly";
        options = "--delete-older-than 14d";
      };
      nix.optimise.automatic = true;

      zramSwap.enable = true;

      programs.zsh.enable = true;
      # 900: beats NixOS's own mkDefault (bash), loses to a plain setting
      # (for example in configuration.nix).
      users.defaultUserShell = lib.mkOverride 900 pkgs.zsh;
      programs.starship.enable = true;
      # Without a ~/.zshrc, zsh starts its interactive "new user" wizard in
      # every new terminal (seen in the live session). NixOS already sets
      # zsh up system-wide, so skip it. /etc/zshenv runs before that check.
      programs.zsh.shellInit = ''
        zsh-newuser-install() { :; }
      '';

      # System summary when a login shell starts (console, SSH).
      programs.zsh.interactiveShellInit = ''
        if [[ -o login ]] && [[ -z "$TANIX_NO_FASTFETCH" ]]; then
          fastfetch
        fi
      '';
    })

    # Wherever Flatpak is on (KDE Plasma), add the Flathub store.
    (lib.mkIf config.services.flatpak.enable {
      systemd.services.tanix-flathub = {
        description = "Add the Flathub app store to Flatpak";
        wantedBy = [ "multi-user.target" ];
        wants = [ "network-online.target" ];
        after = [ "network-online.target" ];
        path = [ pkgs.flatpak ];
        serviceConfig = {
          Type = "oneshot";
          Restart = "on-failure";
          RestartSec = 60;
        };
        script = ''
          flatpak remote-add --system --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo
        '';
      };
    })

    (lib.mkIf cfg.autoUpgrade.enable {
      system.autoUpgrade = {
        enable = true;
        dates = "04:00";
        randomizedDelaySec = "45min";
        allowReboot = false;
      };
    })
  ];
}
