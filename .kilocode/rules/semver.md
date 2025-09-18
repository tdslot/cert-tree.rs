# Semantic Versioning Rules for cert-tree.rs

## Overview

This document outlines the semantic versioning (semver) methodology for the cert-tree.rs project. Semantic versioning provides a clear, predictable way to communicate changes and maintain backward compatibility.

## Semver Basics

Version format: `MAJOR.MINOR.PATCH`

- **MAJOR** (X.y.z): Breaking changes that require user intervention
- **MINOR** (x.Y.z): New features that are backward compatible
- **PATCH** (x.y.Z): Bug fixes and minor improvements that are backward compatible

## Version Increment Rules

### MAJOR Version Increment

Increment the MAJOR version when you make incompatible API changes:

- **CLI Interface Changes**:
  - Removing or renaming command-line arguments
  - Changing argument behavior in breaking ways
  - Removing output formats

- **Output Format Changes**:
  - Changing default output format
  - Modifying JSON structure
  - Breaking changes to tree display format

- **Feature Removal**:
  - Removing supported certificate formats
  - Removing input methods (file, URL, data)
  - Removing display options

- **Architectural Changes**:
  - Major refactoring that affects user-facing behavior
  - Changes to error handling that affect scripts

**Examples for cert-tree.rs:**
- Changing `--tree` to `--format tree` (breaking CLI)
- Removing PEM format support
- Changing JSON output structure

### MINOR Version Increment

Increment the MINOR version when you add functionality in a backward compatible manner:

- **New Features**:
  - Adding support for new certificate extensions
  - Adding new output formats (e.g., XML, YAML)
  - Adding new input sources

- **CLI Enhancements**:
  - Adding new command-line options
  - Adding aliases for existing options
  - Improving help text

- **Display Improvements**:
  - Adding color schemes
  - Adding new display modes
  - Enhancing TUI features

- **Parsing Enhancements**:
  - Supporting new certificate types
  - Improving parsing accuracy
  - Adding validation features

**Examples:**
- Adding `--json-pretty` option
- Supporting new X.509 extension types
- Adding batch processing capabilities

### PATCH Version Increment

Increment the PATCH version when you make backward compatible bug fixes:

- **Bug Fixes**:
  - Fixing certificate parsing errors
  - Correcting display formatting issues
  - Fixing TUI navigation bugs

- **Performance Improvements**:
  - Optimizing parsing speed
  - Reducing memory usage
  - Improving startup time

- **Security Fixes**:
  - Fixing potential vulnerabilities
  - Updating dependencies for security
  - Improving input validation

- **Documentation Updates**:
  - Fixing help text
  - Updating README
  - Improving error messages

**Examples:**
- Fixing crash on malformed certificates
- Correcting expiry date display
- Improving error message clarity

## Pre-release Versions

Use pre-release identifiers for unstable releases:

- **Alpha** (`1.0.0-alpha.1`): Early testing, features may change
- **Beta** (`1.0.0-beta.1`): Feature complete, testing phase
- **Release Candidate** (`1.0.0-rc.1`): Ready for release, final testing

## Project-Specific Rules

### Initial Version
- Start with `1.0.0` for the first stable release
- Use `0.y.z` for initial development (if not yet released)

### Version Management Process
1. Update version in `Cargo.toml`
2. Update changelog with changes
3. Create git tag with version
4. Publish release
5. Update documentation if needed

### Implementation Rule
**Whenever changes are made and new features are successfully implemented:**
1. Run `cargo check` to ensure compilation
2. Update version in `Cargo.toml` according to semver rules
3. Update `CHANGELOG.md` with detailed release notes
4. Update memory-bank context with completed tasks
5. Commit changes with appropriate semver commit message

### Breaking Changes Policy
- **Avoid breaking changes** in MINOR versions
- **Deprecate features** before removing them in MAJOR versions
- **Provide migration guides** for breaking changes
- **Consider backward compatibility** for all changes

### Special Cases
- **Security fixes**: May warrant immediate PATCH release
- **Critical bugs**: May require immediate PATCH release
- **Performance regressions**: Treat as bugs, PATCH level
- **Documentation**: Only increment if significant user-facing changes

## Examples

### Version Progression
```
1.0.0 - Initial stable release
1.0.1 - Bug fix: parsing error on certain certificates
1.1.0 - Feature: added JSON output format
1.1.1 - Bug fix: JSON formatting issue
2.0.0 - Breaking: changed CLI argument structure
2.0.1 - Bug fix: crash on invalid URLs
```

### Commit Message Integration
When making changes, use commit messages that indicate version impact:
- `fix: correct certificate parsing bug` (PATCH)
- `feat: add XML output format` (MINOR)
- `BREAKING: remove legacy CLI options` (MAJOR)

## Maintenance

- **Review versions** before release
- **Document changes** in changelog
- **Test compatibility** with previous versions
- **Communicate changes** to users for MAJOR versions