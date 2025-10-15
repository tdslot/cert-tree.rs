# cert-tree

[![Version](https://img.shields.io/badge/version-0.14.8-blue.svg)](https://github.com/tdslot/cert-tree.rs/releases)

A command-line utility for inspecting X.509 certificates in a tree-like structure, inspired by [cert_tree](https://github.com/jkolezyn/cert_tree).

## Features

- Parse X.509 certificates from files (PEM/DER), URLs, or command-line input
- **Certificate Chain Support**: Automatically detect and display certificate hierarchies
- **HTTPS Certificate Chain Fetching**: Extract certificate chains from any HTTPS website via TLS handshake
- Display certificate information in multiple formats:
  - Text mode (default - non-interactive)
  - Interactive TUI with colors and detailed certificate inspection
  - Tree view for certificate chains
  - Verbose text output for single certificates
- Show detailed certificate information including:
  - Subject and issuer (CN only for cleaner display)
  - Validity dates with expiration status
  - Public key and signature algorithms
  - **Human-readable extensions** (Key Usage, Subject Alternative Names, Authority Information Access, etc.)
  - CA status
- **Color-coded validity status**: Green (valid), Yellow (expiring soon), Red (expired)
- **Sequence numbering**: Bracketed sequence numbers [1], [2] for certificate identification
- **Enhanced TUI**: Interactive navigation with Tab-based pane switching, scrollable certificate list and details, automatic text wrapping for long content, version display, responsive layout, ISO 8601 date-time format
- **CRL Support**: Certificate revocation checking infrastructure with revocation status display
- Comprehensive error handling for invalid certificates
- Efficient parsing using the `x509-parser` crate

## Installation

### From Source

```bash
git clone https://github.com/tdslot/cert-tree.rs
cd cert-tree.rs
cargo build --release
```

The binary will be available at `target/release/cert-tree`.

### Using Cargo

```bash
cargo install --git https://github.com/tdslot/cert-tree.rs cert-tree
```

### From GitHub Releases

Download pre-built binaries for your platform from the [GitHub Releases](https://github.com/tdslot/cert-tree.rs/releases) page.

Releases are automatically built and published via GitHub Actions when version tags are pushed.

### Using mise (Version Manager)

[mise](https://mise.jdx.dev/) is a modern version manager that can install tools from various backends including GitHub releases.

```bash
# Install mise first (if not already installed)
curl https://mise.run | sh

# Restart your shell or source the mise activation
# Then install cert-tree globally
mise use -g ubi:tdslot/cert-tree.rs[exe=cert-tree]
cert-tree --help
```

The `ubi` backend automatically downloads and installs the latest release from GitHub.

## Justfile - Development Workflow

This project includes a comprehensive `Justfile` that provides a complete set of recipes for development workflows, testing, building, and project management. The Justfile ensures consistent execution of all critical tasks while preserving important paths, variables, and configuration details.

### What is Justfile?

[Just](https://github.com/casey/just) is a command runner that allows you to save and run project-specific commands. It's an alternative to Makefiles and shell scripts for task automation, providing:

- **Simple syntax** with clear, human-readable recipes
- **Cross-platform compatibility** (Linux, macOS, Windows)
- **Variable support** for reusable configuration
- **Dependency management** between tasks
- **Shell integration** with automatic environment sourcing

### Installation

```bash
# macOS
brew install just

# Linux (Ubuntu/Debian)
sudo apt install just

# Or from source
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | sh -s --to ~/.local/bin
```

### Quick Start

```bash
# Show all available recipes
just --list

# Quick development check (format + lint + build)
just dev-check

# Full quality assurance (format + lint + test + build)
just quality

# Run with test certificate
just run-test-cert-text

# Get project information
just info
```

### Development Workflow Recipes

#### Quality Assurance
```bash
just quality          # Full quality check (format + lint + test + build)
just quality-release  # Quality check for release build
just dev-check        # Quick development validation (format + lint + build)
```

#### Building
```bash
just check            # Check code compilation
just build            # Build debug version
just build-release    # Build optimized release version
```

#### Testing
```bash
just test             # Run all tests
just test-verbose     # Run tests with verbose output
```

#### Code Quality
```bash
just fmt              # Format code
just clippy           # Run linter
```

#### Application Execution
```bash
just run                      # Run debug version
just run-release             # Run release version
just run-test-cert           # Run with test certificate (debug)
just run-test-cert-release   # Run with test certificate (release)
just run-test-cert-text      # Text mode with test certificate
just run-test-cert-tui       # Interactive TUI with test certificate
```

### Advanced Recipes

#### Dependency Management
```bash
just update-deps      # Update Cargo dependencies
just outdated         # Check for outdated dependencies
just audit            # Security audit of dependencies
just tree             # Generate dependency tree
```

#### Documentation
```bash
just doc              # Generate and open documentation
just doc-build        # Generate documentation
just doc-check        # Check documentation
```

#### Project Information
```bash
just info             # Display project configuration
just version          # Show application version
just help             # Show application help
```

#### File System
```bash
just list-src         # List source files
just list-tests       # List test files
just loc              # Count lines of code
just tree-project     # Show project structure
```

#### Git Workflow
```bash
just status           # Show git status
just log              # Show recent commits
just commit "message" # Create commit with message
just push             # Push to remote
just pull             # Pull from remote
just tag v1.0.0 "Release notes"  # Create and push tag
```

#### Release Management
```bash
just prepare-release  # Full release preparation
just release-archive  # Create release archive
just release          # Complete release workflow
```

#### Cross-Platform Building
```bash
just build-linux      # Build for Linux x86_64
just build-macos      # Build for macOS x86_64
just build-windows    # Build for Windows x86_64
just build-all        # Build for all platforms
```

#### Development Environment
```bash
just setup            # Setup development environment
just install-tools    # Install development tools
just backup           # Backup important files
just emergency-clean  # Complete cleanup
```

### Justfile Configuration

The Justfile includes important project variables:

```justfile
project_name := "cert-tree"
src_dir := "src"
test_dir := "test"
target_dir := "target"
binary_path := target_dir + "/release/" + project_name
test_cert := test_dir + "/cacert.pem"
```

### Recipe Categories

| Category | Purpose | Key Recipes |
|----------|---------|-------------|
| **Quality** | Code quality assurance | `quality`, `dev-check`, `fmt`, `clippy` |
| **Build** | Compilation and building | `check`, `build`, `build-release` |
| **Test** | Testing and validation | `test`, `test-verbose` |
| **Run** | Application execution | `run`, `run-test-cert*` variants |
| **Deps** | Dependency management | `update-deps`, `outdated`, `audit` |
| **Docs** | Documentation | `doc`, `doc-build`, `doc-check` |
| **Info** | Project information | `info`, `version`, `help` |
| **Git** | Version control | `status`, `commit`, `push`, `tag` |
| **Release** | Release management | `prepare-release`, `release` |
| **Cross** | Cross-platform | `build-*` variants |
| **Util** | Utilities | `backup`, `clean`, `setup` |

### Best Practices

1. **Use `just --list`** to see all available recipes
2. **Run `just quality`** before committing changes
3. **Use `just dev-check`** for quick validation during development
4. **Run `just prepare-release`** before creating releases
5. **Use `just info`** to verify project configuration

### Integration with Development Workflow

The Justfile integrates seamlessly with typical Rust development workflows:

```bash
# Daily development cycle
just dev-check        # Quick validation
just test            # Run tests
just build           # Build project

# Before commit
just quality         # Full quality check
just commit "feat: add new feature"

# Release preparation
just prepare-release # Full validation
just tag v1.0.0 "Stable release"
```

The Justfile ensures that all team members use consistent commands and configurations, reducing errors and improving development efficiency.

## Usage

### Basic Usage

```bash
# Inspect a certificate file
cert-tree --file certificate.pem

# Inspect certificate chain from HTTPS website (TLS handshake)
cert-tree --url https://example.com

# Inspect a certificate from a direct URL
cert-tree --url https://example.com/certificate.pem
```

### Output Formats

```bash
# Text mode (default - non-interactive)
cert-tree --file cert.pem

# Interactive TUI with colors
cert-tree --file cert.pem --interactive

# Text mode for certificate chains (explicit)
cert-tree --file cert-chain.pem --text
```

### Certificate Chain Examples

```bash
# Display certificate chain from HTTPS website
cert-tree --url https://github.com --text

# Display certificate chain in text format
cert-tree --file ca_list.pem --text
```

Output:
```
━ Entrust Root Certification Authority                                                     [1] [VALID until: 2026-11-27 20:53:42]
━ QuoVadis Root CA 2                                                                       [2] [VALID until: 2031-11-24 18:23:33]
━ QuoVadis Root CA 3                                                                       [3] [VALID until: 2031-11-24 19:06:44]
━ DigiCert Assured ID Root CA                                                              [4] [VALID until: 2031-11-10 00:00:00]
━ DigiCert Global Root CA                                                                  [5] [VALID until: 2031-11-10 00:00:00]
━ DigiCert High Assurance EV Root CA                                                       [6] [VALID until: 2031-11-10 00:00:00]
━ SwissSign Gold CA - G2                                                                   [7] [VALID until: 2036-10-25 08:30:35]
━ SecureTrust CA                                                                           [8] [VALID until: 2029-12-31 19:40:55]
━ Secure Global CA                                                                         [9] [VALID until: 2029-12-31 19:52:06]
━ COMODO Certification Authority                                                           [10] [VALID until: 2029-12-31 23:59:59]
```

```bash
# Interactive TUI for certificate chains
cert-tree --file ca_list.pem --interactive
```
*TUI mode provides color-coded display with interactive navigation*

### Options

- `-f, --file <FILE>`: Certificate file path (PEM or DER)
- `-U, --url <URL>`: Certificate URL
- `-i, --interactive`: Interactive TUI mode (default: false)
- `-t, --text`: Force text output mode (non-interactive, default: true)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Interactive TUI Mode

The TUI (Terminal User Interface) mode provides a beautiful, color-coded display of certificate information with advanced navigation features:

```bash
cert-tree --file cert.pem --interactive
```

### Color Coding
- 🟢 **Green**: Valid certificates (more than 30 days until expiry)
- 🟡 **Yellow**: Certificates expiring soon (within 30 days)
- 🔴 **Red**: Expired certificates
- 🔵 **Blue**: Field labels
- 🟢 **Green**: Algorithm information
- 🟡 **Yellow**: CA certificates
- 🟣 **Magenta**: Key usage information
- 🟦 **Cyan**: Subject alternative names

### TUI Features
- Real-time certificate validity status with color coding
- Interactive navigation with Tab-based pane switching between certificate list and details
- Context-aware arrow keys: navigate list when details inactive, scroll details when active
- Page Up/Page Down support for fast navigation through certificate lists
- Visual feedback with color-coded borders indicating active pane
- Version number displayed in title bar
- Dynamic column sizing that adapts to terminal width
- Responsive layout for different terminal sizes
- ISO 8601 date-time format for precise validity periods (e.g., 2024-11-10T12:00:00Z)
- Interactive interface (Tab Toggle Panes | ↑/↓ Navigate/Scroll | PageUp/PageDown Fast Nav | 'q' Quit)
- Clean, organized layout with borders and sections
- Human-readable formatting for all certificate fields
- Column headers and right-aligned dates
- Detailed certificate inspection panel with scrollable information including human-readable extension names
- **Automatic text wrapping** for long content (signature algorithm explanations, etc.) to prevent overflow
- **Full content visibility** across different screen sizes and terminal widths

## Examples

### Inspect a single certificate (text mode)

```bash
cert-tree --file certificate.pem --text
```

Output:
```
━ example.com                      [1] [VALID until: 2024-11-10 12:00:00]
```


## Dependencies

- `clap`: Command-line argument parsing
- `x509-parser`: X.509 certificate parsing
- `reqwest`: HTTP client for URL fetching
- `rustls`: TLS library for certificate chain extraction
- `webpki-roots`: Trusted root certificates for TLS validation
- `anyhow` & `thiserror`: Error handling
- `ratatui`: Terminal user interface framework
- `crossterm`: Cross-platform terminal manipulation
- `chrono`: Date/time processing for validity calculations

## Error Handling

The utility provides clear error messages for common issues:

- Invalid certificate format
- File not found
- Network errors when fetching from URLs
- Parsing errors for malformed certificates

## Performance

- Efficient parsing using zero-copy operations where possible
- Minimal memory footprint
- Fast execution for typical certificate sizes

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Inspired by the original [cert_tree](https://github.com/jkolezyn/cert_tree) project.