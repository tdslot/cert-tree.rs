# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.1] - 2025-09-19

### Changed
- **ISO 8601 Date Format**: Updated validity period display in TUI to use standardized ISO 8601 date-time format (YYYY-MM-DDTHH:MM:SSZ)
- **Enhanced Date Display**: Both single certificate and certificate chain TUI modes now show validity periods in ISO 8601 format for better consistency and readability

### Technical
- **Date Formatting**: Modified `display_tui()` and `display_certificate_tree_tui()` functions to parse RFC 2822 dates and format them as ISO 8601 UTC timestamps
- **Improved Consistency**: Standardized date display across all TUI modes for better user experience

## [0.8.0] - 2025-09-19

### Added
- **Enhanced Navigation System**: Completely redesigned TUI navigation for improved usability
- **Tab-based Details Pane Activation**: Press Tab to toggle focus between certificate list and details pane
- **Context-aware Arrow Keys**: ↑/↓ keys navigate list when details inactive, scroll details when active
- **Page Up/Page Down Support**: Fast navigation through certificate lists (10 items at a time)
- **Visual Feedback**: Both certificate list and details pane show activation state with color-coded borders and dynamic titles
- **Terminal-Adaptive Background**: Removed background colors to dynamically match terminal window color scheme
- **Transparent Styling**: Background adapts to user's terminal theme for better integration
- **Dynamic Footer Instructions**: Context-sensitive help text that changes based on current focus

### Changed
- **Replaced 'j'/'k' Key Bindings**: Removed vi-style navigation in favor of more intuitive Tab-based system
- **Enhanced User Experience**: More discoverable and accessible navigation patterns
- **Improved Responsiveness**: Better handling of long certificate lists and details

### Technical
- **State Management**: Added `details_pane_active` flag for focus tracking
- **Enhanced Key Handling**: Context-aware key processing based on active pane
- **Visual State Indicators**: Dynamic UI elements that reflect current navigation state
- **Improved Code Documentation**: Added comprehensive comments explaining the new navigation system

### Documentation
- **README.md Update**: Fixed CLI options documentation to match actual implementation
- **Removed Invalid Options**: Removed references to non-existent --data, --verbose, and --format options
- **Updated Examples**: Cleaned up usage examples to use only existing CLI options
- **Verified Accuracy**: Confirmed all CLI references match the actual binary output

## [0.7.0] - 2025-09-19

### Added
- Enhanced interactive TUI with detailed certificate information display
- Added certificate details section in TUI that shows comprehensive certificate information when selecting a certificate from the chain
- Implemented scrollable certificate details panel with 'j'/'k' keys for navigation
- Added real-time certificate details update upon selection change
- Enhanced TUI layout with 4 sections: title, certificate list, certificate details, and footer
- Improved user experience with detailed certificate inspection including:
  - Issuer information and subject details
  - Validity period with color-coded status
  - Serial number and version information
  - Public key algorithm and signature algorithm details
  - Key usage and subject alternative names
  - Certificate extensions with criticality indicators
  - CA status and other certificate attributes

### Changed
- Updated TUI layout from 3 sections to 4 sections for better information display
- Enhanced footer instructions to include details scrolling ('j'/'k' keys)
- Improved certificate selection mechanism with direct access to certificate data
- Restructured certificate display data structure for better TUI integration

### Technical
- Added `CertificateDisplayItem` struct to encapsulate display and certificate data
- Modified `flatten_certificate_tree()` to include certificate information
- Updated TUI rendering to support detailed certificate information panel
- Implemented scroll state management for certificate details section
- Enhanced keyboard handling for both navigation and details scrolling

## [0.6.1] - 2025-09-19

### Maintenance
- Performed comprehensive code review and optimization
- Removed unused dependencies: tokio, tokio-rustls, term_size
- Cleaned up obsolete commented code in certificate parsing functions
- Optimized dependency tree for faster compilation and smaller binary size

### Technical
- Removed 3 unused dependencies from Cargo.toml
- Cleaned up commented extension parsing code in extract_cert_info()
- Maintained all existing functionality while reducing code complexity
- Improved code maintainability and reduced compilation time

## [0.6.0] - 2025-09-19

### Added
- Modified certificate display to show only CN (Common Name) instead of full subject
- Cleaner output by displaying just the certificate name without full DN details
- Added extract_cn() function to parse CN from certificate subject strings

### Changed
- Updated all display modes (tree, verbose, TUI, text) to use CN only
- Simplified certificate identification in output

### Technical
- Added extract_cn() function for parsing CN from X.509 subject fields
- Modified display_tree(), display_verbose(), display_tui(), and tree display functions
- Maintained all existing functionality while improving readability

## [0.5.1] - 2025-09-19

### Fixed
- Fixed Unicode string slicing bug in TUI display that caused panics when resizing terminal with certificates containing multi-byte characters
- Replaced byte-based string slicing with character-based slicing to properly handle UTF-8 characters
- Fixed TUI layout initialization issue that caused display problems for a few seconds on startup
- Added terminal clear and small delay on TUI startup to ensure proper layout calculation
- Fixed time format in text mode to include seconds (HH:MM:SS instead of HH:MM)
- Updated date column alignment in text mode to accommodate longer time format
- Added adaptive date formatting in TUI for narrow terminals (full date/time, date/time without seconds, or date only)
- Optimized terminal width thresholds to prioritize showing seconds when possible
- Reduced minimum spacing requirements for better date display in moderately narrow terminals
- Added safeguards for extremely narrow terminals to prevent formatting panics
- Ensured minimum column widths are maintained for proper display
- Shifted date column in TUI to the left by 8 characters for better visual alignment
- Verified time format fix works correctly in both TUI and text modes

### Technical
- Updated `display_certificate_tree_tui()` and `display_tree_node_text()` functions to use `chars().take()` instead of byte slicing
- Improved Unicode compatibility for certificate names with special characters

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