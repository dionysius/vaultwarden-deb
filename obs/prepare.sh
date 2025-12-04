#!/bin/bash
# Prepare all OBS build artifacts

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$SCRIPT_DIR/build}"
METADATA_FILE="$SCRIPT_DIR/metadata.json"

cd "$REPO_ROOT"

# Setup git-obs metadata
if [ ! -f "$METADATA_FILE" ]; then
    echo "Error: $METADATA_FILE not found"
    exit 1
fi

apiurl=$(jq -r '.apiurl' "$METADATA_FILE")
project=$(jq -r '.project' "$METADATA_FILE")
package=$(jq -r '.package' "$METADATA_FILE")
git-obs meta set --apiurl="$apiurl" --project="$project" --package="$package" >/dev/null 2>&1

# Generate orig tarballs (main + vendor)
gbp export-orig >/dev/null 2>&1

# Generate Debian source package
dpkg-source -i --extend-diff-ignore="^(?!debian/).*" -b . >/dev/null 2>&1

# Prepare build directory and copy artifacts
PACKAGE_NAME=$(dpkg-parsechangelog -S Source)
VERSION=$(dpkg-parsechangelog -S Version)
UPSTREAM_VERSION=$(echo "$VERSION" | cut -d- -f1)

mkdir -p "$BUILD_DIR"

ORIG_TARBALL="../${PACKAGE_NAME}_${UPSTREAM_VERSION}.orig.tar.gz"
VENDOR_TARBALL="../${PACKAGE_NAME}_${UPSTREAM_VERSION}.orig-vendor.tar.xz"
DSC_FILE="../${PACKAGE_NAME}_${VERSION}.dsc"
DEBIAN_TARBALL="../${PACKAGE_NAME}_${VERSION}.debian.tar.xz"

# Verify and copy all required files
for file in "$ORIG_TARBALL" "$VENDOR_TARBALL" "$DSC_FILE" "$DEBIAN_TARBALL"; do
    if [ ! -f "$file" ]; then
        echo "Error: $(basename "$file") not found"
        exit 1
    fi
    cp "$file" "$BUILD_DIR/"
done

echo "✓ Build ready: $BUILD_DIR"
