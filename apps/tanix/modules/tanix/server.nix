# Tanix servers. The installers' server choices turn these on:
#
#   Server: Minimal       tanix.server.enable
#   SSH server            + tanix.server.ssh.enable (on by default)
#   Server: Web (Caddy)   + tanix.server.web.enable
#   FTP server (vsftpd)   + tanix.server.ftp.enable
#   Server: Containers    + tanix.server.containers.enable
#   Server: Database      + tanix.server.database.enable
#
# The desktop ISO's graphical installer offers one server role at a time; the
# Tanix Server ISO's text installer (installer/server/) offers a checklist.
#
# Every server gets nightly upgrades (tanix.autoUpgrade, see base.nix) and,
# unless switched off, SSH with keys only guarded by fail2ban. Tailscale is an
# extra that is off unless switched on in configuration.nix.
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.tanix.server;

  # Adds a public key to the calling user's ~/.ssh/authorized_keys, so a
  # fresh server can be reached over SSH (password login is off).
  addSshKey = pkgs.writeShellApplication {
    name = "tanix-add-ssh-key";
    runtimeInputs = [ pkgs.curl ];
    text = ''
      usage() {
        cat <<EOF
      Usage: tanix-add-ssh-key <key source>

      Adds SSH public keys to ~/.ssh/authorized_keys for $(id -un).

        tanix-add-ssh-key github:NAME           keys published on GitHub
        tanix-add-ssh-key https://host/keys     keys at any URL
        tanix-add-ssh-key "ssh-ed25519 AAAA..."  a key typed or pasted in
      EOF
      }

      [ $# -eq 1 ] || { usage; exit 1; }
      case "$1" in
        github:*) keys=$(curl -fsSL "https://github.com/''${1#github:}.keys") ;;
        https://*) keys=$(curl -fsSL "$1") ;;
        ssh-* | ecdsa-* | sk-*) keys="$1" ;;
        *) usage; exit 1 ;;
      esac
      [ -n "$keys" ] || { echo "No keys found at $1" >&2; exit 1; }

      mkdir -p ~/.ssh
      chmod 700 ~/.ssh
      touch ~/.ssh/authorized_keys
      chmod 600 ~/.ssh/authorized_keys
      added=0
      while IFS= read -r key; do
        [ -n "$key" ] || continue
        if ! grep -qxF "$key" ~/.ssh/authorized_keys; then
          echo "$key" >> ~/.ssh/authorized_keys
          added=$((added + 1))
        fi
      done <<< "$keys"
      echo "Added $added key(s) to ~/.ssh/authorized_keys."
    '';
  };

  # Asks for the web server's domain and e-mail address, writes them into
  # /etc/nixos/configuration.nix and applies the change.
  webSetup = pkgs.writeShellApplication {
    name = "tanix-web-setup";
    runtimeInputs = [ pkgs.gnused ];
    text = ''
      conf=/etc/nixos/configuration.nix
      if [ "$(id -u)" -ne 0 ]; then
        exec sudo "$0" "$@"
      fi
      grep -q '^ *tanix.server.web.domain = ' "$conf" || {
        echo "No tanix.server.web.domain line in $conf; set it by hand." >&2
        exit 1
      }

      read -rp "Domain name (empty to serve plain HTTP on port 80): " domain
      email=""
      if [ -n "$domain" ]; then
        read -rp "E-mail for the HTTPS certificate (optional): " email
      fi

      quote() { if [ -n "$1" ]; then printf '"%s"' "$1"; else printf 'null'; fi; }
      sed -i \
        -e "s|^\( *tanix.server.web.domain = \).*;|\1$(quote "$domain");|" \
        -e "s|^\( *tanix.server.web.email = \).*;|\1$(quote "$email");|" \
        "$conf"

      echo "Applying the new configuration..."
      nixos-rebuild switch
      if [ -n "$domain" ]; then
        echo "Done. Point $domain at this machine; Caddy fetches the certificate by itself."
      fi
    '';
  };

  welcomePage = pkgs.writeText "index.html" ''
    <!doctype html>
    <html lang="en">
    <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tanix web server</title>
    <style>
      body { margin: 0; min-height: 100vh; display: grid; place-items: center;
             background: linear-gradient(135deg, #0A1030, #1A379E); color: #fff;
             font: 18px/1.6 system-ui, sans-serif; }
      main { max-width: 34rem; padding: 2rem; }
      h1 { margin: 0 0 .5rem; font-size: 2.2rem; }
      code { background: rgba(255,255,255,.12); padding: .1rem .35rem; border-radius: 4px; }
      p { color: #C9D8FF; }
    </style>
    </head>
    <body>
    <main>
      <h1>It works.</h1>
      <p>This is a fresh Tanix web server, running Caddy.</p>
      <p>Replace this page by putting your own files in <code>/var/www</code>.
         To serve a domain over HTTPS, run <code>tanix-web-setup</code>.</p>
    </main>
    </body>
    </html>
  '';
in
{
  options.tanix.server = {
    enable = lib.mkEnableOption "the Tanix server base (SSH with keys only, fail2ban, nightly upgrades)";

    ssh.enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Run the SSH server: keys only, no root login, guarded by fail2ban.";
    };

    tailscale.enable = lib.mkEnableOption "Tailscale, to reach this server over a private network";

    web = {
      enable = lib.mkEnableOption "the Caddy web server, serving /var/www";
      domain = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        example = "example.com";
        description = ''
          Domain to serve. Caddy fetches an HTTPS certificate for it by itself.
          With null, plain HTTP is served on port 80 for any name.
          `tanix-web-setup` asks for it and sets it.
        '';
      };
      email = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "Contact address given to the certificate authority (optional).";
      };
    };

    ftp.enable = lib.mkEnableOption ''
      the vsftpd FTP server. The system's users log in with their password and
      are kept inside their home folder. Plain FTP is not encrypted; with SSH
      on, the same users can also use SFTP with their SSH key'';

    containers.enable = lib.mkEnableOption "Podman containers, with a docker-compatible command";

    database.enable = lib.mkEnableOption "the PostgreSQL database server, reachable from this machine only";
  };

  config = lib.mkMerge [
    (lib.mkIf cfg.enable {
      environment.systemPackages = [ addSshKey ];

      users.motd = ''
        Tanix server.${lib.optionalString cfg.ssh.enable " SSH accepts keys only: add yours from this console with\n  tanix-add-ssh-key github:YOUR-NAME"}
        Settings: /etc/nixos/configuration.nix, then: sudo nixos-rebuild switch
      '';
    })

    (lib.mkIf (cfg.enable && cfg.ssh.enable) {
      services.openssh = {
        enable = true;
        settings = {
          PermitRootLogin = "no";
          PasswordAuthentication = false;
          KbdInteractiveAuthentication = false;
        };
      };
      services.fail2ban.enable = true;
    })

    (lib.mkIf cfg.tailscale.enable {
      services.tailscale.enable = true;
    })

    (lib.mkIf cfg.web.enable {
      services.caddy = {
        enable = true;
        email = cfg.web.email;
        virtualHosts.${if cfg.web.domain == null then ":80" else cfg.web.domain}.extraConfig = ''
          root * /var/www
          file_server
        '';
      };
      systemd.tmpfiles.rules = [
        "d /var/www 0755 caddy caddy -"
        # Copied once; never overwrites a page that is already there.
        "C /var/www/index.html 0644 caddy caddy - ${welcomePage}"
      ];
      # Decided later together with the rest of the firewall question; these
      # two ports are what the tested web server already opened.
      networking.firewall.allowedTCPPorts = [
        80
        443
      ];
      environment.systemPackages = [ webSetup ];
    })

    (lib.mkIf cfg.ftp.enable {
      # NixOS only defines the vsftpd PAM service for virtual users; without
      # it every login of a system user is refused (found in the VM test).
      security.pam.services.vsftpd = { };
      services.vsftpd = {
        enable = true;
        localUsers = true;
        writeEnable = true;
        chrootlocalUser = true;
        allowWriteableChroot = true;
        extraConfig = ''
          pasv_enable=YES
          pasv_min_port=51000
          pasv_max_port=51099
        '';
      };
      networking.firewall.allowedTCPPorts = [ 21 ];
      networking.firewall.allowedTCPPortRanges = [
        {
          from = 51000;
          to = 51099;
        }
      ];
    })

    (lib.mkIf cfg.containers.enable {
      virtualisation.podman = {
        enable = true;
        dockerCompat = true;
        defaultNetwork.settings.dns_enabled = true;
      };
      environment.systemPackages = [ pkgs.podman-compose ];
    })

    (lib.mkIf cfg.database.enable {
      # Listens on localhost only. `sudo -u postgres psql` to get started.
      services.postgresql.enable = true;
    })
  ];
}
