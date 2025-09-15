Hypr-showkey RPM packaging helper

This directory contains a RPM spec file and a small helper to produce a
source tarball that can be used with `rpmbuild`.

Files
- `hypr-showkey.spec` - RPM spec file for building the package.
- `make_sdist.sh` - Creates a source tarball named `hypr-showkey-<version>.tar.gz`

Quick steps to build an RPM (on a system with rpmbuild/cargo/rust installed):

1. Create the source tarball (from the root of the repository):

```zsh
./rpm/make_sdist.sh
# This produces hypr-showkey-<version>.tar.gz in the repo root
```

2. Copy the tarball to your rpmbuild SOURCES directory (common default is `~/rpmbuild/SOURCES`):

```zsh
mkdir -p ~/rpmbuild/SOURCES
cp hypr-showkey-*.tar.gz ~/rpmbuild/SOURCES/
```

3. Build the RPM:

```zsh
rpmbuild -ba rpm/hypr-showkey.spec
```

If you prefer not to copy the tarball, you can tell rpmbuild to use the current directory as the source dir:

```zsh
rpmbuild -ba --define "_sourcedir $(pwd)" rpm/hypr-showkey.spec
```

Notes
- The spec expects the compiled binary to be built with `cargo build --release` inside the source tree.
- The spec will also install `showkey.yaml` to `/etc/hypr-showkey/showkey.yaml` if present in the source tree.
