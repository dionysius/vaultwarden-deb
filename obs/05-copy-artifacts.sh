#!/bin/bash
# Copy all build artifacts to build directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_DIR="${BUILD_DIR:-$SCRIPT_DIR/build}"

cd "$REPO_ROOT"

echo "Preparing build directory: $BUILD_DIR"
mkdir -p "$BUILD_DIR"

# Find and copy the generated files
ORIG_TARBALL=$(ls -t ../*.orig.tar.* 2>/dev/null | grep -v 'orig-vendor' | head -1)
DSC_FILE=$(ls -t ../*.dsc 2>/dev/null | head -1)
DEBIAN_TARBALL=$(ls -t ../*.debian.tar.* 2>/dev/null | head -1)
VENDOR_TARBALL=$(ls -t ../*.orig-vendor.tar.* 2>/dev/null | head -1)

if [ -n "$ORIG_TARBALL" ]; then
    cp "$ORIG_TARBALL" "$BUILD_DIR/"
    echo "  ✓ $(basename "$ORIG_TARBALL")"
fi

if [ -n "$DSC_FILE" ]; then
    cp "$DSC_FILE" "$BUILD_DIR/"
    echo "  ✓ $(basename "$DSC_FILE")"
fi

if [ -n "$DEBIAN_TARBALL" ]; then
    cp "$DEBIAN_TARBALL" "$BUILD_DIR/"
    echo "  ✓ $(basename "$DEBIAN_TARBALL")"
fi

if [ -n "$VENDOR_TARBALL" ]; then
    cp "$VENDOR_TARBALL" "$BUILD_DIR/"
    echo "  ✓ $(basename "$VENDOR_TARBALL")"
else
    echo "  ⚠ Warning: No vendor tarball found (run 03-vendor-cargo.sh)"
fi

echo ""
echo "✓ OBS build environment ready in: $BUILD_DIR"
echo ""
echo "To build, run:"
echo "  cd $BUILD_DIR"
echo "  osc build <REPOSITORY> <ARCH>"
