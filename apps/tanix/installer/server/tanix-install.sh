# tanix-install: the text installer of the Tanix Server ISO.
#
# Asks for a disk, host name, user, SSH key and the server roles (a
# checklist), erases the disk, writes /etc/nixos/configuration.nix that
# imports the Tanix module, and runs nixos-install. Packages are downloaded
# from the NixOS binary cache, so the machine needs internet during install.
#
# Packaged with writeShellApplication in modules/iso-server.nix, which adds
# `set -euo pipefail`, the PATH, and a shellcheck run at build time.
# @volumeId@ is replaced with the ISO's volume label there.

TITLE="Tanix Server installer"
ISO_LABEL="@volumeId@"
LOG=/tmp/tanix-install.log

if [ "$(id -u)" -ne 0 ]; then
  exec sudo "$0" "$@"
fi

# dialog writes the answer to stderr; swap it onto stdout.
d() {
  dialog --backtitle "$TITLE" "$@" 3>&1 1>&2 2>&3
}

# Prompts run inside $(...) subshells, so quitting also signals the main script.
trap 'exit 1' TERM
quit_check() {
  if d --title "Quit" --defaultno --yesno "Quit the installer? Nothing has been changed on any disk yet." 7 60 >/dev/null; then
    clear >/dev/tty
    echo "Installer closed. Run 'sudo tanix-install' to start it again." >/dev/tty
    kill -TERM $$
    exit 1
  fi
}

# Asks again until the answer is not empty and passes the check function.
ask() {
  local check=$1 errmsg=$2 answer
  shift 2
  while true; do
    if ! answer=$(d "$@"); then
      quit_check
      continue
    fi
    if "$check" "$answer"; then
      printf '%s' "$answer"
      return 0
    fi
    d --title "Not valid" --msgbox "$errmsg" 8 60 >/dev/null || true
  done
}

valid_hostname() { [[ $1 =~ ^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?$ ]]; }
valid_username() {
  [[ $1 =~ ^[a-z_][a-z0-9_-]{0,30}$ ]] || return 1
  case "$1" in root | nixbld* | nobody | messagebus | sshd | caddy | vsftpd | podman | postgres) return 1 ;; esac
}
valid_timezone() { [ -f "/etc/zoneinfo/$1" ]; }

# Nix string escaping for values written into configuration.nix.
nix_str() {
  local s=$1
  s=${s//\\/\\\\}
  s=${s//\"/\\\"}
  s=${s//\$\{/\\\$\{}
  printf '"%s"' "$s"
}

# Partition device name: /dev/sda -> /dev/sda1, /dev/nvme0n1 -> /dev/nvme0n1p1
part() {
  if [[ $1 =~ [0-9]$ ]]; then printf '%sp%s' "$1" "$2"; else printf '%s%s' "$1" "$2"; fi
}

# --- Server type from the boot menu ------------------------------------------
# Each boot menu entry adds tanix.role=<type> to the kernel command line (see
# modules/iso-server.nix). A type preselects the software and skips the
# checklist; "custom" shows the checklist. No entry means Minimal.
role=minimal
read -ra cmdline </proc/cmdline
for opt in "${cmdline[@]}"; do
  case "$opt" in tanix.role=*) role=${opt#tanix.role=} ;; esac
done
case "$role" in
  minimal) preset="ssh" role_name="Minimal (SSH only)" ;;
  web) preset=$'ssh\nweb' role_name="Web (Caddy)" ;;
  ftp) preset=$'ssh\nftp' role_name="FTP (vsftpd)" ;;
  containers) preset=$'ssh\ncontainers' role_name="Containers (Podman)" ;;
  *) preset="" role_name="Custom" ;;
esac

# --- Welcome -----------------------------------------------------------------

d --title "Welcome to Tanix Server" --msgbox "\
This installs Tanix Server, a small server system built on NixOS.

Server type: $role_name (chosen in the boot menu)

You will choose a disk, a name for this machine and your user
account. The chosen disk is erased completely.

The software is downloaded during the install, so this machine
needs an internet connection." 16 72 >/dev/null || quit_check

# --- Network -----------------------------------------------------------------

while ! curl -fsS --max-time 10 -o /dev/null https://cache.nixos.org/nix-cache-info; do
  if ! d --title "No internet connection" --yes-label "Try again" --no-label "Quit" --yesno "\
The installer cannot reach the NixOS download server (cache.nixos.org).

Connect a network cable (the network is set up automatically), then
choose 'Try again'." 11 66 >/dev/null; then
    quit_check
  fi
done

# --- Disk --------------------------------------------------------------------

