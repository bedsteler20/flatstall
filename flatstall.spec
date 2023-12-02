%global srcname copr-tito-quickdoc

Name: flatstall
Version: 1.0.1
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

%description
Hellocopr is a very simple demonstration program that does nothing but display
some text on the command line. It is used as an example for automatic RPM
packaging using tito and Fedora's Copr user repository.

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
* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.1-1
- 

* Sat Dec 02 2023 Cameron Dehning <bedsteler2.0@gmail.com> 1.0.0-1
- new package built with tito

