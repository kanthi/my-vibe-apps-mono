Name:           tanux-logos
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux logos, icons, and boot splash assets
License:        MIT
BuildArch:      noarch

Source0:        tanux-logo.svg
Source1:        tanux-mark.svg

BuildRequires:  librsvg2-tools

Provides:       system-logos = %{version}-%{release}
Provides:       system-logos-icewm = %{version}-%{release}
Provides:       redhat-logos = %{version}-%{release}

Conflicts:      fedora-logos
Conflicts:      generic-logos

%description
Tanux logos, application icons, and Plymouth boot splash logo replacing
fedora-logos.

%package httpd
Summary:        Tanux httpd logo placeholder
Provides:       system-logos-httpd = %{version}-%{release}
Provides:       system-logos(httpd-logo-ng)
Conflicts:      fedora-logos-httpd
Conflicts:      generic-logos-httpd

%description httpd
Web server logo placeholder for Tanux.

%install
# System icon referenced by LOGO=tanux in /etc/os-release
install -Dm644 %{SOURCE0} %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/tanux.svg
for size in 48 128 256; do
  install -d %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps
  rsvg-convert -w $size %{SOURCE0} -o %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/tanux.png
done

install -d %{buildroot}%{_datadir}/pixmaps
ln -s ../icons/hicolor/256x256/apps/tanux.png %{buildroot}%{_datadir}/pixmaps/tanux.png

# Menu and system-logos compatibility icons
install -Dm644 %{SOURCE1} %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/fedora-logo-icon.svg
for linkname in start-here kmenu icon-panel-menu gnome-main-menu; do
  ln -s fedora-logo-icon.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/${linkname}.svg
done
install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/places
ln -s ../apps/start-here.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/places/start-here.svg

for size in 16 22 24 32 36 48 96 256; do
  install -d %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps
  install -d %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/places
  rsvg-convert -w $size %{SOURCE1} -o %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/fedora-logo-icon.png
  ln -s ../apps/fedora-logo-icon.png %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/places/start-here.png
done

# Pixmaps & Plymouth boot splash (spinner/bgrt uses system-logo-white.png and watermark.png)
rsvg-convert -w 256 %{SOURCE0} -o %{buildroot}%{_datadir}/pixmaps/fedora-logo.png
rsvg-convert -w 128 %{SOURCE0} -o %{buildroot}%{_datadir}/pixmaps/fedora-logo-small.png
rsvg-convert -w 160 %{SOURCE1} -o %{buildroot}%{_datadir}/pixmaps/fedora-logo-sprite.png
install -Dm644 %{SOURCE1} %{buildroot}%{_datadir}/pixmaps/fedora-logo-sprite.svg
rsvg-convert -w 160 %{SOURCE1} -o %{buildroot}%{_datadir}/pixmaps/fedora-gdm-logo.png
rsvg-convert -w 160 %{SOURCE1} -o %{buildroot}%{_datadir}/pixmaps/system-logo-white.png
rsvg-convert -w 128 %{SOURCE1} -o %{buildroot}%{_datadir}/pixmaps/poweredby.png

install -d %{buildroot}%{_datadir}/plymouth/themes/spinner
ln -s ../../../pixmaps/system-logo-white.png %{buildroot}%{_datadir}/plymouth/themes/spinner/watermark.png

%files
%{_datadir}/icons/hicolor/*/apps/*
%{_datadir}/icons/hicolor/*/places/*
%{_datadir}/pixmaps/*
%exclude %{_datadir}/pixmaps/poweredby.png
%{_datadir}/plymouth/themes/spinner/watermark.png

%files httpd
%{_datadir}/pixmaps/poweredby.png