disk_items=()
while read -r name size type; do
  [ "$type" = disk ] || continue
  case "$name" in /dev/zram* | /dev/loop* | /dev/sr* | /dev/ram*) continue ;; esac
  # Skip the disk this installer was started from.
  if lsblk -nro LABEL "$name" 2>/dev/null | grep -qxF "$ISO_LABEL"; then
    continue
  fi
  model=$(lsblk -dno MODEL "$name" 2>/dev/null | sed 's/ *$//')
  disk_items+=("$name" "$size ${model:-disk}")
done < <(lsblk -dnpo NAME,SIZE,TYPE)

if [ ${#disk_items[@]} -eq 0 ]; then
  d --title "No disk found" --msgbox "No disk was found to install on." 7 50 >/dev/null || true
  clear
  exit 1
fi

while true; do
  if disk=$(d --title "Disk" --menu "Install Tanix Server on which disk? Everything on it will be erased." 15 70 6 "${disk_items[@]}"); then
    break
  fi
  quit_check
done
disk_desc=$(lsblk -dno SIZE,MODEL "$disk" | sed 's/ *$//')

# --- Machine and user --------------------------------------------------------

hostname=$(ask valid_hostname "Use lower-case letters, digits and '-' (not at the start or end)." \
  --title "Host name" --inputbox "Name of this machine on the network:" 9 60 "tanix")

timezone=$(ask valid_timezone "Unknown time zone. Examples: UTC, Europe/Berlin, Asia/Kolkata, America/New_York." \
  --title "Time zone" --inputbox "Time zone (for example UTC, Europe/Berlin, Asia/Kolkata):" 9 60 "UTC")

username=$(ask valid_username "Use lower-case letters, digits, '_' and '-', starting with a letter. Some system names (root, caddy, ...) are taken." \
  --title "Your account" --inputbox "User name to log in with. This user can run sudo." 9 60 "")

while true; do
  pass1=$(d --title "Password" --insecure --passwordbox "Password for $username:" 9 60) || { quit_check; continue; }
  pass2=$(d --title "Password" --insecure --passwordbox "The same password again:" 9 60) || { quit_check; continue; }
  if [ -z "$pass1" ]; then
    d --title "Not valid" --msgbox "The password cannot be empty." 7 50 >/dev/null || true
  elif [ "$pass1" != "$pass2" ]; then
    d --title "Not valid" --msgbox "The two passwords are not the same." 7 50 >/dev/null || true
  else
    break
  fi
done

# --- SSH key -----------------------------------------------------------------

ssh_keys=""
while true; do
  src=$(d --title "SSH key (optional)" --inputbox "\
SSH logins use keys, not passwords. Give your public key:

  github:NAME            the keys you published on GitHub
  https://...            a URL that lists keys
  ssh-ed25519 AAAA...    a key pasted in

Leave empty to add one later with 'tanix-add-ssh-key'." 15 72 "") || { quit_check; continue; }
  [ -n "$src" ] || break
  keys=""
  case "$src" in
    github:*) keys=$(curl -fsSL --max-time 15 "https://github.com/${src#github:}.keys" || true) ;;
    https://*) keys=$(curl -fsSL --max-time 15 "$src" || true) ;;
    ssh-* | ecdsa-* | sk-*) keys=$src ;;
  esac
  keys=$(printf '%s\n' "$keys" | grep -E '^(ssh-|ecdsa-|sk-)' || true)
  if [ -n "$keys" ]; then
    ssh_keys=$keys
    count=$(printf '%s\n' "$ssh_keys" | wc -l)
    d --title "SSH key" --msgbox "Found $count key(s). They will be allowed to log in as $username." 7 64 >/dev/null || true
    break
  fi
  d --title "No key found" --msgbox "No SSH public key was found at: $src" 7 64 >/dev/null || true
done

# --- Roles -------------------------------------------------------------------

roles=$preset
while [ -z "$preset" ]; do
  if roles=$(d --title "Software" --separate-output --checklist "\
Choose the software (space to tick, enter to continue). All of it
can be changed later in /etc/nixos/configuration.nix." 15 76 4 \
    ssh "SSH server: keys only, with fail2ban" on \
    web "Web server: Caddy, serves /var/www" off \
    ftp "FTP server: vsftpd, users in their home" off \
    containers "Containers: Podman, docker command" off); then
    break
  fi
  quit_check
done
has_role() { printf '%s\n' "$roles" | grep -qx "$1"; }
yesno() { if has_role "$1"; then printf true; else printf false; fi; }

roles_text=$(printf '%s\n' "$roles" | paste -sd, - | sed 's/,/, /g')
[ -n "$roles_text" ] || roles_text="none (minimal server)"
ssh_note=""
if has_role ssh && [ -z "$ssh_keys" ]; then
  ssh_note="
Note: no SSH key given; add one at the console after the first boot."
fi

# --- Confirm -----------------------------------------------------------------

d --title "Ready to install" --defaultno --yes-label "Erase and install" --no-label "Quit" --yesno "\
Disk:       $disk ($disk_desc)  -- WILL BE ERASED
Host name:  $hostname
Time zone:  $timezone
User:       $username
SSH keys:   $(if [ -n "$ssh_keys" ]; then printf '%s' "$(printf '%s\n' "$ssh_keys" | wc -l)"; else printf none; fi)
Software:   $roles_text
$ssh_note" 15 74 >/dev/null || {
  clear
  echo "Nothing was changed. Run 'sudo tanix-install' to start again."
  exit 1
}

# --- Install -----------------------------------------------------------------

clear
# Show the output and keep a copy in $LOG; the terminal is restored at the end.
exec 4>&1 5>&2
exec > >(tee "$LOG") 2>&1

fail() {
  echo
  echo "!! The install failed. The full log is in $LOG."
  echo "!! Run 'sudo tanix-install' to try again."
  exit 1
}
trap fail ERR

echo "==> Erasing and partitioning $disk"
umount -R /mnt 2>/dev/null || true
wipefs -af "$disk"
if [ -d /sys/firmware/efi ]; then
  efi=true
  parted -s "$disk" -- mklabel gpt \
    mkpart ESP fat32 1MiB 513MiB set 1 esp on \
    mkpart root ext4 513MiB 100%
else
  efi=false
  parted -s "$disk" -- mklabel gpt \
    mkpart bios 1MiB 2MiB set 1 bios_grub on \
    mkpart root ext4 2MiB 100%
fi
udevadm settle
root_part=$(part "$disk" 2)

echo "==> Creating file systems"
mkfs.ext4 -F -L tanix-root "$root_part"
mount "$root_part" /mnt
if $efi; then
  boot_part=$(part "$disk" 1)
  mkfs.fat -F 32 -n TANIX_BOOT "$boot_part"
  mkdir -p /mnt/boot
  mount -o umask=077 "$boot_part" /mnt/boot
fi

echo "==> Writing the configuration"
nixos-generate-config --root /mnt
cp -rL /etc/tanix /mnt/etc/nixos/tanix
chmod -R u+w /mnt/etc/nixos/tanix

if $efi; then
  bootloader='  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = true;'
else
  bootloader="  boot.loader.grub.enable = true;
  boot.loader.grub.device = $(nix_str "$disk");"
fi

keys_nix=""
if [ -n "$ssh_keys" ]; then
  while IFS= read -r key; do
    keys_nix+="      $(nix_str "$key")
"
  done <<<"$ssh_keys"
fi

cat >/mnt/etc/nixos/configuration.nix <<EOF
# Tanix Server, written by tanix-install. Change anything here, then apply
# it with: sudo nixos-rebuild switch
# What the tanix.* settings do: /etc/nixos/tanix/server.nix
{ config, pkgs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    # Tanix branding, base and server roles. Copied here by the installer.
    ./tanix
  ];

$bootloader

  networking.hostName = $(nix_str "$hostname");
  time.timeZone = $(nix_str "$timezone");
  i18n.defaultLocale = "en_US.UTF-8";

  users.users.$username = {
    isNormalUser = true;
    extraGroups = [ "wheel" ];
    openssh.authorizedKeys.keys = [
$keys_nix    ];
  };

  # Tanix server roles: set to true or false, then nixos-rebuild switch.
  tanix.server.enable = true;
  tanix.server.ssh.enable = $(yesno ssh);         # SSH, keys only, fail2ban
  tanix.server.web.enable = $(yesno web);         # Caddy serving /var/www
  tanix.server.web.domain = null;          # or run: tanix-web-setup
  tanix.server.web.email = null;
  tanix.server.ftp.enable = $(yesno ftp);         # vsftpd, users in their home
  tanix.server.containers.enable = $(yesno containers);  # Podman + docker command
  tanix.server.database.enable = false;    # PostgreSQL, this machine only
  # tanix.server.tailscale.enable = true;  # private network with Tailscale

  # Nightly upgrades from the NixOS channel (never reboots by itself).
  tanix.autoUpgrade.enable = true;

  system.stateVersion = "@stateVersion@";
}
EOF

echo "==> Installing (downloads the software; this takes a few minutes)"
nixos-install --root /mnt --no-root-passwd

echo "==> Setting the password for $username"
printf '%s:%s\n' "$username" "$pass1" | nixos-enter --root /mnt -c chpasswd

trap - ERR
sync
exec 1>&4 2>&5

if d --title "Done" --yes-label "Reboot" --no-label "Shell" --yesno "\
Tanix Server is installed on $disk.

Remove the installation media, then reboot. Log in as $username.
Settings: /etc/nixos/configuration.nix" 11 66 >/dev/null; then
  umount -R /mnt || true
  reboot
fi
clear
echo "Installed. The new system is mounted at /mnt. Type 'reboot' when ready."
