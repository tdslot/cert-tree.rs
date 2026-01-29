#!/usr/bin/env just --justfile
# Justfile for cert-tree.rs - X.509 Certificate Inspection Utility
# This file contains recipes for common development tasks

# Project configuration
project_name := "cert-tree"
src_dir := "src"
test_dir := "test"
target_dir := "target"
release_dir := target_dir + "/release"
debug_dir := target_dir + "/debug"

# Binary names
binary_name := "cert-tree"
binary_path := release_dir + "/" + binary_name
binary_path_debug := debug_dir + "/" + binary_name

# Test files
test_cert := test_dir + "/cacert.pem"
test_cert_single := test_dir + "/single_cert.pem"

# Cargo configuration
cargo_toml := "Cargo.toml"
cargo_lock := "Cargo.lock"

# Version extraction
version := `grep '^version' Cargo.toml | head -1 | cut -d'"' -f2`

# Documentation
readme := "README.md"
changelog := "CHANGELOG.md"
agents_md := "AGENTS.md"

# Default recipe (shows available commands)
default:
    @echo "🚀 cert-tree.rs - Development Recipes"
    @echo ""
    @echo "📋 Available recipes:"
    @just --list --unsorted

# Development workflow recipes
# ==============================

# Check code for compilation errors
check:
    @echo "🔍 Checking code compilation..."
    cargo check

# Build debug version
build:
    @echo "🔨 Building debug version..."
    cargo build

# Build optimized release version
build-release:
    @echo "⚡ Building optimized release version..."
    cargo build --release

# Run tests
test:
    @echo "🧪 Running tests..."
    cargo test

# Run tests with output (nocapture shows println! output)
test-verbose:
    @echo "🧪 Running tests with verbose output..."
    cargo test -- --nocapture

# Format code with rustfmt
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt

# Lint code with clippy (treat warnings as errors)
clippy:
    @echo "🔎 Running clippy linter..."
    cargo clippy -- -D warnings

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean

# Full clean (removes target directory completely)
clean-all:
    @echo "🧹 Removing entire target directory..."
    rm -rf {{target_dir}}

# Development quality checks
# ==========================

# Run all quality checks (format, lint, test, build)
quality: fmt clippy test build
    @echo "✅ All quality checks passed!"

# Run all quality checks for release
quality-release: fmt clippy test build-release
    @echo "✅ All quality checks passed for release!"

# Quick development check (format, lint, build)
dev-check: fmt clippy build
    @echo "✅ Development checks completed!"

# Application execution recipes
# =============================

# Run debug version (shows help)
run:
    @echo "🚀 Running debug version..."
    cargo run

# Run release version (shows help)
run-release:
    @echo "🚀 Running release version..."
    cargo run --release

# Run with test certificate chain (text mode)
run-test-cert:
    @echo "🚀 Running with test certificate chain (text mode)..."
    cargo run -- --file {{test_cert}} --text

# Run with test certificate chain (interactive TUI)
run-test-cert-tui:
    @echo "🖥️  Running with test certificate chain (interactive TUI)..."
    cargo run -- --file {{test_cert}} --interactive

# Run with single certificate (text mode)
run-single-cert:
    @echo "📄 Running with single certificate (text mode)..."
    cargo run -- --file {{test_cert_single}} --text

# Run with single certificate (interactive TUI)
run-single-cert-tui:
    @echo "🖥️  Running with single certificate (interactive TUI)..."
    cargo run -- --file {{test_cert_single}} --interactive

# Run with URL certificate fetching
run-url url="https://google.com":
    @echo "🌐 Fetching certificates from {{url}}..."
    cargo run -- --url {{url}} --text

# Dependency management
# ====================

# Update dependencies to latest compatible versions
update-deps:
    @echo "📦 Updating dependencies..."
    cargo update

# Check for outdated dependencies (requires cargo-outdated)
outdated:
    @echo "📊 Checking for outdated dependencies..."
    @echo "Note: Install with 'cargo install cargo-outdated'"
    cargo outdated || echo "cargo-outdated not installed"

# Audit dependencies for security issues (requires cargo-audit)
audit:
    @echo "🔒 Auditing dependencies for security issues..."
    @echo "Note: Install with 'cargo install cargo-audit'"
    cargo audit || echo "cargo-audit not installed"

# Generate dependency tree
dep-tree:
    @echo "🌳 Generating dependency tree..."
    cargo tree

# Documentation recipes
# ====================

