"""Single source of truth for the five Tanux installer editions.

Both the Kickstart files (/ks/<id>.ks) and the GRUB boot menu (/EFI/BOOT/grub.cfg)
are generated from this table so they can never drift out of sync.
"""

DISTRO_NAME = "Tanux"
FEDORA_VERSION = "45"
ISO_LABEL = "TANUX_45"

COMMON_EXCLUDES = [
    "fedora-release*",
    "fedora-logos*",
    "generic-release*",
    "generic-logos*",
]

SERVER_SSHD_POST = [
    "mkdir -p /etc/ssh/sshd_config.d",
    "printf 'PermitRootLogin no\\n' > /etc/ssh/sshd_config.d/50-tanux.conf",
]

EDITIONS = [
    {
        "id": "kde",
        "title": "Desktop: KDE Plasma",
        "grub_label": "Install Tanux: KDE Plasma",
        "packages": [
            "@kde-desktop-environment",
            "kernel",
            "kate",
            "firefox",
            "tanux-release-kde",
            "tanux-logos",
            "tanux-desktop-kde",
        ],
        "services": ["NetworkManager", "plasmalogin"],
        "firewall_services": [],
        "post": ["systemctl set-default graphical.target"],
    },
    {
        "id": "sway",
        "title": "Desktop: Sway",
        "grub_label": "Install Tanux: Sway",
        "packages": [
            "@sway-desktop-environment",
            "kernel",
            "kitty",
            "dolphin",
            "tanux-release-sway",
            "tanux-logos",
            "tanux-desktop-sway",
        ],
        "services": ["NetworkManager", "sddm"],
        "firewall_services": [],
        "post": ["systemctl set-default graphical.target"],
    },
    {
        "id": "server-minimal",
        "title": "Server: Minimal",
        "grub_label": "Install Tanux: Server Minimal",
        "packages": [
            "@core",
            "@standard",
            "kernel",
            "openssh-server",
            "NetworkManager",
            "firewalld",
            "tanux-release-server",
            "tanux-logos",
        ],
        "services": ["NetworkManager", "sshd", "firewalld"],
        "firewall_services": ["ssh"],
        "post": [
            "systemctl set-default multi-user.target",
            *SERVER_SSHD_POST,
        ],
    },
    {
        "id": "server-web",
        "title": "Server: Web (Caddy)",
        "grub_label": "Install Tanux: Server Web (Caddy)",
        "packages": [
            "@core",
            "@standard",
            "kernel",
            "openssh-server",
            "NetworkManager",
            "firewalld",
            "tanux-release-server-web",
            "tanux-logos",
            "tanux-server-web",
        ],
        "services": ["NetworkManager", "sshd", "firewalld", "caddy"],
        "firewall_services": ["ssh", "http", "https"],
        "post": [
            "systemctl set-default multi-user.target",
            *SERVER_SSHD_POST,
        ],
    },
    {
        "id": "security",
        "title": "Security Edition",
        "grub_label": "Install Tanux: Security Edition",
        "packages": [
            "@kde-desktop-environment",
            "kernel",
            "kate",
            "firefox",
            "tanux-release-security",
            "tanux-logos",
            "tanux-security",
        ],
        "services": ["NetworkManager", "plasmalogin"],
        "firewall_services": [],
        "post": ["systemctl set-default graphical.target"],
    },
]
