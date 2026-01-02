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

# Cargo configuration
cargo_toml := "Cargo.toml"
cargo_lock := "Cargo.lock"

# Version extraction
version := `grep '^version' Cargo.toml | head -1 | cut -d'"' -f2`

# Documentation
readme := "README.md"
changelog := "CHANGELOG.md"

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
    source "$HOME/.cargo/env" && cargo check

# Build debug version
build:
    @echo "🔨 Building debug version..."
    source "$HOME/.cargo/env" && cargo build

# Build optimized release version
build-release:
    @echo "⚡ Building optimized release version..."
    source "$HOME/.cargo/env" && cargo build --release

# Run tests
test:
    @echo "🧪 Running tests..."
    source "$HOME/.cargo/env" && cargo test

# Run tests with output
test-verbose:
    @echo "🧪 Running tests with verbose output..."
    source "$HOME/.cargo/env" && cargo test -- --nocapture

# Format code
fmt:
    @echo "🎨 Formatting code..."
    source "$HOME/.cargo/env" && cargo fmt

# Lint code
clippy:
    @echo "🔎 Running clippy linter..."
    source "$HOME/.cargo/env" && cargo clippy

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    source "$HOME/.cargo/env" && cargo clean

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

# Run debug version
run:
    @echo "🚀 Running debug version..."
    source "$HOME/.cargo/env" && cargo run

# Run release version
run-release:
    @echo "🚀 Running release version..."
    source "$HOME/.cargo/env" && cargo run --release

# Run with test certificate (debug)
run-test-cert:
    @echo "🚀 Running with test certificate (debug)..."
    cargo run -- --file {{test_cert}}

# Run with test certificate (release)
run-test-cert-release:
    @echo "🚀 Running with test certificate (release)..."
    {{binary_path}} --file {{test_cert}}

# Run with test certificate in text mode
run-test-cert-text:
    @echo "📄 Running with test certificate in text mode..."
    {{binary_path}} --file {{test_cert}} --text

# Run interactive TUI with test certificate
run-test-cert-tui:
    @echo "🖥️  Running interactive TUI with test certificate..."
    {{binary_path}} --file {{test_cert}}

# Dependency management
# ====================

# Update dependencies
update-deps:
    @echo "📦 Updating dependencies..."
    source "$HOME/.cargo/env" && cargo update

# Check for outdated dependencies
outdated:
    @echo "📊 Checking for outdated dependencies..."
    source "$HOME/.cargo/env" && cargo outdated

# Audit dependencies for security issues
audit:
    @echo "🔒 Auditing dependencies for security issues..."
    source "$HOME/.cargo/env" && cargo audit

# Generate dependency tree
tree:
    @echo "🌳 Generating dependency tree..."
    source "$HOME/.cargo/env" && cargo tree

# Documentation recipes
# ====================

# Generate documentation
doc:
    @echo "📚 Generating documentation..."
    source "$HOME/.cargo/env" && cargo doc --open

# Generate documentation without opening browser
doc-build:
    @echo "📚 Generating documentation..."
    source "$HOME/.cargo/env" && cargo doc

# Check documentation
doc-check:
    @echo "📚 Checking documentation..."
    source "$HOME/.cargo/env" && cargo doc --no-deps

# Project information recipes
# ===========================

# Show project version
version:
    @echo "📋 Project version:"
    {{binary_path}} --version

# Show help
help:
    @echo "📖 Application help:"
    {{binary_path}} --help

# Show project information
info:
    @echo "📊 Project Information:"
    @echo "Name: {{project_name}}"
    @echo "Source directory: {{src_dir}}"
    @echo "Test directory: {{test_dir}}"
    @echo "Target directory: {{target_dir}}"
    @echo "Binary path (release): {{binary_path}}"
    @echo "Binary path (debug): {{binary_path_debug}}"
    @echo "Test certificate: {{test_cert}}"

# File system recipes
# ===================

# Show project structure
tree-project:
    @echo "📁 Project structure:"
    tree -I {{target_dir}} -a

# Show source files
list-src:
    @echo "📄 Source files:"
    find {{src_dir}} -name "*.rs" -type f

# Show test files
list-tests:
    @echo "🧪 Test files:"
    find {{test_dir}} -type f

# Count lines of code
loc:
    @echo "📊 Lines of code:"
    find {{src_dir}} -name "*.rs" -type f -exec wc -l {} + | tail -1

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

# Create and push a new tag
tag version message:
    @echo "🏷️  Creating tag {{version}} with message: {{message}}"
    git tag -a {{version}} -m "{{message}}"
    git push origin {{version}}

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

# Run benchmarks (if any)
bench:
    @echo "⚡ Running benchmarks..."
    source "$HOME/.cargo/env" && cargo bench

# Profile release build
profile:
    @echo "📊 Profiling release build..."
    source "$HOME/.cargo/env" && cargo build --release
    @echo "Binary size:"
    ls -lh {{binary_path}}

# Cross-compilation recipes
# =========================

# Build for Linux (x86_64)
build-linux:
    @echo "🐧 Building for Linux x86_64..."
    source "$HOME/.cargo/env" && cargo build --release --target x86_64-unknown-linux-gnu

# Build for macOS (x86_64)
build-macos:
    @echo "🍎 Building for macOS x86_64..."
    source "$HOME/.cargo/env" && cargo build --release --target x86_64-apple-darwin

# Build for Windows (x86_64)
build-windows:
    @echo "🪟 Building for Windows x86_64..."
    source "$HOME/.cargo/env" && cargo build --release --target x86_64-pc-windows-gnu

# Build for all platforms
build-all: build-linux build-macos build-windows
    @echo "🌍 Built for all platforms!"

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

# Create backup of important files
backup:
    @echo "💾 Creating backup..."
    mkdir -p backup
    cp {{cargo_toml}} backup/
    cp {{cargo_lock}} backup/
    cp {{readme}} backup/
    cp {{changelog}} backup/
    @echo "✅ Backup created in backup/ directory"

# Emergency cleanup (removes backup, target, and other generated files)
emergency-clean:
    @echo "🚨 Emergency cleanup..."
    rm -rf {{target_dir}}
    rm -rf backup
    rm -rf release
    @echo "✅ Emergency cleanup completed"