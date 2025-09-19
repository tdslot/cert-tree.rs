# cert-tree

A command-line utility for inspecting X.509 certificates in a tree-like structure, inspired by [cert_tree](https://github.com/jkolezyn/cert_tree).

## Features

- Parse X.509 certificates from files (PEM/DER), URLs, or command-line input
- **Certificate Chain Support**: Automatically detect and display certificate hierarchies
- **HTTPS Certificate Chain Fetching**: Extract certificate chains from any HTTPS website via TLS handshake
- Display certificate information in multiple formats:
  - Tree view (default)
  - Verbose text output
  - Interactive TUI with colors and detailed certificate inspection
  - Text mode for certificate chains (non-interactive)
- Show detailed certificate information including:
  - Subject and issuer (CN only for cleaner display)
  - Validity dates with expiration status
  - Public key and signature algorithms
  - Extensions (Key Usage, Subject Alternative Names, etc.)
  - CA status
- **Color-coded validity status**: Green (valid), Yellow (expiring soon), Red (expired)
- **Sequence numbering**: Bracketed sequence numbers [1], [2] for certificate identification
- **Enhanced TUI**: Interactive navigation with Tab-based pane switching, scrollable certificate list and details, version display, responsive layout
- Comprehensive error handling for invalid certificates
- Efficient parsing using the `x509-parser` crate

## Installation

### From Source

```bash
git clone <repository-url>
cd cert-tree.rs
cargo build --release
```

The binary will be available at `target/release/cert_tree`.

### Using Cargo

```bash
cargo install --git <repository-url> cert_tree
```

## Usage

### Basic Usage

```bash
# Inspect a certificate file
cert_tree --file certificate.pem

# Inspect certificate chain from HTTPS website (TLS handshake)
cert_tree --url https://example.com

# Inspect a certificate from a direct URL
cert_tree --url https://example.com/certificate.pem

# Inspect certificate data from command line
cert_tree --data "-----BEGIN CERTIFICATE-----...-----END CERTIFICATE-----"
```

### Output Formats

```bash
# Tree view (default)
cert_tree --file cert.pem

# Interactive TUI with colors (default)
cert_tree --file cert.pem

# Text mode for certificate chains (non-interactive)
cert_tree --file cert-chain.pem --text

# Verbose output (use with --text or TUI)
cert_tree --file cert.pem --verbose
```

### Certificate Chain Examples

```bash
# Display certificate chain from HTTPS website
cert_tree --url https://github.com --text

# Display certificate chain in text format
cert_tree --file ca_list.pem --text
```

Output:
```
━ CorpRoot                                              [1] [VALID until: 2040-05-05 18:19]
     └ ServerCA                                         [2] [VALID until: 2025-05-29 19:51]
         ├ example_cert                                  [3] [VALID until: 2025-06-15 00:07]
         └ example_2                                     [4] [VALID until: 2025-06-04 14:56]
━ RootCert                                              [5] [VALID until: 2029-04-28 14:53]
     └ example_cert3                                     [6] [EXPIRED until: 2019-06-03 13:26]
         ├ other                                         [7] [EXPIRED until: 2022-09-05 21:32]
         ├ other1                                        [8] [EXPIRED until: 2017-06-16 21:12]
         └ AnotherOne                                    [9] [VALID until: 2023-10-06 15:30]
```

```bash
# Interactive TUI for certificate chains
cert_tree --file ca_list.pem --format tui
```
*TUI mode provides color-coded display with interactive navigation*

### Options

- `-f, --file <FILE>`: Path to certificate file (PEM or DER)
- `-U, --url <URL>`: URL to fetch certificate from
- `-d, --data <DATA>`: Certificate data as string
- `-i, --interactive`: Interactive TUI mode (default: true)
- `-t, --text`: Text output mode (non-interactive, for certificate chains)
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Interactive TUI Mode

The TUI (Terminal User Interface) mode provides a beautiful, color-coded display of certificate information with advanced navigation features:

```bash
cert_tree --file cert.pem --format tui
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
- Interactive interface (Tab Toggle Panes | ↑/↓ Navigate/Scroll | PageUp/PageDown Fast Nav | 'q' Quit)
- Clean, organized layout with borders and sections
- Human-readable formatting for all certificate fields
- Column headers and right-aligned dates
- Detailed certificate inspection panel with scrollable information

## Examples

### Inspect a self-signed certificate

```bash
cert_tree --file selfsigned.pem
```

Output:
```
C=NO, ST=Some Test Certificate, L=Oslo, O=Internet Widgits Pty Ltd, OU=Home, CN=some.local.host
├── Issuer: C=NO, ST=Some Test Certificate, L=Oslo, O=Internet Widgits Pty Ltd, OU=Home, CN=some.local.host
├── Serial: 3e 82 32 19 0b 3a 2f 41 7f 5d e2 75 5d c6 03 fc 51 52 08 0
├── Valid: Thu, 18 Sep 2025 16:59:59 +0000 to Fri, 18 Sep 2026 16:59:59 +0000
├── Public Key: RSA (2056 bits)
├── Signature: OID(1.2.840.113549.1.1.11)
├── Version: 2
├── Is CA: true
└── Extensions:
    ├── 2.5.29.14 (non-critical)
    ├── 2.5.29.35 (non-critical)
    └── 2.5.29.19 (critical)
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