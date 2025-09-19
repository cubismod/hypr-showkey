Name:           hypr-showkey
Version:        0.1.0
Release:        1%{?dist}
Summary:        A TUI application to display and search Hyprland keybindings

License:        Apache-2.0
URL:            https://github.com/cubismod/hypr-showkey
Source0:        %{name}-%{version}.tar.gz
# If you use COPR or another builder that fetches sources remotely, you can
# replace the local tarball with a GitHub archive URL (uncomment and adjust
# if you tag releases):
# Source0:        https://github.com/cubismod/hypr-showkey/archive/refs/tags/v%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust

%description
Hypr-showkey is a fast, configurable TUI application for displaying and
searching Hyprland keybindings with fuzzy search functionality.

%prep
# Expect a source tarball named %{name}-%{version}.tar.gz to be placed in
# the rpmbuild SOURCES directory (or use _sourcedir to point at the repo).
%setup -q -n %{name}-%{version}

%build
export CARGO_HOME="%{_builddir}/.cargo"
# Use the source tree's Cargo.toml and build a release binary.
cargo build --release --locked --target-dir=target

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}%{_bindir}
install -m 0755 target/release/hypr-showkey %{buildroot}%{_bindir}/hypr-showkey

mkdir -p %{buildroot}%{_docdir}/%{name}-%{version}
install -m 0644 README.md %{buildroot}%{_docdir}/%{name}-%{version}/README.md
install -m 0644 LICENSE %{buildroot}%{_docdir}/%{name}-%{version}/LICENSE

mkdir -p %{buildroot}%{_sysconfdir}/%{name}
if [ -f showkey.yaml ]; then
  install -m 0644 showkey.yaml %{buildroot}%{_sysconfdir}/%{name}/showkey.yaml
fi

%files
%license LICENSE
%doc README.md
%config(noreplace) %{_sysconfdir}/%{name}/showkey.yaml
%{_bindir}/hypr-showkey

%changelog
* Aug 11 2025 Ryan Wallace <git@hexa.mozmail.com>
- initial version
