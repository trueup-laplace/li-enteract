#!/bin/bash

# Script to clean up aggregate audio devices
# This script builds and runs the cleanup binary from its own workspace

set -e  # Exit on any error

echo "[cleanup_script] Building cleanup binary..."
cd cleanup_workspace

# Build the cleanup binary
cargo build

echo "[cleanup_script] Running cleanup..."
# Run the cleanup binary
./target/debug/cleanup_aggregate_devices

echo "[cleanup_script] Cleanup completed!"
