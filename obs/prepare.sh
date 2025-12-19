#!/bin/bash
# Prepare all OBS build artifacts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$SCRIPT_DIR/build}"
METADATA_FILE="$SCRIPT_DIR/metadata.json"

RUST_VERSION=1.89.0
RUST_SHA256_x86_64=c4f2796b10ee886001f0799bc40caea38746403a33c379d77878c4f4683f9b51
RUST_SHA256_aarch64=ae6f35b027cb32339fa4ac94dab37a21194e9a5c680491d01e54aa61e9da4de7

cd "$REPO_ROOT"

# Setup git-obs metadata
if [ ! -f "$METADATA_FILE" ]; then
    echo "Error: $METADATA_FILE not found"
    exit 1
fi

apiurl=$(jq -r '.apiurl' "$METADATA_FILE")
project=$(jq -r '.project' "$METADATA_FILE")
package=$(jq -r '.package' "$METADATA_FILE")
git-obs meta set --apiurl="$apiurl" --project="$project" --package="$package"

# Get package information using dpkg's Makefile helpers
eval "$(make -s -f /usr/share/dpkg/default.mk -f - <<'EOF'
all:
	@echo "DEB_SOURCE=$(DEB_SOURCE)"
	@echo "DEB_VERSION=$(DEB_VERSION)"
	@echo "DEB_VERSION_UPSTREAM=$(DEB_VERSION_UPSTREAM)"
	@echo "DEB_BUILD_GNU_CPU=$(DEB_BUILD_GNU_CPU)"
	@echo "DEB_BUILD_ARCH_OS=$(DEB_BUILD_ARCH_OS)"
	@echo "DEB_BUILD_ARCH_LIBC=$(DEB_BUILD_ARCH_LIBC)"
EOF
)"

# Generate orig tarballs
gbp export-orig
echo "../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig.tar.gz: created"
git archive vendor/$DEB_VERSION_UPSTREAM -o ../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-vendor.tar.gz
echo "../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-vendor.tar.gz: created"

# Download additional binary files
for arch in amd64 arm64; do
    gnu_arch=$(dpkg-architecture -qDEB_TARGET_GNU_CPU -a$arch)
    rust_sha256_var="RUST_SHA256_${gnu_arch}"
    rust_sha256="${!rust_sha256_var}"
    rust_tarball="rust-${RUST_VERSION}-${gnu_arch}-unknown-${DEB_BUILD_ARCH_OS}-${DEB_BUILD_ARCH_LIBC}.tar.xz"
    rust_path="../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-rust-${arch}.tar.xz"
    if [ ! -f "$rust_path" ]; then
        curl -fsSL "https://static.rust-lang.org/dist/${rust_tarball}" -o "$rust_path"
    fi
    echo "${rust_sha256}  $rust_path" | sha256sum -c -
done

# Generate Debian source package
dpkg-source -i --extend-diff-ignore="^(?!debian/).*" -b .

# Prepare build directory and copy artifacts
mkdir -p "$BUILD_DIR"

ORIG_TARBALL="../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig.tar.gz"
VENDOR_TARBALL="../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-vendor.tar.gz"
DSC_FILE="../${DEB_SOURCE}_${DEB_VERSION}.dsc"
DEBIAN_TARBALL="../${DEB_SOURCE}_${DEB_VERSION}.debian.tar.xz"

# Verify and copy all required files
for file in "$ORIG_TARBALL" "$VENDOR_TARBALL" "$DSC_FILE" "$DEBIAN_TARBALL"; do
    if [ ! -f "$file" ]; then
        echo "Error: $(basename "$file") not found"
        exit 1
    fi
    mv "$file" "$BUILD_DIR/"
done

for arch in amd64 arm64; do
    rust_path="../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-rust-${arch}.tar.xz"
    if [ ! -f "$rust_path" ]; then
        echo "Error: $(basename "$rust_path") not found"
        exit 1
    fi
    cp "$rust_path" "$BUILD_DIR/"
done

echo "✓ Build ready: $BUILD_DIR"
