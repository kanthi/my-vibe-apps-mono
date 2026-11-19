# The Tanix ISO: one image that boots into a live KDE Plasma session and also
# carries the graphical installer (Calamares).
#
# It builds on the stock NixOS "graphical Calamares Plasma 6" installer image
# and adds the Tanix branding module and the Tanix installer overlay.
{
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}:

{
  imports = [
    "${modulesPath}/installer/cd-dvd/installation-cd-graphical-calamares-plasma6.nix"
    ./tanix
    ./iso-boot-menu.nix
  ];

  # Tanix desktop list and branding inside the installer.
  nixpkgs.overlays = [ (import ../installer) ];

  # The installer copies this folder to /etc/nixos/tanix on the target, so the
  # installed system carries the same branding as the live session.
  environment.etc."tanix".source = ./tanix;

  # A system built from a flake points NIX_PATH at "flake:nixpkgs". The
  # installer runs a plain (non-flake) nixos-install, which then fails with
  # "experimental Nix feature 'flakes' is disabled". Turn that off so the
  # live system finds nixpkgs through the bundled channel, as the stock
  # NixOS ISO does.
  nixpkgs.flake.setNixPath = false;
  nixpkgs.flake.setFlakeRegistry = false;

  # Name servers for the live session. Some internet providers hand out DNS
  # servers over DHCP that fail or give wrong answers, and then neither the
  # browser nor the installer's downloads work. These two public servers are
  # put in front of whatever the network offers.
  #
  # Keep this list at two. Only the first three name servers in
  # /etc/resolv.conf are ever used, so the third place is left for the
  # network's own server, as a fallback on networks that block outside DNS.
  networking.networkmanager.insertNameservers = [
    "1.1.1.1" # Cloudflare
    "9.9.9.9" # Quad9
  ];

  # The live user has no password, so locking the screen only gets in the way
  # (for example in the middle of a long install).
  environment.etc."xdg/kscreenlockerrc".text = ''
    [Daemon]
    Autolock=false
    LockOnResume=false
  '';

  # The live user is "tanix" instead of the stock "nixos". The stock user
  # comes from the installer profile and cannot be removed, only switched
  # off; everything that names it is pointed at "tanix" instead.
  users.users.nixos.enable = false;
  users.users.tanix = {
    isNormalUser = true;
    description = "Tanix live user";
    extraGroups = [
      "wheel"
      "networkmanager"
      "video"
    ];
    initialHashedPassword = "";
  };
  services.displayManager.autoLogin.user = lib.mkForce "tanix";
  services.getty.autologinUser = lib.mkForce "tanix";
  nix.settings.trusted-users = lib.mkForce [
    "root"
    "tanix"
  ];
  services.getty.helpLine = lib.mkForce ''
    The "tanix" and "root" accounts have empty passwords.

    To log in over ssh, set a password with `passwd` (or `sudo passwd` for
    root), or add your public key to /home/tanix/.ssh/authorized_keys.

    To set up a wireless connection, run `nmtui`.
  '';

  # Desktop icons of the live session: the installer and the partition
  # editor. Upstream also puts the NixOS manual here; Tanix leaves it out.
  system.activationScripts.installerDesktop = lib.mkForce ''
    mkdir -p /home/tanix/Desktop
    chown tanix /home/tanix /home/tanix/Desktop
    ln -sfT ${pkgs.gparted}/share/applications/gparted.desktop /home/tanix/Desktop/gparted.desktop
    ln -sfT ${pkgs.calamares-nixos}/share/applications/calamares.desktop /home/tanix/Desktop/calamares.desktop
  '';

  # No app store in the live session: nothing installed there survives.
  tanix.desktop.extras.flatpak = false;

  # ISO file name, volume label and boot menu.
  isoImage.edition = "live";
  image.baseName = lib.mkForce "tanix-${config.system.nixos.label}-${pkgs.stdenv.hostPlatform.system}";
  isoImage.volumeID = "TANIX_LIVE";
  isoImage.appendToMenuLabel = " Live / Installer";
}