# Generate and open documentation
doc:
    @echo "📚 Generating documentation..."
    cargo doc --open

# Generate documentation without opening browser
doc-build:
    @echo "📚 Building documentation..."
    cargo doc --no-deps

# Check documentation for warnings
doc-check:
    @echo "📚 Checking documentation..."
    cargo doc --no-deps --document-private-items

# Project information recipes
# ===========================

# Show project version from Cargo.toml
show-version:
    @echo "📋 Project version: {{version}}"

# Show application help (requires release build)
show-help: build-release
    @echo "📖 Application help:"
    {{binary_path}} --help

# Show application version (requires release build)
show-app-version: build-release
    @echo "📋 Application version:"
    {{binary_path}} --version

# Show project information
info:
    @echo "📊 Project Information:"
    @echo "  Name: {{project_name}}"
    @echo "  Version: {{version}}"
    @echo "  Source directory: {{src_dir}}"
    @echo "  Test directory: {{test_dir}}"
    @echo "  Target directory: {{target_dir}}"
    @echo "  Binary path (release): {{binary_path}}"
    @echo "  Binary path (debug): {{binary_path_debug}}"
    @echo "  Test certificate chain: {{test_cert}}"
    @echo "  Test single certificate: {{test_cert_single}}"

# File system recipes
# ===================

# Show project structure (requires tree command)
show-tree:
    @echo "📁 Project structure:"
    @tree -I 'target' -a || echo "tree command not installed"

# Show all source files
list-src:
    @echo "📄 Source files:"
    @find {{src_dir}} -name "*.rs" -type f | sort

# Show test files
list-tests:
    @echo "🧪 Test certificate files:"
    @find {{test_dir}} -type f | sort

# Count lines of code in source files
loc:
    @echo "📊 Lines of code statistics:"
    @echo ""
    @echo "Source files:"
    @find {{src_dir}} -name "*.rs" -type f -exec wc -l {} + | sort -n
    @echo ""
    @echo "Total:"
    @find {{src_dir}} -name "*.rs" -type f -exec cat {} + | wc -l

# Detailed code statistics
stats:
    @echo "📈 Code Statistics:"
    @echo ""
    @echo "Rust source files:"
    @find {{src_dir}} -name "*.rs" -type f | wc -l
    @echo ""
    @echo "Total lines (including comments and blanks):"
    @find {{src_dir}} -name "*.rs" -type f -exec cat {} + | wc -l
    @echo ""
    @echo "Non-empty lines:"
    @find {{src_dir}} -name "*.rs" -type f -exec cat {} + | grep -v '^[[:space:]]*$' | wc -l

# Git recipes
# ===========

# Show git status
status:
    @echo "📊 Git status:"
    git status --short

# Show recent commits
log:
    @echo "📝 Recent commits:"
    git log --oneline -10

# Create a new commit with conventional format
commit message:
    @echo "📝 Creating commit with message: {{message}}"
    git add .
    git commit -m "{{message}}"

# Push to remote
push:
    @echo "⬆️  Pushing to remote..."
    git push

# Pull from remote
pull:
    @echo "⬇️  Pulling from remote..."
    git pull

# Create and push a new version tag (use for manual releases)
tag tag_name message:
    @echo "🏷️  Creating tag {{tag_name}} with message: {{message}}"
    git tag -a {{tag_name}} -m "{{message}}"
    git push origin {{tag_name}}

# Release workflow recipes
# ========================

# Prepare for release (full quality check + build)
prepare-release: quality-release
    @echo "🎉 Release preparation completed!"
    @echo "Binary available at: {{binary_path}}"

# Create release archive
release-archive: build-release
    @echo "📦 Creating release archive..."
    mkdir -p release
    cp {{binary_path}} release/
    cp {{readme}} release/
    cp {{changelog}} release/
    cd release && tar -czf {{project_name}}-$(date +%Y%m%d).tar.gz *
    @echo "📦 Release archive created: release/{{project_name}}-$(date +%Y%m%d).tar.gz"

# Full release workflow
release: prepare-release release-archive
    @echo "🎊 Full release workflow completed!"

