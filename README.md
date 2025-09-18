# cert-tree

A command-line utility for inspecting X.509 certificates in a tree-like structure, inspired by [cert_tree](https://github.com/jkolezyn/cert_tree).

## Features

- Parse X.509 certificates from files (PEM/DER), URLs, or command-line input
- **Certificate Chain Support**: Automatically detect and display certificate hierarchies
- Display certificate information in multiple formats:
  - Tree view (default)
  - Verbose text output
  - JSON export
  - Interactive TUI with colors
- Show detailed certificate information including:
  - Subject and issuer
  - Validity dates with expiration status
  - Public key and signature algorithms
  - Extensions (Key Usage, Subject Alternative Names, etc.)
  - CA status
- **Color-coded validity status**: Green (valid), Yellow (expiring soon), Red (expired)
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

# Inspect a certificate from a URL
cert_tree --url https://example.com/certificate.pem

# Inspect certificate data from command line
cert_tree --data "-----BEGIN CERTIFICATE-----...-----END CERTIFICATE-----"
```

### Output Formats

```bash
# Tree view (default)
cert_tree --file cert.pem

# Interactive TUI with colors
cert_tree --file cert.pem --format tui

# Text mode for certificate chains (non-interactive)
cert_tree --file cert-chain.pem --text

# Verbose output
cert_tree --file cert.pem --format verbose

# JSON output
cert_tree --file cert.pem --format json

# Save JSON to file
cert_tree --file cert.pem --format json --output cert.json
```

### Certificate Chain Examples

```bash
# Display certificate chain in text format
cert_tree --file ca_list.pem --text
```

Output:
```
━ CorpRoot            [valid until: 2040-05-05 18:19:20]
    ┣━ ServerCA       [valid until: 2025-05-29 19:51:12]
    ┣━ example_cert   [valid until: 2025-06-15 00:07:55]
    ┗━ example_2      [valid until: 2025-06-04 14:56:07]
━ RootCert            [valid until: 2029-04-28 14:53:22]
    ┣━ example_cert3  [EXPIRED on: 2019-06-03 13:26:21]
    ┣━ other          [valid until: 2022-09-05 21:32:11]
    ┣━ other1         [EXPIRED on: 2017-06-16 21:12:18]
    ┗━ AnotherOne     [valid until: 2023-10-06 15:30:47]
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
- `--format <FORMAT>`: Output format (tree, json, verbose, tui) [default: tree]
- `-o, --output <FILE>`: Output file for JSON format
- `-t, --text`: Text output mode (non-interactive, for certificate chains)
- `-v, --verbose`: Enable verbose output
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Interactive TUI Mode

The TUI (Terminal User Interface) mode provides a beautiful, color-coded display of certificate information:

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
- Real-time certificate validity status
- Color-coded visual indicators
- Interactive interface (press 'q' to quit)
- Clean, organized layout with borders and sections
- Human-readable formatting for all certificate fields

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

### JSON Export

```bash
cert_tree --file cert.pem --format json
```

Output:
```json
{
  "subject": "CN=example.com,O=Example Inc",
  "issuer": "CN=CA,O=Example Inc",
  "serial_number": "12345678901234567890",
  "not_before": "Mon, 01 Jan 2023 00:00:00 +0000",
  "not_after": "Tue, 31 Dec 2024 23:59:59 +0000",
  "public_key_algorithm": "RSA",
  "signature_algorithm": "SHA256-RSA",
  "version": 3,
  "extensions": [...],
  "is_ca": false,
  "key_usage": "Digital Signature, Key Encipherment",
  "subject_alt_names": ["DNS:example.com"]
}
```

## Dependencies

- `clap`: Command-line argument parsing
- `x509-parser`: X.509 certificate parsing
- `reqwest`: HTTP client for URL fetching
- `serde` & `serde_json`: JSON serialization
- `anyhow` & `thiserror`: Error handling
- `ratatui`: Terminal user interface framework
- `crossterm`: Cross-platform terminal manipulation
- `chrono`: Date/time processing for validity calculations
- `tokio`: Async runtime (for future enhancements)

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