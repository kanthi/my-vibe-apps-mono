Name:           tanux-security
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux Security Edition tool set
License:        MIT
BuildArch:      noarch

Requires:       tanux-desktop-kde
# Tool list intentionally empty for now, matching Tanix (modules/tanix/security.nix).
# Examples that exist in official Fedora 45 repositories:
# Requires:       nmap
# Requires:       wireshark
# Requires:       tcpdump
# Requires:       aircrack-ng
# Requires:       john
# Requires:       hashcat
# Requires:       hydra
# Requires:       gobuster

%description
Meta-package for the Tanux Security Edition. Installs on top of Tanux
KDE Plasma; the curated security tool list will be added here.

%files
