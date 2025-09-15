#!/usr/bin/env bash
set -euo pipefail

# create a source tarball named hypr-showkey-<version>.tar.gz
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

version=$(awk -F '"' '/^version/ {print $2; exit}' Cargo.toml || true)
if [ -z "$version" ]; then
  echo "Could not determine version from Cargo.toml"
  exit 1
fi

name="hypr-showkey"
tarball="${name}-${version}.tar.gz"

echo "Creating source tarball: $tarball"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

# copy files we need
mkdir -p "$tmpdir/${name}-${version}"
cp -r Cargo.toml src README.md LICENSE showkey.yaml "$tmpdir/${name}-${version}/" || true

pushd "$tmpdir" >/dev/null
tar -czf "$ROOT_DIR/$tarball" "${name}-${version}"
popd >/dev/null

echo "Created $tarball"
