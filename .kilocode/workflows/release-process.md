# Release Process for cert-tree.rs

This document outlines the complete process for creating a new release of cert-tree.rs, from version planning to automated GitHub release creation. The process follows semantic versioning rules and ensures consistent, high-quality releases.

## Overview

The release process is designed to work seamlessly with GitHub automation:

1. **Manual pre-release preparation** (version update, changelog, testing)
2. **Automated release creation** (git tagging triggers GitHub Actions)
3. **Fully automated distribution** (cross-platform builds and GitHub release)
4. **Post-release verification** (documentation updates and monitoring)

**Key Integration Point**: Pushing a version tag to GitHub automatically triggers the complete build and release process via GitHub Actions, eliminating manual distribution steps.

## Pre-Release Preparation

### 1. Determine Version Increment

Follow [semver rules](../rules/semver.md) to determine the appropriate version increment:

- **MAJOR** (X.y.z): Breaking changes, incompatible API changes
- **MINOR** (x.Y.z): New features, backward compatible
- **PATCH** (x.y.Z): Bug fixes, backward compatible

### 2. Update Version in Code

Update the version in `Cargo.toml`:

```toml
[package]
name = "cert-tree"
version = "0.15.0"  # Update this according to semver rules
```

### 3. Update CHANGELOG.md

Add a new section at the top of `CHANGELOG.md` with the new version:

```markdown
## [0.15.0] - 2025-01-21

### Added
- New feature description

### Changed
- Breaking change description (if applicable)

### Fixed
- Bug fix description

### Security
- Security fix description (if applicable)
```

**Guidelines:**
- Use present tense for changes ("Add feature" not "Added feature")
- Group changes by type: Added, Changed, Fixed, Security
- Reference issue/PR numbers when applicable
- Keep descriptions concise but informative

### 4. Run Quality Checks

Execute comprehensive quality checks before release:

```bash
just quality-release  # Full quality check (fmt + clippy + test + build)
just run-test-cert-release  # Test release binary with sample certificates
just version  # Verify version is correctly updated
```

### 5. Update Documentation

**Critical Step**: Ensure all documentation accurately reflects the current functionality and options.

#### Documentation Checklist
- [ ] **README.md**: Update version badge and feature descriptions
- [ ] **CLI Options**: Verify all command-line options match implementation
- [ ] **Usage Examples**: Test all examples work with current version
- [ ] **Installation Instructions**: Confirm all installation methods work
- [ ] **Feature Descriptions**: Ensure all advertised features are implemented
- [ ] **Platform Support**: Verify documented platforms match build matrix
- [ ] **Dependencies**: Update dependency list if changed
- [ ] **Screenshots/Demos**: Update if UI has changed

#### Key Documentation Files to Update
- `README.md` - Main user documentation
- `CHANGELOG.md` - Already updated in step 3
- `.kilocode/workflows/release-process.md` - This file (if process changed)
- Any API documentation or man pages

#### Documentation Verification Commands
```bash
# Test CLI help output matches documentation
just run-release --help

# Verify version information
just version

# Test documented examples
just run-test-cert-text
just run-test-cert-tui

# Check for broken links (if applicable)
# Verify installation instructions work
```

**Important**: Never release with outdated or incorrect documentation. Users rely on documentation to understand how to use the software correctly.

### 6. Update Memory Bank

**Critical Step**: Update the memory bank to maintain project continuity and context.

#### Memory Bank Files to Update
- `.kilocode/rules/memory-bank/context.md` - Current work focus and recent changes
- `.kilocode/rules/memory-bank/product.md` - If product description changed
- `.kilocode/rules/memory-bank/architecture.md` - If architecture changed
- `.kilocode/rules/memory-bank/tech.md` - If technology stack changed

