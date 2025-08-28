#!/bin/bash

# Phase 2 Test Runner for macOS Audio Loopback
# This script runs comprehensive tests for device format detection

set -e

echo "=========================================="
echo "Phase 2: Device Format Detection Tests"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Check if we're in the right directory
if [ ! -f "src-tauri/Cargo.toml" ]; then
    print_status $RED "Error: Must run from li-enteract directory"
    exit 1
fi

print_status $BLUE "Running Phase 2 tests for macOS audio loopback..."

# Run the basic device enumeration tests first
print_status $YELLOW "1. Running basic device enumeration tests..."
cd src-tauri
cargo test device_enumeration -- --nocapture 2>&1 | grep -E "\[PHASE1\]|test result|running|passed|failed"

# Run the comprehensive Phase 2 tests
print_status $YELLOW "2. Running comprehensive device format detection tests..."
cargo test phase2_tests::phase2_tests::test_comprehensive_device_format_detection -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "3. Running device transport type tests..."
cargo test phase2_tests::phase2_tests::test_device_transport_types -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "4. Running device capability analysis tests..."
cargo test phase2_tests::phase2_tests::test_device_capability_analysis -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "5. Running format compatibility validation tests..."
cargo test phase2_tests::phase2_tests::test_format_compatibility_validation -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "6. Running device enumeration performance tests..."
cargo test phase2_tests::phase2_tests::test_device_enumeration_performance -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "7. Running error handling tests..."
cargo test phase2_tests::phase2_tests::test_error_handling_invalid_devices -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

print_status $YELLOW "8. Running device sorting and prioritization tests..."
cargo test phase2_tests::phase2_tests::test_device_sorting_and_prioritization -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

# Run all Phase 2 tests together
print_status $YELLOW "9. Running all Phase 2 tests together..."
cargo test phase2_tests -- --nocapture 2>&1 | grep -E "\[PHASE2\]|test result|running|passed|failed"

cd ..

print_status $GREEN "=========================================="
print_status $GREEN "Phase 2 tests completed!"
print_status $GREEN "=========================================="
print_status $BLUE "Next steps:"
print_status $BLUE "1. Review test output for any warnings or issues"
print_status $BLUE "2. Check device compatibility results"
print_status $BLUE "3. Proceed to Phase 3: Basic Audio Recorder"
print_status $BLUE "4. Update MACOS_AUDIO_LOOPBACK_PLAN.md with results"
