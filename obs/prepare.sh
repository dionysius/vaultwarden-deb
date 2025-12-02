#!/bin/bash
# Prepare all OBS build artifacts
# This is the main script that runs all preparation steps in order

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "========================================="
echo "OBS Build Preparation"
echo "========================================="
echo ""

# Step 1: Setup git-obs metadata
echo "Step 1/5: Setting up git-obs metadata"
echo "-----------------------------------------"
"$SCRIPT_DIR/01-setup-metadata.sh"
echo ""

# Step 2: Generate orig tarball
echo "Step 2/5: Generating orig tarball"
echo "-----------------------------------------"
"$SCRIPT_DIR/02-generate-orig.sh"
echo ""

# Step 3: Vendor cargo dependencies (most time-consuming)
echo "Step 3/5: Vendoring cargo dependencies"
echo "-----------------------------------------"
"$SCRIPT_DIR/03-vendor-cargo.sh"
echo ""

# Step 4: Generate Debian source package
echo "Step 4/5: Generating Debian source package"
echo "-----------------------------------------"
"$SCRIPT_DIR/04-generate-dsc.sh"
echo ""

# Step 5: Copy artifacts to build directory
echo "Step 5/5: Copying artifacts to build directory"
echo "-----------------------------------------"
"$SCRIPT_DIR/05-copy-artifacts.sh"
echo ""

echo "========================================="
echo "✓ All preparation steps completed"
echo "========================================="
