# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2025-09-19

### Added
- Implemented sequence numbers in separate square brackets [1], [2], etc.
- Reformatted status/date display with square brackets: [VALID until: YYYY-MM-DD HH:MM]
- Separated sequence numbers from certificate names for cleaner display
- Enhanced visual formatting with dual bracket system

### Changed
- Updated text mode output format to use bracketed sequence numbers and status
- Improved readability by separating numbering from certificate names
- Enhanced visual consistency with uniform bracket formatting

### Technical
- Modified `display_tree_node_text()` function to support bracketed formatting
- Updated sequence number display logic for separate bracket placement
- Maintained color coding and alignment while improving format structure

## [0.4.0] - 2025-09-18

### Added
- Implemented cascading tree structure in text mode with clean indentation
- Perfect column alignment for dates in text output
- Enhanced tree visualization with proper depth-based indentation
- Improved visual hierarchy with cascading `└` connectors

### Changed
- Restructured text mode tree display for better readability
- Updated indentation system to use 4-space increments per level
- Improved column positioning for consistent date alignment
- Enhanced tree structure to match user-specified format

### Technical
- Refactored `display_tree_node_text()` function for cascading display
- Implemented depth-based indentation calculation
- Added fixed column positioning for date alignment
- Maintained color coding while improving tree structure

## [0.3.0] - 2025-09-18

### Added
- Enhanced text mode display with color-coded certificate validity status
- Aligned date columns in text output for consistent formatting
- Simplified tree structure with clean connectors (├── └──)
- Smart truncation of long certificate names with ellipsis
- Terminal width detection for responsive column alignment

### Changed
- Improved text mode visual hierarchy and readability
- Better spacing and alignment in certificate tree display
- Enhanced color coding for validity status (green/yellow/red)

### Technical
- Added `term_size` dependency for terminal width detection
- Implemented dynamic column positioning based on terminal size
- Enhanced `display_tree_node_text()` function with color support
- Added ANSI color codes for cross-platform terminal coloring

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
- HTTPS certificate chain fetching from any website URL
- TLS handshake certificate extraction using rustls library
- Support for both direct certificate URLs and regular HTTPS websites
- Improved text mode certificate tree formatting with clean visual hierarchy
- Better validity status display with consistent date formatting
- Added color-coded output in text mode (green for valid, yellow for expiring, red for expired)
- Implemented aligned date columns with consistent positioning
- Simplified tree structure with clean connectors (├── └──)
- Smart truncation of long certificate names with ellipsis
- Implemented cascading tree structure with proper indentation levels
- Added sequence numbers in separate square brackets [1], [2], etc. before status/date
- Moved VALID status to front of date/time format: "[VALID until: YYYY-MM-DD HH:MM]"
- Separated sequence numbers from certificate names for cleaner display

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