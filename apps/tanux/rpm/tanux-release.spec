%global dist_version 45

Name:           tanux-release
Version:        %{dist_version}
Release:        1%{?dist}
Summary:        Tanux release files
License:        MIT
BuildArch:      noarch

Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Requires:       fedora-repos(%{dist_version})

Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}

Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description
Tanux release identification files (/etc/os-release, /etc/system-release,
RPM macros and DNF5 defaults) replacing fedora-release.

%package common
Summary:        Tanux release common files
Requires:       tanux-release-variant = %{version}-%{release}
Requires:       fedora-repos(%{dist_version})
Conflicts:      fedora-release-common
Conflicts:      generic-release-common

%description common
Files shared across all Tanux variants.

%package kde
Summary:        Tanux release files (KDE Plasma)
Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}
Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description kde
Release identity files for Tanux KDE Plasma.

%package sway
Summary:        Tanux release files (Sway)
Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}
Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description sway
Release identity files for Tanux Sway.

%package server
Summary:        Tanux release files (Server Minimal)
Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}
Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description server
Release identity files for Tanux Server Minimal.

%package server-web
Summary:        Tanux release files (Server Web)
Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}
Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description server-web
Release identity files for Tanux Server Web (Caddy).

%package security
Summary:        Tanux release files (Security Edition)
Requires:       tanux-release-common = %{version}-%{release}
Requires:       distribution-systemd-presets
Provides:       system-release
Provides:       system-release(%{dist_version})
Provides:       system-release(releasever) = %{dist_version}
Provides:       system-release-product
Provides:       tanux-release-variant = %{version}-%{release}
Conflicts:      system-release
Conflicts:      fedora-release
Conflicts:      fedora-release-variant
Conflicts:      fedora-release-identity
Conflicts:      generic-release
Conflicts:      tanux-release-variant

%description security
Release identity files for Tanux Security Edition.

%install
install -d %{buildroot}%{_prefix}/lib/os.release.d
install -d %{buildroot}%{_sysconfdir}

echo "Tanux release %{dist_version} (Fedora %{dist_version} base)" > %{buildroot}%{_prefix}/lib/fedora-release
echo "cpe:/o:tanux:tanux:%{dist_version}" > %{buildroot}%{_prefix}/lib/system-release-cpe

ln -s ../usr/lib/fedora-release %{buildroot}%{_sysconfdir}/fedora-release
ln -s ../usr/lib/system-release-cpe %{buildroot}%{_sysconfdir}/system-release-cpe
ln -s fedora-release %{buildroot}%{_sysconfdir}/redhat-release
ln -s fedora-release %{buildroot}%{_sysconfdir}/system-release
ln -s ../usr/lib/os-release %{buildroot}%{_sysconfdir}/os-release

write_os_release() {
  local out="$1"
  local variant="$2"
  local variant_id="$3"
  cat > "$out" <<EOF
NAME=Tanux
VERSION="%{dist_version}${variant:+ ($variant)}"
ID=tanux
ID_LIKE=fedora
VERSION_ID=%{dist_version}
PRETTY_NAME="Tanux %{dist_version}${variant:+ ($variant)}"
ANSI_COLOR="0;35"
LOGO=tanux
CPE_NAME="cpe:/o:tanux:tanux:%{dist_version}"
HOME_URL="https://fedoraproject.org/"
SUPPORT_URL="https://docs.fedoraproject.org/"
BUG_REPORT_URL="https://bugzilla.redhat.com/"
EOF
  if [ -n "$variant" ]; then
    echo "VARIANT=\"$variant\"" >> "$out"
    echo "VARIANT_ID=$variant_id" >> "$out"
  fi
}

write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-default "" ""
write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-kde "KDE Plasma" "kde"
write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-sway "Sway" "sway"
write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-server "Server Minimal" "server"
write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-server-web "Server Web" "server-web"
write_os_release %{buildroot}%{_prefix}/lib/os.release.d/os-release-security "Security Edition" "security"
touch %{buildroot}%{_prefix}/lib/os-release