#### Context.md Update Template
```markdown
## Current Work Focus
- **Project Status**: [Current status]
- **Last Major Update**: [Latest significant change] (v[new_version])
- **Current Phase**: [Development/Production/Maintenance phase]
- **Code Quality**: [Current quality metrics]
- **Documentation Status**: [Documentation state]
- **Memory Bank Status**: Updated and synchronized

## Recent Changes
- ✅ **Completed**: [Major change 1 description]
- ✅ **Completed**: [Major change 2 description]
- ✅ **Completed**: Updated version to [new_version] according to semver methodology
- ✅ **Completed**: CHANGELOG.md updated with release notes for v[new_version]
- ✅ **Completed**: Documentation updated to match current functionality
- ✅ **Completed**: Memory bank synchronized with latest project state
```

#### Memory Bank Verification
```bash
# Check memory bank files exist and are current
ls -la .kilocode/rules/memory-bank/

# Verify context.md reflects current state
cat .kilocode/rules/memory-bank/context.md
```

**Important**: The memory bank is the source of truth for project context. Never release without updating it to maintain continuity for future development sessions.

### 7. Commit Version Changes

Create a commit for the version update:

```bash
just commit "chore(release): bump version to 0.15.0"
```

## Release Creation

### 1. Create Git Tag

Create an annotated git tag for the release:

```bash
just tag v0.15.0 "Release v0.15.0"
```

This creates a tag in the format `v{major}.{minor}.{patch}`.

### 2. Push Tag to GitHub

Push the tag to trigger the automated release workflow:

```bash
git push origin v0.15.0
```

**Automation Trigger**: This single command initiates the entire automated release pipeline.

## Automated GitHub Release Creation

The release process is fully automated via GitHub Actions (`.github/workflows/release.yml`).

### Workflow Overview

The workflow triggers on tag push (`v*`) and performs:

1. **Multi-platform builds** (Linux, macOS, Windows)
2. **Binary artifact creation**
3. **Release notes extraction**
4. **GitHub release creation**

### Build Matrix

The workflow builds for three platforms:
- **Linux x86_64**: `x86_64-unknown-linux-gnu`
- **macOS x86_64**: `x86_64-apple-darwin`
- **Windows x86_64**: `x86_64-pc-windows-msvc`

### Build Steps

For each platform:
1. **Checkout code** using `actions/checkout@v4`
2. **Install Rust** with target support using `dtolnay/rust-toolchain@stable`
3. **Build release** with `cargo build --release --target {target}`
4. **Rename binary** to include platform identifier
5. **Upload artifact** for collection

### Release Creation

After all builds complete:
1. **Download artifacts** from all build jobs
2. **Extract version** from git tag
3. **Generate human-readable release notes** from CHANGELOG.md with enhanced formatting
4. **Create GitHub release** using `softprops/action-gh-release@v1`

#### Enhanced Release Notes Generation

The workflow automatically generates beautiful, user-friendly release notes that include:

- **🎯 Clear version information** with release highlights
- **📦 Download links** for all platforms with clear categorization
- **🚀 Installation instructions** for different package managers
- **📋 What's new** section with user-friendly descriptions
- **🐛 Technical changelog** appended for developers
- **📚 Documentation and community links**

The release notes are generated using a shell script that:
1. Extracts the current version from the git tag
2. Gets the release date
3. Creates a structured, emoji-enhanced release notes document
4. Appends the raw changelog for technical details

**Key Features:**
- **Platform-specific download sections** (Linux, macOS)
- **Package manager instructions** (DEB, RPM, tar.gz)
- **Quick start examples** for immediate use
- **Community links** for support and contributions
- **Professional formatting** with consistent styling

**Example generated release notes:**

