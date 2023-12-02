%global srcname copr-tito-quickdoc

Name: flatstall
Version: 1.0.3
Release: 1%{?dist}
License: GPLv3
Summary: A Sideloader for Flatpak
Url: https://pagure.io/%{srcname}
# Sources can be obtained by
# git clone https://pagure.io/copr-tito-quickdoc
# cd copr-tito-quickdoc
# tito build --tgz
Source0: %{name}-%{version}.tar.gz

BuildArch: x86_64

BuildRequires: blueprint-compiler
BuildRequires: cargo
BuildRequires: rustc
BuildRequires: meson
BuildRequires: ninja-build
BuildRequires: flatpak-devel
BuildRequires: libadwaita-devel
BuildRequires: gtk4-devel

Requires: flatpak
Requires: libadwaita

%description
Flatstall is a sideloader for Flatpak.

#-- PREP, BUILD & INSTALL -----------------------------------------------------#
%prep
%autosetup

%build
%meson
%meson_build

%install
%meson_install

#-- FILES ---------------------------------------------------------------------#
%files
%doc README.md
%license LICENSE
%{_bindir}/flatstall
%{_datadir}/applications/dev.bedsteler20.Flatstall.desktop
%{_datadir}/metainfo/dev.bedsteler20.Flatstall.metainfo.xml

#-- CHANGELOG -----------------------------------------------------------------#
%changelog
* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.3-1
- Remove gnome post install (bedsteler2.0@gmail.com)

* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.2-1
- Updated Dependencys 

* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.1-1
- 

* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.0-1
- new package built with tito

