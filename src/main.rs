//! # cert-tree.rs - X.509 Certificate Inspection Utility
//!
//! This crate provides a command-line tool for inspecting and displaying X.509 certificates
//! in a human-readable tree format. It supports multiple input sources (files, URLs) and
//! output formats (tree view, verbose text, interactive TUI).
//!
//! ## Features
//!
//! - Parse certificates from PEM/DER files or HTTPS URLs
//! - Display certificate chains with hierarchical tree structure
//! - Interactive TUI with color-coded validity status
//! - Text mode for non-interactive environments
//! - Comprehensive certificate information including extensions
//!
//! ## Usage
//!
//! ```bash
//! # Inspect a certificate file
//! cert-tree --file certificate.pem
//!
//! # Inspect certificates from a website
//! cert-tree --url https://example.com
//!
//! # Interactive mode
//! cert-tree --file certificate.pem --interactive
//! ```

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod cli;
mod display;
mod error;
mod io;
mod models;
mod parser;
mod tree;

use std::error::Error;

use cli::parse_args;
use display::{
    display_certificate_tree_text, display_certificate_tree_tui, display_tui, display_verbose,
};
use io::{fetch_certificate_chain_from_url, load_certificate_from_file};
use parser::parse_certificate_chain;
use tree::build_certificate_tree;

fn main() -> Result<(), Box<dyn Error>> {
    let args = parse_args();

    let certificates = if let Some(file) = args.file.as_ref() {
        let data = load_certificate_from_file(file)?;
        parse_certificate_chain(&data)?
    } else if let Some(url) = args.url.as_ref() {
        fetch_certificate_chain_from_url(url)?
    } else {
        // Unreachable due to CLI validation
        unreachable!();
    };

    if certificates.len() == 1 {
        let cert_info = &certificates[0];

        if args.interactive {
            display_tui(cert_info)?;
        } else {
            display_verbose(cert_info);
        }
    } else {
        let tree = build_certificate_tree(certificates);

        if args.interactive {
            display_certificate_tree_tui(&tree)?;
        } else {
            display_certificate_tree_text(&tree);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CertError;
    use crate::parser::parse_certificate;

    #[test]
    fn test_parse_certificate_invalid_data() {
        let invalid_data = b"invalid certificate data";
        let result = parse_certificate(invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_certificate_from_file_not_found() {
        let result = load_certificate_from_file("nonexistent.pem");
        assert!(matches!(result, Err(CertError::NotFound)));
    }

    #[test]
    fn test_display_tree() {
        let cert = CertificateInfo {
            subject: "CN=example.com".to_string(),
            issuer: "CN=CA".to_string(),
            serial_number: "12345".to_string(),
            not_before: "2023-01-01".to_string(),
            not_after: "2024-01-01".to_string(),
            public_key_algorithm: "RSA".to_string(),
            signature_algorithm: "SHA256-RSA".to_string(),
            version: 3,
            extensions: vec![
                crate::models::ExtensionInfo {
                    oid: "2.5.29.14".to_string(),
                    name: crate::parser::oid_to_name("2.5.29.14"),
                    critical: false,
                    value: "KeyIdentifier(...)".to_string(),
                },
                crate::models::ExtensionInfo {
                    oid: "2.5.29.17".to_string(),
                    name: crate::parser::oid_to_name("2.5.29.17"),
                    critical: false,
                    value: "GeneralNames(...)".to_string(),
                },
            ],
            is_ca: false,
            key_usage: Some("Digital Signature".to_string()),
            subject_alt_names: vec!["example.com".to_string()],
        };

        // This will print to stdout, but we can't easily test output
        // In a real scenario, we'd capture stdout or use a different approach
        display::display_tree(&cert, "", true);
    }

    #[test]
    fn test_certificate_info_creation() {
        let cert = CertificateInfo {
            subject: "CN=test".to_string(),
            issuer: "CN=issuer".to_string(),
            serial_number: "67890".to_string(),
            not_before: "2023-01-01".to_string(),
            not_after: "2024-01-01".to_string(),
            public_key_algorithm: "ECDSA".to_string(),
            signature_algorithm: "SHA256-ECDSA".to_string(),
            version: 3,
            extensions: vec![],
            is_ca: true,
            key_usage: None,
            subject_alt_names: vec![],
        };

        // Test basic field access
        assert_eq!(cert.subject, "CN=test");
        assert_eq!(cert.issuer, "CN=issuer");
        assert_eq!(cert.is_ca, true);
        assert_eq!(cert.version, 3);
    }
}