```markdown
# 🎉 cert-tree v0.14.4 - Quality & Performance Release

**Released on:** 2025-09-23

## ✨ What's New

### 🚀 Performance Improvements
- **High-performance memory allocator**: Added mimalloc for significantly better memory management and application performance
- **Optimized build configuration**: Enhanced compilation settings for maximum speed

### 🔧 Code Quality Enhancements
- **Comprehensive code review**: Applied Microsoft Rust Guidelines for professional-grade code quality
- **Magic number elimination**: Replaced hardcoded values with descriptive constants
- **Enhanced documentation**: Added detailed inline documentation with examples and safety notes
- **Advanced linting**: Enabled pedantic, complexity, and performance clippy checks

### 📊 Technical Improvements
- **Warning reduction**: Decreased compiler warnings from 74 to 57 for cleaner builds
- **Memory safety**: Maintained Rust's zero-cost abstractions and memory safety guarantees
- **Cross-platform compatibility**: Verified builds work across all supported platforms

## 📦 Downloads

Choose the right package for your system:

### 🐧 Linux
- **Binary**: `cert_tree-linux-x86_64.tar.gz` (Universal Linux binary)
- **RPM Packages**:
  - CentOS/RHEL: `cert_tree-centos-x86_64.rpm`
  - Rocky Linux: `cert_tree-rocky-x86_64.rpm`
  - Alma Linux: `cert_tree-alma-x86_64.rpm`
- **DEB Packages**:
  - Debian: `cert_tree-debian-x86_64.deb`
  - Ubuntu: `cert_tree-ubuntu-x86_64.deb`

### 🍎 macOS
- **Intel**: `cert_tree-macos-x86_64.tar.gz`
- **Apple Silicon**: `cert_tree-macos-aarch64.tar.gz`

## 🚀 Quick Start

### From Binary (All Platforms)
```bash
# Download and extract
tar -xzf cert_tree-*-x86_64.tar.gz
cd cert_tree-*

# Make executable and run
chmod +x cert_tree
./cert_tree --help
```

### From Package Managers

**Ubuntu/Debian:**
```bash
sudo dpkg -i cert_tree-ubuntu-*.deb
cert-tree --help
```

**CentOS/RHEL/Rocky/Alma:**
```bash
sudo rpm -i cert_tree-centos-*.rpm
cert-tree --help
```

### Basic Usage Examples

```bash
# Inspect a certificate file
cert-tree --file certificate.pem

# Check website certificates
cert-tree --url https://github.com

# Interactive mode
cert-tree --file certificate.pem --interactive
```

## 🔗 Links

- **📖 Documentation**: [README.md](https://github.com/tdslot/cert-tree.rs/blob/main/README.md)
- **🐛 Report Issues**: [GitHub Issues](https://github.com/tdslot/cert-tree.rs/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/tdslot/cert-tree.rs/discussions)

## 🙏 Acknowledgments

Built with ❤️ using Rust and modern development practices.

---

*This release maintains full backward compatibility and includes comprehensive testing across all platforms.*
```

### Release Artifacts

The release includes:
- `cert-tree-linux-x86_64` (Linux binary)
- `cert-tree-macos-x86_64` (macOS binary)
- `cert-tree-windows-x86_64.exe` (Windows executable)
- Release notes extracted from CHANGELOG.md

## Post-Release Tasks

### 1. Verify Release

After the workflow completes:
- Check GitHub Releases for the new release
- Verify all artifacts are present and downloadable
- Test downloaded binaries on target platforms

### 2. Update Documentation

Update external documentation if needed:
- Update README.md version references (if any)
- Update website or documentation sites
- Update package manager metadata

### 3. Announce Release

- Create GitHub release announcement
- Update project status badges
- Notify community channels (if applicable)

### 4. Monitor Issues

Monitor for post-release issues:
- Check GitHub Issues for bug reports
- Monitor CI/CD pipeline status
- Be prepared for hotfix releases if critical issues arise

## Rollback Process

If a release needs to be rolled back:

1. **Delete the git tag**:
   ```bash
   git tag -d v0.15.0
   git push origin :refs/tags/v0.15.0
   ```

2. **Delete GitHub release** via GitHub web interface

3. **Revert version changes** and create patch release

## Automation and Tools

