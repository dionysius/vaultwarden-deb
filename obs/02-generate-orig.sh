#!/bin/bash
# Generate upstream orig tarball

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

echo "Generating orig tarball..."
gbp export-orig

PACKAGE_NAME=$(dpkg-parsechangelog -S Source)
VERSION=$(dpkg-parsechangelog -S Version | cut -d- -f1)
ORIG_TARBALL="../${PACKAGE_NAME}_${VERSION}.orig.tar.gz"

if [ -f "$ORIG_TARBALL" ]; then
    echo "✓ Created: $(basename "$ORIG_TARBALL") ($(du -h "$ORIG_TARBALL" | cut -f1))"
else
    echo "✗ Failed to create orig tarball"
    exit 1
fi
