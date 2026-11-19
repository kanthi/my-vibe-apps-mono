Name:           tanux-server-web
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux Web Server (Caddy) configuration
License:        MIT
BuildArch:      noarch

Requires:       caddy
Requires:       firewalld

%description
Configures Caddy to serve /var/www on port 80, creates /var/www via
systemd-tmpfiles, enables caddy.service via preset, and opens HTTP/HTTPS
ports in firewalld.

%install
install -d %{buildroot}%{_sysconfdir}/tanux
cat > %{buildroot}%{_sysconfdir}/tanux/Caddyfile << 'EOF'
# Tanux Web Server default Caddyfile.
# Replace ":80" with your domain name to enable automatic HTTPS.
:80 {
	root * /var/www
	file_server
}
EOF

install -d %{buildroot}%{_sysconfdir}/systemd/system/caddy.service.d
cat > %{buildroot}%{_sysconfdir}/systemd/system/caddy.service.d/tanux.conf << 'EOF'
[Service]
ExecStartPre=
ExecStartPre=/usr/bin/caddy validate --config /etc/tanux/Caddyfile
ExecStart=
ExecStart=/usr/bin/caddy run --environ --config /etc/tanux/Caddyfile
ExecReload=
ExecReload=/usr/bin/caddy reload --config /etc/tanux/Caddyfile
EOF

install -d %{buildroot}%{_tmpfilesdir}
cat > %{buildroot}%{_tmpfilesdir}/tanux-server-web.conf << 'EOF'
d /var/www 0755 caddy caddy -
EOF

install -d %{buildroot}%{_presetdir}
cat > %{buildroot}%{_presetdir}/80-tanux-server-web.preset << 'EOF'
enable caddy.service
EOF

%post
if [ -x /usr/bin/firewall-offline-cmd ]; then
  /usr/bin/firewall-offline-cmd --add-service=http --add-service=https >/dev/null 2>&1 || :
fi

%files
%dir %{_sysconfdir}/tanux
%config(noreplace) %{_sysconfdir}/tanux/Caddyfile
%dir %{_sysconfdir}/systemd/system/caddy.service.d
%{_sysconfdir}/systemd/system/caddy.service.d/tanux.conf
%{_tmpfilesdir}/tanux-server-web.conf
%{_presetdir}/80-tanux-server-web.preset
