Name:           tanux-desktop-kde
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux KDE Plasma desktop layout and defaults
License:        MIT
BuildArch:      noarch

Requires:       tanux-backgrounds
Requires:       plasma-workspace
Requires:       systemd

%description
Configures KDE Plasma with the Tanux macOS-style layout (top menu bar,
floating bottom dock, left window buttons) and default Tanux wallpaper
using only stock Plasma components.

%install
install -d %{buildroot}%{_datadir}/tanux
cat > %{buildroot}%{_datadir}/tanux/plasma-layout.js << 'EOF'
var old = panels();
for (var i = 0; i < old.length; i++) {
  old[i].remove();
}

var bar = new Panel;
bar.location = "top";
bar.height = 28;
bar.floating = false;
bar.addWidget("org.kde.plasma.kickoff");
bar.addWidget("org.kde.plasma.appmenu");
bar.addWidget("org.kde.plasma.panelspacer");
bar.addWidget("org.kde.plasma.systemtray");
bar.addWidget("org.kde.plasma.digitalclock");

var dock = new Panel;
dock.location = "bottom";
dock.height = 56;
dock.lengthMode = "fit";
dock.alignment = "center";
dock.floating = true;
dock.hiding = "dodgewindows";
dock.addWidget("org.kde.plasma.icontasks");
EOF

install -d %{buildroot}%{_libexecdir}
cat > %{buildroot}%{_libexecdir}/tanux-plasma-setup << 'EOF'
#!/bin/sh
set -eu
stamp="${XDG_STATE_HOME:-$HOME/.local/state}/tanux/plasma-setup-done"
[ -e "$stamp" ] && exit 0

for attempt in $(seq 1 30); do
  sleep 2
  if plasma-apply-wallpaperimage /usr/share/backgrounds/tanux/tanux-dark.png; then
    busctl --user call org.kde.plasmashell /PlasmaShell \
      org.kde.PlasmaShell evaluateScript s "$(cat /usr/share/tanux/plasma-layout.js)" || true
    mkdir -p "$(dirname "$stamp")"
    touch "$stamp"
    exit 0
  fi
done
exit 1
EOF
chmod 0755 %{buildroot}%{_libexecdir}/tanux-plasma-setup

install -d %{buildroot}%{_sysconfdir}/xdg/autostart
cat > %{buildroot}%{_sysconfdir}/xdg/autostart/tanux-plasma-setup.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Tanux desktop setup
Exec=/usr/libexec/tanux-plasma-setup
OnlyShowIn=KDE;
NoDisplay=true
X-KDE-autostart-phase=2
EOF

cat > %{buildroot}%{_sysconfdir}/xdg/kwinrc << 'EOF'
[org.kde.kdecoration2]
ButtonsOnLeft=XIA
ButtonsOnRight=
EOF

%files
%{_datadir}/tanux/plasma-layout.js
%{_libexecdir}/tanux-plasma-setup
%{_sysconfdir}/xdg/autostart/tanux-plasma-setup.desktop
%config(noreplace) %{_sysconfdir}/xdg/kwinrc
