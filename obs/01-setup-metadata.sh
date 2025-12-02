#!/bin/bash
# Configure git-obs metadata

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
METADATA_FILE="$SCRIPT_DIR/metadata.json"

if [ ! -f "$METADATA_FILE" ]; then
    echo "Error: $METADATA_FILE not found"
    exit 1
fi

# Read metadata from JSON file
apiurl=$(jq -r '.apiurl' "$METADATA_FILE")
project=$(jq -r '.project' "$METADATA_FILE")
package=$(jq -r '.package' "$METADATA_FILE")

echo "Initializing git-obs metadata:"
echo "  API URL: $apiurl"
echo "  Project: $project"
echo "  Package: $package"

# Set git-obs metadata
git-obs meta set --apiurl="$apiurl" --project="$project" --package="$package"

echo "✓ git-obs metadata initialized successfully"
