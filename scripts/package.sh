#!/bin/bash
set -euo pipefail

# Inputs:
# 1. VERSION (e.g. 0.1.0)
# 2. BIN_AMD64 (path to x86_64-unknown-linux-gnu binary)
# 3. BIN_ARM64 (path to aarch64-unknown-linux-gnu binary)
# 4. OUT_DIR (path to write generated packages)

if [ "$#" -ne 4 ]; then
  echo "Error: Missing arguments."
  echo "Usage: $0 <version> <bin_amd64> <bin_arm64> <out_dir>"
  exit 1
fi

VERSION="$1"
BIN_AMD64="$2"
BIN_ARM64="$3"
OUT_DIR="$4"

mkdir -p "$OUT_DIR"

echo "=== Generating Debian (.deb) packages ==="

# Build AMD64 DEB
echo "Building deb for amd64..."
DEB_AMD64_DIR="deb_amd64_build"
mkdir -p "$DEB_AMD64_DIR/usr/bin" "$DEB_AMD64_DIR/DEBIAN"
cp "$BIN_AMD64" "$DEB_AMD64_DIR/usr/bin/kdc"
chmod 755 "$DEB_AMD64_DIR/usr/bin/kdc"

cat <<EOF > "$DEB_AMD64_DIR/DEBIAN/control"
Package: kdc
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Utkarsh Patrikar <utkarsh232005@gmail.com>
Description: Kubernetes Docker Commander, a project-centric DevOps TUI
EOF

dpkg-deb --build "$DEB_AMD64_DIR" "$OUT_DIR/kdc_${VERSION}_amd64.deb"
rm -rf "$DEB_AMD64_DIR"

# Build ARM64 DEB
echo "Building deb for arm64..."
DEB_ARM64_DIR="deb_arm64_build"
mkdir -p "$DEB_ARM64_DIR/usr/bin" "$DEB_ARM64_DIR/DEBIAN"
cp "$BIN_ARM64" "$DEB_ARM64_DIR/usr/bin/kdc"
chmod 755 "$DEB_ARM64_DIR/usr/bin/kdc"

cat <<EOF > "$DEB_ARM64_DIR/DEBIAN/control"
Package: kdc
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: arm64
Maintainer: Utkarsh Patrikar <utkarsh232005@gmail.com>
Description: Kubernetes Docker Commander, a project-centric DevOps TUI
EOF

dpkg-deb --build "$DEB_ARM64_DIR" "$OUT_DIR/kdc_${VERSION}_arm64.deb"
rm -rf "$DEB_ARM64_DIR"

echo "=== Generating RPM packages ==="

# Install rpm tools if not present
if ! command -v rpmbuild &> /dev/null; then
  echo "Installing rpm tool..."
  sudo apt-get update && sudo apt-get install -y rpm
fi

# Create rpmbuild directories
RPM_BUILD_DIR="$(pwd)/rpmbuild"
mkdir -p "$RPM_BUILD_DIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

# Write spec template
cat <<'EOF' > kdc.spec
%global __strip /bin/true
%global debug_package %{nil}
%global __os_install_post %{nil}

Name:           kdc
Version:        %{version}
Release:        1
Summary:        Kubernetes Docker Commander, a project-centric DevOps TUI
License:        MIT
URL:            https://github.com/KDM-cli/kdc-cli

%description
Kubernetes Docker Commander, a project-centric DevOps TUI

%prep
%build

%install
mkdir -p %{buildroot}/usr/bin
cp %{binary_source} %{buildroot}/usr/bin/kdc
chmod 755 %{buildroot}/usr/bin/kdc

%files
/usr/bin/kdc
EOF

# Build AMD64 RPM
echo "Building RPM for x86_64..."
rpmbuild -bb --target x86_64-linux \
  --define "version ${VERSION}" \
  --define "binary_source $(pwd)/$BIN_AMD64" \
  --define "_topdir $RPM_BUILD_DIR" \
  kdc.spec

# Build ARM64 RPM
echo "Building RPM for aarch64..."
rpmbuild -bb --target aarch64-linux \
  --define "version ${VERSION}" \
  --define "binary_source $(pwd)/$BIN_ARM64" \
  --define "_topdir $RPM_BUILD_DIR" \
  kdc.spec

# Copy generated RPMs to output directory
find "$RPM_BUILD_DIR"/RPMS -name '*.rpm' -exec cp {} "$OUT_DIR/" \;
rm -rf "$RPM_BUILD_DIR" kdc.spec

echo "Package generation completed successfully."
