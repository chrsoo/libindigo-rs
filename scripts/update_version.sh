#!/bin/bash
# Update version metadata in all Cargo.toml files based on INDIGO version
#
# This script extracts the INDIGO version from the submodule and updates
# the SemVer build metadata in all workspace crates.
#
# Usage: ./scripts/update_version.sh [base_version]
#   base_version: Optional base version (e.g., "0.3.2"). If not provided,
#                 the current version from the root Cargo.toml is used.

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

INDIGO_SUBMODULE="sys/externals/indigo"
MAKEFILE="$INDIGO_SUBMODULE/Makefile"

# Check if INDIGO submodule exists
if [ ! -f "$MAKEFILE" ]; then
    echo -e "${RED}Error: INDIGO Makefile not found at $MAKEFILE${NC}"
    echo "Run: git submodule update --init --recursive"
    exit 1
fi

# Extract INDIGO version from Makefile
INDIGO_VERSION=$(grep "^INDIGO_VERSION" "$MAKEFILE" | sed 's/INDIGO_VERSION *= *//')
INDIGO_BUILD=$(grep "^INDIGO_BUILD" "$MAKEFILE" | sed 's/INDIGO_BUILD *= *//')

if [ -z "$INDIGO_VERSION" ] || [ -z "$INDIGO_BUILD" ]; then
    echo -e "${RED}Error: Could not extract INDIGO version from $MAKEFILE${NC}"
    exit 1
fi

INDIGO_FULL_VERSION="${INDIGO_VERSION}.${INDIGO_BUILD}"
BUILD_METADATA="INDIGO.${INDIGO_FULL_VERSION}"

echo -e "${GREEN}Extracted INDIGO version: ${INDIGO_FULL_VERSION}${NC}"

# Get base version from argument or current Cargo.toml
if [ -n "$1" ]; then
    BASE_VERSION="$1"
else
    BASE_VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version *= *"\([^+]*\).*/\1/')
fi

NEW_VERSION="${BASE_VERSION}+${BUILD_METADATA}"

echo -e "${YELLOW}Updating workspace versions to: ${NEW_VERSION}${NC}"

# Update root Cargo.toml
sed -i.bak "s|^version = \"[^\"]*\"|version = \"${NEW_VERSION}\"|" Cargo.toml
echo "  ✓ Updated Cargo.toml"

# Update sys/Cargo.toml
sed -i.bak "s|^version = \"[^\"]*\"|version = \"${NEW_VERSION}\"|" sys/Cargo.toml
echo "  ✓ Updated sys/Cargo.toml"

# Update rs/Cargo.toml (no build metadata for pure Rust crate)
sed -i.bak "s|^version = \"[^\"]*\"|version = \"${BASE_VERSION}\"|" rs/Cargo.toml
# Update dependency reference
sed -i.bak "s|libindigo = { path = \"\.\.\", version = \"[^\"]*\" }|libindigo = { path = \"..\", version = \"${BASE_VERSION}\" }|" rs/Cargo.toml
echo "  ✓ Updated rs/Cargo.toml"

# Update ffi/Cargo.toml (no build metadata for FFI wrapper)
sed -i.bak "s|^version = \"[^\"]*\"|version = \"${BASE_VERSION}\"|" ffi/Cargo.toml
# Update dependency references
sed -i.bak "s|libindigo = { path = \"\.\.\", version = \"[^\"]*\" }|libindigo = { path = \"..\", version = \"${BASE_VERSION}\" }|" ffi/Cargo.toml
sed -i.bak "s|libindigo-sys = { path = \"\.\./sys\", version = \"[^\"]*\" }|libindigo-sys = { path = \"../sys\", version = \"${BASE_VERSION}\" }|" ffi/Cargo.toml
echo "  ✓ Updated ffi/Cargo.toml"

# Clean up backup files
rm -f Cargo.toml.bak sys/Cargo.toml.bak rs/Cargo.toml.bak ffi/Cargo.toml.bak

echo -e "${GREEN}✓ Version update complete!${NC}"
echo ""
echo "Summary:"
echo "  INDIGO version: ${INDIGO_FULL_VERSION}"
echo "  Base version:   ${BASE_VERSION}"
echo "  Full version:   ${NEW_VERSION}"
echo ""
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Test build: cargo build"
echo "  3. Commit: git add -u && git commit -m 'Update to INDIGO ${INDIGO_FULL_VERSION}'"
