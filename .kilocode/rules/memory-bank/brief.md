# cert-tree.rs - X.509 Certificate Inspection Utility

## Project Overview
A command-line utility written in Rust for inspecting and displaying X.509 certificates in a human-readable tree format. Inspired by the `cert_tree` project, this tool provides comprehensive certificate analysis with multiple output formats.

## Core Purpose
To provide security professionals, developers, and system administrators with an efficient, reliable tool for examining X.509 certificate details including:
- Certificate chains and hierarchies
- Public key information
- Validity periods
- Extensions and constraints
- Issuer and subject details

## Key Requirements
- **Input Sources**: Files (PEM/DER), URLs, command-line data
- **Output Formats**: Tree view, JSON, verbose text, interactive TUI
- **Certificate Chains**: Automatic parsing and display of certificate hierarchies
- **Performance**: Fast parsing with minimal memory footprint
- **Reliability**: Comprehensive error handling and validation
- **Usability**: Clean, human-readable output with proper formatting and color coding

## Technical Foundation
- **Language**: Rust with 2021 edition
- **Key Dependencies**:
  - `x509-parser`: Certificate parsing
  - `clap`: CLI argument handling
  - `reqwest`: HTTP client for URL fetching
  - `serde`: Serialization for JSON output
  - `ratatui`: Terminal user interface framework
  - `crossterm`: Cross-platform terminal manipulation
  - `chrono`: Date/time processing for validity calculations
- **Architecture**: Single binary with modular design for parsing, display, TUI, and I/O operations

## Success Criteria
- Parses all common certificate formats correctly
- Provides clear, professional output
- Handles edge cases and invalid certificates gracefully
- Maintains high performance for large certificate chains
- Easy to install and use across different platforms