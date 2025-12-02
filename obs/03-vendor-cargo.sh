#!/bin/bash
# Vendor cargo dependencies

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

PACKAGE_NAME=$(dpkg-parsechangelog -S Source)
VERSION=$(dpkg-parsechangelog -S Version | cut -d- -f1)
ORIG_TARBALL="$REPO_ROOT/../${PACKAGE_NAME}_${VERSION}.orig.tar.gz"
VENDOR_TARBALL="$REPO_ROOT/../${PACKAGE_NAME}_${VERSION}.orig-vendor.tar.xz"

# Skip if vendor tarball already exists
if [ -f "$VENDOR_TARBALL" ]; then
    echo "Vendor tarball already exists: $(basename "$VENDOR_TARBALL") ($(du -h "$VENDOR_TARBALL" | cut -f1))"
    echo "Delete it to regenerate or skip this step."
    exit 0
fi

if [ ! -f "$ORIG_TARBALL" ]; then
    echo "Error: Orig tarball not found: $ORIG_TARBALL"
    echo "Run 02-generate-orig.sh first"
    exit 1
fi

VENDOR_SOURCE_DIR="/tmp/vaultwarden-vendor-$$"

echo "Extracting source for vendoring..."
mkdir -p "$VENDOR_SOURCE_DIR"
tar -xzf "$ORIG_TARBALL" -C "$VENDOR_SOURCE_DIR" --strip-components=1

echo "Vendoring cargo dependencies (this may take a while)..."
cd "$VENDOR_SOURCE_DIR"
cargo vendor

# Create vendor tarball (just the vendor directory)
echo "Creating vendor tarball: $(basename "$VENDOR_TARBALL")"
# Tarball contains vendored crates at root - dpkg-source extracts orig-vendor to vendor/
tar -cJf "$VENDOR_TARBALL" -C "$VENDOR_SOURCE_DIR/vendor" .

# Clean up
cd "$REPO_ROOT"
rm -rf "$VENDOR_SOURCE_DIR"

echo "✓ Created: $(basename "$VENDOR_TARBALL") ($(du -h "$VENDOR_TARBALL" | cut -f1))"
