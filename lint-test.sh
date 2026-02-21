#!/bin/bash
# Daggerheart VTT - Lint and Test Script
# Used locally and in CI/CD

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🎲 Daggerheart VTT - Lint & Test${NC}"
echo ""

# Change to server directory
cd server

echo -e "${YELLOW}📝 Running cargo fmt check...${NC}"
cargo fmt -- --check
echo -e "${GREEN}✓ Formatting OK${NC}"
echo ""

echo -e "${YELLOW}📋 Running cargo clippy...${NC}"
cargo clippy -- -D warnings
echo -e "${GREEN}✓ Clippy OK (no warnings)${NC}"
echo ""

echo -e "${YELLOW}🧪 Running tests...${NC}"
cargo test --verbose
echo -e "${GREEN}✓ Tests passed${NC}"
echo ""

echo -e "${YELLOW}🔍 Running tests with coverage...${NC}"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Html --output-dir ../coverage
    echo -e "${GREEN}✓ Coverage report generated in coverage/index.html${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed, skipping coverage${NC}"
    echo -e "${YELLOW}   Install with: cargo install cargo-tarpaulin${NC}"
fi
echo ""

echo -e "${GREEN}✅ All checks passed!${NC}"
