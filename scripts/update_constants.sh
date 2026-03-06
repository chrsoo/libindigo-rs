#!/bin/bash
# Script to update INDIGO constants from the upstream library
#
# This script:
# 1. Updates the INDIGO git submodule to the latest version
# 2. Triggers the build system to regenerate src/constants.rs
# 3. Verifies the constants were generated successfully
#
# Usage: ./scripts/update_constants.sh

set -e

echo "==> Updating INDIGO submodule..."
git submodule update --init --recursive sys/externals/indigo

echo "==> Cleaning build artifacts..."
cargo clean

echo "==> Building with FFI features to trigger constant extraction..."
# The build.rs only extracts constants when FFI features are enabled
# We need to build a crate that has those features
cd sys && cargo build && cd ..

echo "==> Verifying constants were generated..."
if [ -f "src/constants.rs" ]; then
    line_count=$(wc -l < src/constants.rs)
    echo "✓ Constants file generated with $line_count lines"

    # Show first few lines to verify format
    echo ""
    echo "First 10 lines of generated constants:"
    head -10 src/constants.rs

    echo ""
    echo "✓ Constants successfully updated!"
    echo ""
    echo "Next steps:"
    echo "  1. Review the changes: git diff src/constants.rs"
    echo "  2. Test the build: cargo build"
    echo "  3. Commit the changes: git add src/constants.rs && git commit -m 'Update INDIGO constants'"
else
    echo "✗ Error: src/constants.rs was not generated"
    exit 1
fi