# GitHub release workflow (creates signed tag and lets GitHub Actions handle release)
release-github:
    @echo "📝 Staging and committing release workflow updates..."
    git add .github/workflows/release.yml Justfile Cargo.toml CHANGELOG.md
    git commit -m "chore: update release workflow and Justfile for v{{version}}" || true
    git push
    @echo "🏷️ Cleaning up any existing release tags to avoid conflicts..."
    git tag -d v{{version}} || true
    git push origin --delete v{{version}} || true
    @echo "🔐 Creating new GPG-signed annotated version tag v{{version}}..."
    git tag -s v{{version}} -m "Release v{{version}}"
    git push origin v{{version}}
    @echo "✅ Signed version tag v{{version}} successfully created and pushed to remote."
    @echo "🚀 Release workflow initiated. GitHub Actions will now build and publish the release."

# Development environment recipes
# ===============================

# Setup development environment
setup:
    @echo "🔧 Setting up development environment..."
    rustc --version
    cargo --version
    @echo "✅ Development environment ready!"

# Install development tools
install-tools:
    @echo "🔧 Installing development tools..."
    cargo install cargo-audit
    cargo install cargo-outdated
    @echo "✅ Development tools installed!"

# Watch mode for continuous development
watch:
    @echo "👀 Starting watch mode (requires cargo-watch)..."
    cargo watch -x check

# Benchmarking recipes
# ====================

# Run benchmarks (requires benchmark setup)
bench:
    @echo "⚡ Running benchmarks..."
    @cargo bench || echo "No benchmarks configured"

# Show release build profile information
profile: build-release
    @echo "📊 Release build profile:"
    @echo ""
    @echo "Binary size:"
    @ls -lh {{binary_path}}
    @echo ""
    @echo "Binary type:"
    @file {{binary_path}}
    @echo ""
    @echo "Dependencies:"
    @cargo tree --edges normal --depth 1

# Cross-compilation recipes
# =========================

# Build for Linux (x86_64) - requires target installed
build-linux:
    @echo "🐧 Building for Linux x86_64..."
    @rustup target add x86_64-unknown-linux-gnu || true
    cargo build --release --target x86_64-unknown-linux-gnu

# Build for macOS (x86_64) - requires target installed
build-macos:
    @echo "🍎 Building for macOS x86_64..."
    @rustup target add x86_64-apple-darwin || true
    cargo build --release --target x86_64-apple-darwin

# Build for macOS (ARM64) - requires target installed
build-macos-arm:
    @echo "🍎 Building for macOS ARM64..."
    @rustup target add aarch64-apple-darwin || true
    cargo build --release --target aarch64-apple-darwin

# Build for Windows (x86_64) - requires target and cross-compiler
build-windows:
    @echo "🪟 Building for Windows x86_64..."
    @rustup target add x86_64-pc-windows-gnu || true
    cargo build --release --target x86_64-pc-windows-gnu

# List available build targets
list-targets:
    @echo "🎯 Available build targets:"
    @rustup target list | grep installed

# Utility recipes
# ===============

# Show environment variables
env:
    @echo "🔧 Environment variables:"
    @echo "RUST_BACKTRACE=1"
    @echo "CARGO_HOME={{env_var_or_default('CARGO_HOME', '~/.cargo')}}"
    @echo "RUSTUP_HOME={{env_var_or_default('RUSTUP_HOME', '~/.rustup')}}"

# Show system information
sysinfo:
    @echo "💻 System information:"
    uname -a
    @echo ""
    rustc --version
    cargo --version

# Create backup of important project files
backup:
    @echo "💾 Creating backup..."
    @mkdir -p backup
    @cp {{cargo_toml}} backup/
    @cp {{cargo_lock}} backup/
    @cp {{readme}} backup/
    @cp {{changelog}} backup/
    @cp {{agents_md}} backup/
    @cp Justfile backup/
    @echo "✅ Backup created in backup/ directory with timestamp: $(date +%Y-%m-%d_%H-%M-%S)"

# Emergency cleanup (removes all generated files and artifacts)
emergency-clean:
    @echo "🚨 Emergency cleanup - removing all generated files..."
    @rm -rf {{target_dir}}
    @rm -rf backup
    @rm -rf release
    @cargo clean
    @echo "✅ Emergency cleanup completed"

# Comprehensive project check (runs all quality checks and tests)
full-check: fmt clippy test doc-check build-release
    @echo "✅ Full project check completed successfully!"
    @echo ""
    @echo "Summary:"
    @echo "  ✓ Code formatted"
    @echo "  ✓ Clippy lints passed"
    @echo "  ✓ Tests passed"
    @echo "  ✓ Documentation checked"
    @echo "  ✓ Release build successful"

# Quick iteration workflow (format, check, test)
quick: fmt check test
    @echo "✅ Quick check completed!"