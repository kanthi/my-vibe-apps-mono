Name:           tanux-backgrounds
Version:        45.0
Release:        1%{?dist}
Summary:        Tanux desktop wallpapers (dark and light)
License:        CC-BY-SA-4.0
BuildArch:      noarch

Source0:        tanux-dark.svg
Source1:        tanux-light.svg

BuildRequires:  librsvg2-tools

%description
Tanux 4K wallpapers installed for standard desktops, KDE Plasma, and GNOME.

%install
bg=%{buildroot}%{_datadir}/backgrounds/tanux
install -d "$bg" %{buildroot}%{_datadir}/gnome-background-properties

rsvg-convert -w 3840 -h 2160 %{SOURCE0} -o "$bg/tanux-dark.png"
rsvg-convert -w 3840 -h 2160 %{SOURCE1} -o "$bg/tanux-light.png"
ln -s tanux/tanux-dark.png %{buildroot}%{_datadir}/backgrounds/default.png

for variant in dark light; do
  name="Tanux-$variant"
  kde="%{buildroot}%{_datadir}/wallpapers/$name"
  install -d "$kde/contents/images"
  ln -s "../../../backgrounds/tanux/tanux-$variant.png" "$kde/contents/images/3840x2160.png"
  cat > "$kde/metadata.json" <<EOF
{
  "KPlugin": {
    "Id": "$name",
    "Name": "Tanux ($variant)",
    "License": "CC-BY-SA-4.0"
  }
}
EOF
done

cat > %{buildroot}%{_datadir}/gnome-background-properties/tanux.xml << 'EOF'
<?xml version="1.0"?>
<!DOCTYPE wallpapers SYSTEM "gnome-wp-list.dtd">
<wallpapers>
  <wallpaper deleted="false">
    <name>Tanux</name>
    <filename>/usr/share/backgrounds/tanux/tanux-light.png</filename>
    <filename-dark>/usr/share/backgrounds/tanux/tanux-dark.png</filename-dark>
    <options>zoom</options>
    <shade_type>solid</shade_type>
    <pcolor>#6C42BD</pcolor>
    <scolor>#120826</scolor>
  </wallpaper>
</wallpapers>
EOF

%files
%{_datadir}/backgrounds/default.png
%{_datadir}/backgrounds/tanux/
%{_datadir}/wallpapers/Tanux-dark/
%{_datadir}/wallpapers/Tanux-light/
%{_datadir}/gnome-background-properties/tanux.xml
