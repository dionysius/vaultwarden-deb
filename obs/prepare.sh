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
git-obs meta set --apiurl="$apiurl" --project="$project" --package="$package"

# Get package information using dpkg's Makefile helpers
eval "$(make -s -f /usr/share/dpkg/pkg-info.mk -f - <<'EOF'
all:
	@echo "DEB_SOURCE=$(DEB_SOURCE)"
	@echo "DEB_VERSION=$(DEB_VERSION)"
	@echo "DEB_VERSION_UPSTREAM=$(DEB_VERSION_UPSTREAM)"
EOF
)"

# Generate orig tarballs
gbp export-orig
git archive vendor/$DEB_VERSION_UPSTREAM -o ../${DEB_SOURCE}_${DEB_VERSION_UPSTREAM}.orig-vendor.tar.gz

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

echo "✓ Build ready: $BUILD_DIR"
