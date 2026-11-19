Name:           tanux-desktop-sway
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux Sway desktop defaults
License:        MIT
BuildArch:      noarch

Requires:       tanux-backgrounds
Requires:       sway
Requires:       swaybg
Requires:       kitty
Requires:       waybar
Requires:       dolphin

%description
Applies Tanux wallpaper and default terminal configuration on top of
Fedora's Sway configuration.

%install
install -d %{buildroot}%{_sysconfdir}/sway/config.d
cat > %{buildroot}%{_sysconfdir}/sway/config.d/90-tanux.conf << 'EOF'
# Tanux Sway defaults (overrides /etc/sway/config defaults cleanly)
output * bg /usr/share/backgrounds/tanux/tanux-dark.png fill
set $term kitty
EOF

%files
%dir %{_sysconfdir}/sway/config.d
%config(noreplace) %{_sysconfdir}/sway/config.d/90-tanux.conf
