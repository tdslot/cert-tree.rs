# AGENTS.md

This file provides guidelines for agentic coding agents working on the cert-tree.rs project.

## Build, Test, and Lint Commands

### Primary Commands (via Justfile)
```bash
just test              # Run all tests
just test-verbose      # Run tests with output (--nocapture)
just fmt               # Format code with rustfmt
just clippy            # Run Clippy linter
just check             # Check compilation without building
just build             # Build debug version
just build-release     # Build optimized release version
```

### Running Single Tests
```bash
# Run a specific test function
cargo test test_function_name

# Run tests in a specific module
cargo test module_name::tests

# Run tests matching a pattern
cargo test pattern

# Run tests with output
cargo test test_name -- --nocapture
```

### Quality Assurance Workflows
```bash
just dev-check         # Quick check: fmt + clippy + build
just quality           # Full QA: fmt + clippy + test + build
just quality-release   # Release QA: fmt + clippy + test + build-release
```

### Direct Cargo Commands
```bash
cargo check            # Check compilation
cargo build            # Build
cargo test             # Run tests
cargo clippy           # Lint
cargo fmt              # Format
```

## Code Style Guidelines

### Imports and Module Organization
- Group imports alphabetically or logically
- Use `use crate::*` for internal modules
- External dependencies first, then local crate modules
- Prefer specific imports over glob imports

### Formatting
- Use `cargo fmt` or `just fmt` before committing
- No custom rustfmt.toml (uses Rust defaults)
- Maximum line length: 100 characters (implied)

### Naming Conventions
- Functions and variables: `snake_case`
- Types and structs: `PascalCase` (CamelCase)
- Constants: `SCREAMING_SNAKE_CASE`
- Module files: lowercase with underscores (e.g., `error.rs`, `cli.rs`)

### Types and Data Structures
- Use `#[derive(Debug, Clone)]` for data types
- Add `#[derive(Serialize, Deserialize)]` if using serde
- Prefer `Option<T>` over `None` values in structs
- Use `Result<T, E>` for fallible operations
- Define custom error types using `thiserror::Error`

### Error Handling
- Use `thiserror` crate for custom error types
- Implement `From` traits for error conversions
- Use `anyhow` for main() error handling only
- Error messages should be descriptive and user-friendly
- Pattern match on specific error variants when needed

### Documentation
- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include example usage in doc comments
- Document non-obvious behavior and invariants

### Testing
- Place tests in `#[cfg(test)]` modules at file bottom
- Use descriptive test function names: `test_<feature>_<scenario>`
- Test both success and error paths
- Use `assert!`, `assert_eq!`, `assert_matches!` macros
- Integration tests in `test/` directory for test certificates

### Linting
- Clippy is configured with pedantic level in Cargo.toml
- All warnings are treated as errors (warn level)
- Key enabled lints: complexity, correctness, perf, style, suspicious
- Use `#![allow]` sparingly and only in main.rs when necessary

### CLI (clap)
- Use `#[derive(Parser)]` for CLI arguments
- Use `#[derive(Subcommand)]` for subcommands
- Provide help text for all arguments
- Use `short` and `long` flags consistently
- Document usage examples in CLI help

### Serialization (serde)
- Add `#[derive(Serialize, Deserialize)]` to data types needing serialization
- Use serde for configuration and data export formats
- Handle serialization errors gracefully

### Performance Considerations
- Uses mimalloc as global allocator (configured in main.rs)
- Release builds optimized with LTO, opt-level 3, single codegen-unit
- Prefer references over copying where possible
- Use efficient data structures for large datasets

### Code Organization
- One module per file in `src/` directory
- Main entry point in `src/main.rs`
- Separate concerns: `cli`, `error`, `io`, `parser`, `display`, `models`, `tree`, `completions`
- Keep functions focused and reasonably sized
- Extract complex logic into helper functions

### Dependencies
- Add dependencies using `cargo add` or manually to Cargo.toml
- Prefer well-maintained crates with good documentation
- Use `cargo update` to update dependencies
- Review dependency security with `just audit`

### Git Workflow
- Run `just quality` before committing
- Use conventional commit messages
- Update CHANGELOG.md for user-facing changes
- Tag releases following semantic versioning

### Release Process
- Use `just prepare-release` for full QA
- Create signed git tags for releases
- GitHub Actions builds multi-platform binaries
- Release notes generated from CHANGELOG.md

## Project-Specific Patterns

### Certificate Parsing
- Parse PEM/DER formats using `x509-parser` crate
- Extract CN (Common Name) from subject DNs for cleaner display
- Map OIDs to human-readable extension names
- Handle certificate chain hierarchies

### Display Output
- Support both text and TUI modes
- Use color coding for validity status (green/yellow/red)
- Format dates in ISO 8601 format when possible
- Handle long content with text wrapping

### TUI (ratatui)
- Use ratatui for terminal UI
- Implement responsive layouts
- Support interactive navigation (arrow keys, Tab, Page Up/Down)
- Provide quit functionality with 'q' key
