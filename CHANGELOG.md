# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-09-18

### Added
- Enhanced interactive TUI for certificate chain inspection with:
  - Scrollable list of certificates using arrow keys
  - Dynamic column headers ("Certificate Name" and "Valid Until")
  - Right-aligned date/time column formatted as YYYY-MM-DD HH:MM:SS
  - Color-coded validity status (green/yellow/red)
  - Keyboard navigation (↑/↓ to scroll, 'q' to quit, 't' for text mode)
  - Proper handling of long certificate names with truncation and ellipsis
  - Responsive layout that adapts to terminal width changes
  - Dynamic column sizing with left-aligned names anchored to left edge
  - Right-aligned dates anchored to right edge with consistent spacing

### Changed
- Improved TUI layout with better column alignment and spacing
- Enhanced user experience with visual feedback for selected items

### Technical
- Added `flatten_certificate_tree()` function to convert tree structure to flat list
- Updated TUI rendering to use ratatui List widget with proper styling
- Improved date formatting and parsing for consistent display

## [0.1.0] - 2025-09-18

### Added
- Initial release of cert-tree.rs certificate inspection utility
- Support for multiple input sources (files, URLs, command-line data)
- Multiple output formats (tree, JSON, verbose, TUI)
- Certificate chain parsing and hierarchical display
- Interactive TUI with color-coded validity status
- Comprehensive error handling and validation
- Cross-platform compatibility

### Features
- X.509 certificate parsing using x509-parser
- Tree-based certificate chain visualization
- Color coding for certificate expiry status
- Text mode for non-interactive environments
- JSON output for programmatic access