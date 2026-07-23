#
# spec file for package chord
#
# Copyright (c) 2026 Rodrigo Brito
#

Name:           chord
Version:        1.0.0
Release:        0
Summary:        Emulador de terminal do Lyra Enterprise Linux
License:        GPL-3.0-or-later
Group:          System/X11/Terminals
URL:            https://github.com/britors/Chord
Source0:        %{name}-%{version}.tar.zst
Source1:        vendor.tar.zst

BuildRequires:  appstream-glib
BuildRequires:  cargo
BuildRequires:  cargo-packaging
BuildRequires:  desktop-file-utils
BuildRequires:  gtk4-devel >= 4.18
BuildRequires:  libadwaita-devel >= 1.5
BuildRequires:  pkgconfig
BuildRequires:  rust >= 1.85
BuildRequires:  vte-devel >= 0.80
BuildRequires:  zstd

%description
Chord é o emulador de terminal do ecossistema Lyra Enterprise Linux. A interface
GTK4 oferece abas renomeáveis, divisões horizontais e verticais, busca no
histórico, zoom e preferências persistentes.

O núcleo compartilhado em Rust concentra temas, perfis de shell e configuração.
O tema padrão usa a paleta enterprise oficial do Lyra.

%prep
%autosetup -a1

%build
%{cargo_build}

%install
install -Dm0755 target/release/chord-gtk \
    %{buildroot}%{_bindir}/chord-gtk
install -Dm0644 data/org.lyraos.Chord.desktop \
    %{buildroot}%{_datadir}/applications/org.lyraos.Chord.desktop
install -Dm0644 data/org.lyraos.Chord.metainfo.xml \
    %{buildroot}%{_datadir}/metainfo/org.lyraos.Chord.metainfo.xml
install -Dm0644 data/icons/chord-icon-mark.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/chord-icon-mark.svg

desktop-file-validate \
    %{buildroot}%{_datadir}/applications/org.lyraos.Chord.desktop
appstream-util validate-relax --nonet \
    %{buildroot}%{_datadir}/metainfo/org.lyraos.Chord.metainfo.xml

%check
cargo test --offline -p chord-core

%post
%desktop_database_post
%icon_theme_cache_post

%postun
%desktop_database_postun
%icon_theme_cache_postun

%files
%license LICENSE
%doc README.md
%{_bindir}/chord-gtk
%{_datadir}/applications/org.lyraos.Chord.desktop
%{_datadir}/metainfo/org.lyraos.Chord.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/chord-icon-mark.svg

%changelog
