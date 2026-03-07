#!/bin/bash
# Build script for libindigo-relm on macOS
# This script ensures the correct pkg-config is used to find GTK4 libraries

set -e

# Use Homebrew's pkg-config which knows about Homebrew library paths
export PKG_CONFIG=/opt/homebrew/bin/pkg-config

# Verify pkg-config can find required libraries
echo "Checking for required libraries..."
$PKG_CONFIG --modversion gtk4 || { echo "Error: gtk4 not found"; exit 1; }
$PKG_CONFIG --modversion pango || { echo "Error: pango not found"; exit 1; }
$PKG_CONFIG --modversion cairo || { echo "Error: cairo not found"; exit 1; }

echo "All required libraries found:"
echo "  GTK4: $($PKG_CONFIG --modversion gtk4)"
echo "  Pango: $($PKG_CONFIG --modversion pango)"
echo "  Cairo: $($PKG_CONFIG --modversion cairo)"
echo ""

# Build the project
echo "Building libindigo-relm..."
cargo build "$@"
