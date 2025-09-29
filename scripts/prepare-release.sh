#!/usr/bin/env bash
# Release preparation script for Telelog

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

# Check if version argument is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.2.0"
    exit 1
fi

NEW_VERSION=$1

print_status "Preparing release $NEW_VERSION for Telelog"

# 1. Update versions
print_status "Updating version files..."
python scripts/version.py "$NEW_VERSION" || {
    print_error "Failed to update version files"
    exit 1
}

# 2. Run tests
print_status "Running Rust tests..."
cargo test --all-features || {
    print_error "Rust tests failed"
    exit 1
}
print_success "Rust tests passed"

# 3. Build and test Python module
print_status "Building and testing Python module..."
maturin develop || {
    print_error "Failed to build Python module"
    exit 1
}
print_success "Python module built successfully"

# Run Python tests if they exist
if [ -d "tests" ] && [ -n "$(ls -A tests)" ]; then
    print_status "Running Python tests..."
    python -m pytest tests/ || {
        print_error "Python tests failed"
        exit 1
    }
    print_success "Python tests passed"
else
    print_warning "No Python tests found, skipping..."
fi

# 4. Build release binary
print_status "Building release binary..."
cargo build --release || {
    print_error "Release build failed"
    exit 1
}
print_success "Release binary built successfully"

# 5. Build Python wheels
print_status "Building Python wheels..."
maturin build --release || {
    print_error "Failed to build Python wheels"
    exit 1
}
print_success "Python wheels built successfully"

print_success "Release preparation completed successfully!"
echo
echo "Next steps for release $NEW_VERSION:"
echo "1. Review changes: git diff"
echo "2. Commit: git add -A && git commit -m 'Release v$NEW_VERSION'"
echo "3. Tag: git tag v$NEW_VERSION"
echo "4. Push: git push && git push --tags"
echo
echo "For manual publishing:"
echo "5. Publish Rust: cargo publish"
echo "6. Publish Python: maturin publish"
echo
echo "Or just push the tag and let GitHub Actions handle publishing!"