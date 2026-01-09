#!/bin/bash
# Run test suite

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_step() {
    echo -e "${GREEN}==>${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}WARNING:${NC} $1"
}

# Parse arguments
TEST_TYPE=${1:-all}

# Check if test cluster is running
if ! kubectl cluster-info &> /dev/null; then
    print_warning "Test cluster not found. Starting..."
    ./scripts/setup-test-cluster.sh
fi

case $TEST_TYPE in
    unit)
        print_step "Running unit tests..."
        cargo test --lib
        ;;

    integration)
        print_step "Running integration tests..."
        cargo test --test '*'
        ;;

    chaos)
        print_step "Running chaos tests..."
        if ! kubectl get namespace chaos-testing &> /dev/null; then
            print_warning "Chaos Mesh not installed. Installing..."
            ./scripts/install-chaos-mesh.sh
        fi
        cargo test --test chaos -- --chaos
        ;;

    bench)
        print_step "Running performance benchmarks..."
        cargo bench
        ;;

    all)
        print_step "Running full test suite..."

        print_step "1. Running unit tests..."
        cargo test --lib

        print_step "2. Running integration tests..."
        cargo test --test '*'

        print_step "3. Running performance benchmarks..."
        cargo bench

        echo -e "${GREEN}All tests passed!${NC}"
        ;;

    *)
        echo "Usage: $0 [unit|integration|chaos|bench|all]"
        exit 1
        ;;
esac
