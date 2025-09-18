# Architecture: cert-tree.rs

## System Architecture

### Core Components
```
cert-tree.rs
├── CLI Layer (clap)
│   ├── Argument parsing
│   ├── Help system
│   └── Error handling
├── I/O Layer
│   ├── File loader (PEM/DER)
│   ├── URL fetcher (HTTP/HTTPS)
│   └── Data stream handler
├── Parser Layer (x509-parser)
│   ├── Certificate parsing
│   ├── Extension extraction
│   └── Validation
├── Display Layer
│   ├── Tree formatter
│   ├── JSON serializer
│   └── Verbose text output
└── Error Handling Layer
    ├── Custom error types
    ├── Result propagation
    └── User-friendly messages
```

### Data Flow
1. **Input Processing**: CLI args → Input validation → Data loading
2. **Parsing**: Raw bytes → x509-parser → CertificateInfo struct
3. **Processing**: CertificateInfo → Extension analysis → Display preparation
4. **Output**: Formatted display → stdout/file → User

## Source Code Structure

### Key Files
- `src/main.rs`: Main application logic (357 lines)
  - CLI argument parsing
  - Certificate loading functions
  - Parsing and display logic
  - Error handling
  - Unit tests

- `Cargo.toml`: Project configuration
  - Dependencies management
  - Build profiles
  - Metadata

- `README.md`: User documentation
  - Installation instructions
  - Usage examples
  - Feature descriptions

### Module Organization
```
src/
├── main.rs (single-file application)
│   ├── CLI argument parsing (Args struct)
│   ├── Error types (CertError enum)
│   ├── Data structures (CertificateInfo, ExtensionInfo)
│   ├── Core functions:
│   │   ├── load_certificate_from_file()
│   │   ├── load_certificate_from_url()
│   │   ├── parse_certificate()
│   │   ├── extract_cert_info()
│   │   ├── display_tree()
│   │   ├── display_verbose()
│   └── Tests module
```

## Key Technical Decisions

### Language & Framework
- **Rust 2021**: Memory safety, performance, modern tooling
- **Single binary**: No runtime dependencies, easy distribution
- **Command-line interface**: Familiar UX for system tools

### Dependencies
- **x509-parser**: Pure Rust X.509 parsing, no C dependencies
- **clap**: Industry standard CLI parsing with derive macros
- **reqwest**: Async HTTP client with blocking API for simplicity
- **serde**: JSON serialization with derive macros
- **thiserror**: Ergonomic error handling

### Design Patterns
- **Error handling**: Custom error types with thiserror for user-friendly messages
- **Builder pattern**: clap derive for CLI argument parsing
- **Result propagation**: Comprehensive error handling throughout
- **Separation of concerns**: Clear separation between I/O, parsing, and display

### Performance Considerations
- **Zero-copy parsing**: Where possible using x509-parser's efficient parsing
- **Release optimization**: LTO and opt-level 3 for maximum performance
- **Minimal allocations**: Efficient string handling and data structures
- **Fast startup**: Single binary with no initialization overhead

## Component Relationships

### CLI ↔ Parser
- CLI validates input sources and output formats
- Parser receives validated data and returns structured results
- Error propagation from parser to CLI for user feedback

### Parser ↔ Display
- Parser produces CertificateInfo structs
- Display layer consumes structs and formats for output
- Display handles different output formats (tree, JSON, verbose)

### I/O ↔ Parser
- I/O layer handles different input sources (file, URL, data)
- Parser receives raw bytes regardless of source
- Unified error handling across all input methods

## Critical Implementation Paths

### Certificate Loading Path
```
Input Source → Data Loading → Format Detection → Parsing → Validation → Display
```

### Error Handling Path
```
Error Occurrence → Error Type Matching → User Message → Exit Code → Logging
```

### Extension Processing Path
```
Raw Extensions → OID Extraction → Name Resolution → Criticality Check → Display Formatting
```

## Security Considerations
- **Input validation**: All inputs validated before processing
- **Memory safety**: Rust guarantees prevent buffer overflows
- **No arbitrary code execution**: Pure data processing, no eval or similar
- **HTTPS validation**: Uses system certificate store for URL fetching

## Future Extensibility
- **Plugin system**: Could add custom parsers for exotic formats
- **Output formats**: Easy to add new display formats
- **Input sources**: Modular I/O layer supports new sources
- **Extension handlers**: Pluggable extension processing