# The Tanix Server ISO: a small text-only image whose installer
# (installer/server/tanix-install.sh) asks for a disk, a user and the server
# roles, then installs Tanix Server. Software is downloaded during the
# install, so the ISO itself stays small.
#
# Built on the stock NixOS minimal installer image, with everything a server
# installer does not need taken out (see "Size" below).
{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}:

let
  volumeId = "TANIX_SERVER";

  # Boot menu entries, in menu order. The first entry is the ISO's own
  # configuration (Minimal); every other one is a specialisation that only
  # adds `tanix.role=<id>` to the kernel command line, which the installer
  # reads to preselect the software. Each adds a few KB to the ISO.
  menuLabel = text: " - Install Server: ${text}";
  extraRoles = [
    {
      id = "web";
      label = "Web (Caddy)";
    }
    {
      id = "ftp";
      label = "FTP (vsftpd)";
    }
    {
      id = "containers";
      label = "Containers (Podman)";
    }
    {
      id = "custom";
      label = "Custom (choose the software)";
    }
  ];

  tanixInstall = pkgs.writeShellApplication {
    name = "tanix-install";
    runtimeInputs = with pkgs; [
      dialog
      curl
      util-linux
      parted
      dosfstools
      e2fsprogs
      gnugrep
      gnused
      coreutils
    ];
    # The ask() helper calls the valid_* functions by name.
    excludeShellChecks = [ "SC2317" ];
    text =
      builtins.replaceStrings
        [ "@volumeId@" "@stateVersion@" ]
        [ volumeId config.system.nixos.release ]
        (builtins.readFile ../installer/server/tanix-install.sh);
  };
in
{
  imports = [
    "${modulesPath}/installer/cd-dvd/installation-cd-minimal.nix"
    ./tanix
    ./iso-boot-menu.nix
  ];

  # The installer copies this folder to /etc/nixos/tanix on the target.
  environment.etc."tanix".source = ./tanix;

  # Plain (non-flake) nixos-install, using the channel bundled on the ISO.
  # Same reason as in iso.nix.
  nixpkgs.flake.setNixPath = false;
  nixpkgs.flake.setFlakeRegistry = false;

  environment.systemPackages = [ tanixInstall ];

  # --- Size ------------------------------------------------------------------
  # The Tanix base (editors, zsh, tools) belongs on the installed server, not
  # on the installer.
  tanix.base.enable = false;

  # Firmware for all hardware is several hundred MB. Left out: the ISO is
  # meant for virtual machines and servers whose network cards work without
  # extra firmware. Installed systems detect and add what they need.
  hardware.enableRedistributableFirmware = lib.mkForce false;

  # No ZFS, no manuals, no git or polkit in the installer.
  boot.supportedFilesystems.zfs = lib.mkForce false;
  documentation.enable = lib.mkForce false;
  documentation.man.enable = lib.mkForce false;
  documentation.doc.enable = lib.mkForce false;
  documentation.nixos.enable = lib.mkForce false;
  programs.git.enable = lib.mkForce false;
  security.polkit.enable = lib.mkForce false;

  # Wired network only, set up automatically (DHCP); no NetworkManager.
  networking.networkmanager.enable = lib.mkForce false;
  networking.useDHCP = lib.mkForce true;

  # Smaller image, slower to build.
  isoImage.squashfsCompression = "xz -Xdict-size 100%";

  # --- Live session ----------------------------------------------------------
  # Live user "tanix" instead of the stock "nixos" (as in iso.nix).
  users.users.nixos.enable = false;
  users.users.tanix = {
    isNormalUser = true;
    description = "Tanix live user";
    extraGroups = [ "wheel" ];
    initialHashedPassword = "";
  };
  services.getty.autologinUser = lib.mkForce "tanix";
  nix.settings.trusted-users = lib.mkForce [
    "root"
    "tanix"
  ];
  services.getty.helpLine = lib.mkForce ''
    Tanix Server installer. Start it with: sudo tanix-install

    The "tanix" and "root" accounts have empty passwords. To install over
    SSH, set a password with `passwd` and log in as tanix.
  '';

  # Start the installer on the first console after the automatic login.
  # Quitting it leaves a normal shell.
  environment.loginShellInit = ''
    if [ "$(tty)" = /dev/tty1 ] && [ "$USER" = tanix ] && [ ! -e /tmp/.tanix-install-started ]; then
      touch /tmp/.tanix-install-started
      sudo tanix-install || true
    fi
  '';

  # ISO file name, volume label and boot menu.
  isoImage.edition = "server";
  image.baseName = lib.mkForce "tanix-server-${config.system.nixos.label}-${pkgs.stdenv.hostPlatform.system}";
  isoImage.volumeID = volumeId;
  isoImage.appendToMenuLabel = menuLabel "Minimal (SSH only)";

  # One boot menu entry per server type (see extraRoles above). The numeric
  # prefix keeps the menu order, which follows the attribute names.
  specialisation = lib.listToAttrs (
    lib.imap1 (
      i: role:
      lib.nameValuePair "${toString i}-${role.id}" {
        configuration = {
          boot.kernelParams = [ "tanix.role=${role.id}" ];
          isoImage.appendToMenuLabel = lib.mkForce (menuLabel role.label);
        };
      }
    ) extraRoles
  );
}