### Justfile Recipes

Key justfile recipes for release process:
- `just quality-release` - Full pre-release quality check
- `just build-release` - Build optimized release binary
- `just tag <version> <message>` - Create annotated git tag
- `just version` - Display current version

### GitHub Actions Integration

The automated workflow handles:
- Cross-platform compilation
- Binary distribution
- Release note generation
- GitHub release creation

**No manual intervention required** after pushing the tag - the entire distribution process is automated.

## Documentation Maintenance

### Documentation Accuracy Principles

**Documentation must always reflect the actual functionality** to prevent user confusion and support issues.

#### Key Documentation Areas
- **CLI Interface**: Command-line options, flags, and behavior
- **Features**: What the software actually does vs. what documentation claims
- **Installation**: All supported installation methods and platforms
- **Examples**: Working code examples that users can copy-paste
- **Version Information**: Current version numbers and compatibility
- **Dependencies**: Required and optional dependencies

#### Documentation Update Triggers
- [ ] New features implemented
- [ ] CLI options changed or added
- [ ] Installation methods modified
- [ ] Dependencies updated
- [ ] Platform support changed
- [ ] Bug fixes that affect user behavior
- [ ] Security changes
- [ ] Breaking changes

#### Documentation Quality Checks
```bash
# Verify CLI matches documentation
just run-release --help | grep -E "(file|url|interactive|text)"

# Test documented examples
just run-test-cert-text  # Should work as documented
just run-test-cert-tui   # Should work as documented

# Check version consistency
just version  # Should match README badge
```

### Documentation as Code

Treat documentation as code with the same quality standards:
- **Version control**: Documentation changes committed with code
- **Review process**: Documentation reviewed during code review
- **Testing**: Examples tested as part of CI/CD
- **Accuracy**: Documentation verified before releases

## Quality Assurance

### Pre-Release Checklist

- [ ] Version updated in `Cargo.toml`
- [ ] CHANGELOG.md updated with release notes
- [ ] **Documentation updated and verified**:
  - [ ] README.md version badge updated
  - [ ] CLI options match implementation (`just run-release --help`)
  - [ ] Usage examples tested and working
  - [ ] Installation instructions verified
  - [ ] Feature descriptions accurate
  - [ ] Platform support matches build matrix
- [ ] All tests pass (`just test`)
- [ ] Code formatting correct (`just fmt`)
- [ ] No clippy warnings (`just clippy`)
- [ ] Release binary tested (`just run-test-cert-release`)
- [ ] **Memory bank updated**:
  - [ ] context.md updated with current work focus and recent changes
  - [ ] All relevant memory bank files synchronized
  - [ ] Project state accurately reflected
- [ ] Version changes committed

### Post-Release Checklist

- [ ] GitHub Actions workflow completed successfully
- [ ] Release created in GitHub Releases
- [ ] All artifacts present and downloadable
- [ ] Release notes display correctly
- [ ] **Documentation verification**:
  - [ ] Version badges updated in README.md
  - [ ] GitHub release links work correctly
  - [ ] Installation instructions are current
  - [ ] No broken links in documentation
  - [ ] Community links (issues, discussions) are accessible

## Troubleshooting

### Common Issues

**Workflow fails to trigger:**
- Ensure tag format matches `v*` pattern
- Verify tag is pushed to correct repository

**Build failures:**
- Check Rust version compatibility
- Verify dependencies are available
- Review build logs for platform-specific issues

**Release creation fails:**
- Verify CHANGELOG.md format matches expected pattern
- Check GITHUB_TOKEN permissions
- Ensure release notes extraction works correctly

### Emergency Procedures

For critical issues requiring immediate action:
1. Assess severity and impact
2. Consider rollback vs. hotfix
3. Communicate with stakeholders
4. Execute appropriate remediation steps

This process ensures consistent, reliable releases while maintaining code quality and proper version management through seamless GitHub automation.