echo "Tanux %{dist_version} (\l)" > %{buildroot}%{_prefix}/lib/issue
echo "Kernel \r on \m" >> %{buildroot}%{_prefix}/lib/issue
ln -s ../usr/lib/issue %{buildroot}%{_sysconfdir}/issue

echo "Tanux %{dist_version}" > %{buildroot}%{_prefix}/lib/issue.net
echo "Kernel \r on \m" >> %{buildroot}%{_prefix}/lib/issue.net
ln -s ../usr/lib/issue.net %{buildroot}%{_sysconfdir}/issue.net

install -d -m 755 %{buildroot}%{_rpmconfigdir}/macros.d
cat > %{buildroot}%{_rpmconfigdir}/macros.d/macros.dist << 'EOF'
%fedora                45
%dist                %{?distprefix}.fc45%{?with_bootstrap:~bootstrap}
%fc45                1
EOF

install -d %{buildroot}%{_datadir}/dnf5/libdnf.conf.d
cat > %{buildroot}%{_datadir}/dnf5/libdnf.conf.d/20-defaults.conf << 'EOF'
[main]
allow_vendor_change=False
best=False
pkg_gpgcheck=True
skip_if_unavailable=True
EOF

install -d %{buildroot}%{_datadir}/dnf5/vendors.d
cat > %{buildroot}%{_datadir}/dnf5/vendors.d/allow-cmdline.conf << 'EOF'
version = '1.1'

[[incoming_packages]]
filters = [
  { filter = 'cmdline_repo', value = 'true' }
]
EOF

install -d %{buildroot}%{_datadir}/dnf/plugins
cat > %{buildroot}%{_datadir}/dnf/plugins/copr.vendor.conf << 'EOF'
[main]
distribution = fedora
releasever = 45
EOF

%files common
%dir %{_prefix}/lib/os.release.d
%ghost %{_prefix}/lib/os-release
%{_prefix}/lib/fedora-release
%{_prefix}/lib/system-release-cpe
%{_sysconfdir}/os-release
%{_sysconfdir}/fedora-release
%{_sysconfdir}/redhat-release
%{_sysconfdir}/system-release
%{_sysconfdir}/system-release-cpe
%attr(0644,root,root) %{_prefix}/lib/issue
%config(noreplace) %{_sysconfdir}/issue
%attr(0644,root,root) %{_prefix}/lib/issue.net
%config(noreplace) %{_sysconfdir}/issue.net
%attr(0644,root,root) %{_rpmconfigdir}/macros.d/macros.dist
%dir %{_datadir}/dnf5
%dir %{_datadir}/dnf5/libdnf.conf.d
%{_datadir}/dnf5/libdnf.conf.d/20-defaults.conf
%dir %{_datadir}/dnf5/vendors.d
%{_datadir}/dnf5/vendors.d/allow-cmdline.conf
%dir %{_datadir}/dnf
%dir %{_datadir}/dnf/plugins
%{_datadir}/dnf/plugins/copr.vendor.conf

%files
%{_prefix}/lib/os.release.d/os-release-default
%post -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-default", "%{_prefix}/lib/os-release")

%files kde
%{_prefix}/lib/os.release.d/os-release-kde
%post kde -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-kde", "%{_prefix}/lib/os-release")

%files sway
%{_prefix}/lib/os.release.d/os-release-sway
%post sway -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-sway", "%{_prefix}/lib/os-release")

%files server
%{_prefix}/lib/os.release.d/os-release-server
%post server -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-server", "%{_prefix}/lib/os-release")

%files server-web
%{_prefix}/lib/os.release.d/os-release-server-web
%post server-web -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-server-web", "%{_prefix}/lib/os-release")

%files security
%{_prefix}/lib/os.release.d/os-release-security
%post security -p <lua>
os.remove("%{_prefix}/lib/os-release")
posix.symlink("os.release.d/os-release-security", "%{_prefix}/lib/os-release")
