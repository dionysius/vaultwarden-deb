#!/bin/bash
# Generate Debian source package

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

echo "Generating Debian source package..."
# -i: ignore VCS patterns from .gitignore
# -b: build source package from directory
# --extend-diff-ignore: for OBS builds, ignore all upstream files (debian branch has no sources)
dpkg-source -i --extend-diff-ignore="^(?!debian/).*" -b .

PACKAGE_NAME=$(dpkg-parsechangelog -S Source)
VERSION=$(dpkg-parsechangelog -S Version)
DSC_FILE="../${PACKAGE_NAME}_${VERSION}.dsc"
DEBIAN_TARBALL="../${PACKAGE_NAME}_${VERSION}.debian.tar.xz"

if [ -f "$DSC_FILE" ] && [ -f "$DEBIAN_TARBALL" ]; then
    echo "✓ Created: $(basename "$DSC_FILE") ($(du -h "$DSC_FILE" | cut -f1))"
    echo "✓ Created: $(basename "$DEBIAN_TARBALL") ($(du -h "$DEBIAN_TARBALL" | cut -f1))"
else
    echo "✗ Failed to create Debian source package"
    exit 1
fi
