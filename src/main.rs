use chrono::{DateTime, Utc};
use clap::{CommandFactory, Parser};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Terminal,
};
use rustls::{ClientConfig, RootCertStore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use webpki_roots::TLS_SERVER_ROOTS;
use x509_parser::prelude::*;
use std::str;
// CRL parsing will be implemented later

// Function to extract CN from certificate subject
fn extract_cn(subject: &str) -> String {
    // Parse the DN format: C=US, ST=New Jersey, L=Jersey City, O=The USERTRUST Network, CN=USERTrust RSA Cer...
    let parts: Vec<&str> = subject.split(',').collect();

    for part in parts {
        let trimmed = part.trim();
        if trimmed.starts_with("CN=") {
            return trimmed[3..].to_string(); // Remove "CN=" prefix
        }
    }

    // If no CN found, return the whole subject as fallback
    subject.to_string()
}

// Function to map OID to human-readable extension name
fn oid_to_name(oid: &str) -> Option<String> {
    match oid {
        // Standard X.509 extensions
        "2.5.29.14" => Some("Subject Key Identifier".to_string()),
        "2.5.29.15" => Some("Key Usage".to_string()),
        "2.5.29.16" => Some("Private Key Usage Period".to_string()),
        "2.5.29.17" => Some("Subject Alternative Name".to_string()),
        "2.5.29.18" => Some("Issuer Alternative Name".to_string()),
        "2.5.29.19" => Some("Basic Constraints".to_string()),
        "2.5.29.30" => Some("Name Constraints".to_string()),
        "2.5.29.31" => Some("CRL Distribution Points".to_string()),
        "2.5.29.32" => Some("Certificate Policies".to_string()),
        "2.5.29.33" => Some("Policy Mappings".to_string()),
        "2.5.29.35" => Some("Authority Information Access".to_string()),
        "2.5.29.36" => Some("Policy Constraints".to_string()),
        "2.5.29.37" => Some("Extended Key Usage".to_string()),
        "2.5.29.46" => Some("Freshest CRL".to_string()),

        // Microsoft extensions
        "1.3.6.1.4.1.311.20.2" => Some("Microsoft Smart Card Login".to_string()),
        "1.3.6.1.4.1.311.21.1" => Some("Microsoft Individual Code Signing".to_string()),

        // Entrust extensions
        "1.2.840.113533.7.65.0" => Some("Entrust Version Information".to_string()),

        // Netscape extensions
        "2.16.840.1.113730.1.1" => Some("Netscape Certificate Type".to_string()),

        // VeriSign extensions
        "2.23.42.7.0" => Some("VeriSign Individual SHA1 Hash".to_string()),

        // Other common extensions
        "1.3.6.1.5.5.7.1.1" => Some("Authority Information Access".to_string()),
        "1.3.6.1.4.1.11129.2.4.2" => Some("Signed Certificate Timestamp".to_string()),
        _ => None,
    }
}



// Function to map signature algorithm OID to human-readable name
fn signature_alg_to_name(oid_str: &str) -> Option<String> {
    match oid_str {
        "1.2.840.113549.1.1.1" => Some("RSA with MD5".to_string()),
        "1.2.840.113549.1.1.4" => Some("RSA with MD5".to_string()),
        "1.2.840.113549.1.1.5" => Some("SHA1 with RSA".to_string()),
        "1.2.840.113549.1.1.11" => Some("SHA256 with RSA".to_string()),
        "1.2.840.113549.1.1.12" => Some("SHA384 with RSA".to_string()),
        "1.2.840.113549.1.1.13" => Some("SHA512 with RSA".to_string()),
        "1.3.14.3.2.29" => Some("SHA1 with RSA".to_string()),
        "1.2.840.10045.4.1" => Some("SHA1 with ECDSA".to_string()),
        "1.2.840.10045.4.3.2" => Some("SHA256 with ECDSA".to_string()),
        "1.2.840.10045.4.3.3" => Some("SHA384 with ECDSA".to_string()),
        "1.2.840.10045.4.3.4" => Some("SHA512 with ECDSA".to_string()),
        "1.2.840.10040.4.3" => Some("SHA1 with DSA".to_string()),
        _ => None,
    }
}

// Function to explain signature algorithm in simple terms
fn explain_signature_algorithm(alg: &str) -> String {
    if alg.contains("RSA") {
        "This certificate uses RSA encryption with hashing. RSA is like a digital lock that only the certificate issuer has the key to open. The hashing creates a unique fingerprint of the certificate data. Together, they create a digital signature that proves the certificate is genuine and hasn't been tampered with. This is essential for secure websites and encrypted communications.".to_string()
    } else if alg.contains("ECDSA") {
        "This certificate uses Elliptic Curve Digital Signature Algorithm (ECDSA). It's a modern, efficient way to create digital signatures using advanced mathematics with elliptic curves. Like RSA, it creates a unique signature that proves the certificate's authenticity, but it's faster and uses smaller keys. This helps keep internet communications secure and private.".to_string()
    } else if alg.contains("DSA") {
        "This certificate uses Digital Signature Algorithm (DSA). It's a method for creating digital signatures that verify the authenticity of the certificate. Using mathematical techniques, it creates a unique code that only the legitimate issuer can produce. This prevents fake certificates and ensures trust in online communications.".to_string()
    } else {
        "This is a cryptographic signature method that verifies the certificate's authenticity. It uses mathematical algorithms to create a unique digital signature that proves the certificate is legitimate and hasn't been altered. This is crucial for establishing secure and trustworthy connections on the internet.".to_string()
    }
}

impl From<rustls::Error> for CertError {
    fn from(err: rustls::Error) -> Self {
        CertError::Tls(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum CertError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("TLS error: {0}")]
    Tls(String),
    #[error("X.509 parsing error: {0}")]
    X509Parse(String),
    #[error("CRL parsing error: {0}")]
    CrlParse(String),
    #[error("Invalid certificate format")]
    InvalidFormat,
    #[error("Certificate not found")]
    NotFound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub not_before: String,
    pub not_after: String,
    pub public_key_algorithm: String,
    pub signature_algorithm: String,
    pub version: u32,
    pub extensions: Vec<ExtensionInfo>,
    pub is_ca: bool,
    pub key_usage: Option<String>,
    pub subject_alt_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CertificateNode {
    pub cert: CertificateInfo,
    pub index: usize,
    pub children: Vec<CertificateNode>,
    pub validity_status: ValidityStatus,
    pub validation_status: ValidationStatus,
}

#[derive(Debug, Clone)]
pub struct CertificateTree {
    pub roots: Vec<CertificateNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionInfo {
    pub oid: String,
    pub name: Option<String>,
    pub critical: bool,
    pub value: String,
}

#[derive(Parser)]
#[command(name = "cert-tree")]
#[command(about = "X.509 certificate inspection utility")]
#[command(version)]
pub struct Args {
    /// Certificate file path (PEM or DER)
    #[arg(short, long)]
    pub file: Option<String>,

    /// Certificate URL
    #[arg(short = 'U', long)]
    pub url: Option<String>,

    /// Interactive TUI mode
    #[arg(short = 'i', long, default_value = "false")]
    pub interactive: bool,

    /// Force text output mode (non-interactive)
    #[arg(short = 't', long, default_value = "true")]
    pub text: bool,

}

pub fn load_certificate_from_file(path: &str) -> Result<Vec<u8>, CertError> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(CertError::NotFound);
    }

    let data = fs::read(path)?;
    Ok(data)
}

pub fn load_certificate_from_url(url: &str) -> Result<Vec<u8>, CertError> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;
    let data = response.bytes()?;
    Ok(data.to_vec())
}

pub fn fetch_certificate_chain_from_url(url: &str) -> Result<Vec<CertificateInfo>, CertError> {
    // Parse the URL to extract hostname
    let url_parsed = url::Url::parse(url).map_err(|_| CertError::InvalidFormat)?;
    let hostname = url_parsed.host_str().ok_or(CertError::InvalidFormat)?;

    // First, try to fetch as direct certificate data (for URLs like cacert.pem)
    let client = reqwest::blocking::Client::new();
    match client.get(url).send() {
        Ok(response) => {
            let data = response.bytes()?;
            let content = String::from_utf8_lossy(&data);

            // Check if the URL contains certificate data
            if content.contains("-----BEGIN CERTIFICATE-----") {
                return parse_certificate_chain(&data);
            }
        }
        Err(_) => {
            // If direct fetch fails, try to get certificate chain from HTTPS connection
        }
    }

    // For HTTPS URLs, establish a TLS connection and capture the certificate chain
    fetch_certificate_chain_via_tls(hostname)
}

fn fetch_certificate_chain_via_tls(hostname: &str) -> Result<Vec<CertificateInfo>, CertError> {
    use rustls::client::ClientConnection;
    use std::io::{Read, Write};

    // Set up TLS configuration
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    // Create a TCP connection
    let mut socket = std::net::TcpStream::connect((hostname, 443))?;
    socket.set_read_timeout(Some(std::time::Duration::from_secs(10)))?;
    socket.set_write_timeout(Some(std::time::Duration::from_secs(10)))?;

    let server_name =
        rustls::ServerName::try_from(hostname).map_err(|_| CertError::InvalidFormat)?;

    let mut conn = ClientConnection::new(Arc::new(config), server_name)?;

    // Perform TLS handshake
    let mut tls_stream = rustls::Stream::new(&mut conn, &mut socket);

    // Send a minimal HTTP request to trigger the handshake
    let request = format!("GET / HTTP/1.0\r\nHost: {}\r\n\r\n", hostname);
    tls_stream.write_all(request.as_bytes())?;

    // Read response to complete handshake
    let mut buffer = [0u8; 1024];
    let _ = tls_stream.read(&mut buffer);

    // Extract certificate chain from the connection
    if let Some(certs) = conn.peer_certificates() {
        let mut certificates = Vec::new();
        for cert_der in certs {
            match X509Certificate::from_der(cert_der.as_ref()) {
                Ok((_, cert)) => {
                    let cert_info = extract_cert_info(&cert)?;
                    certificates.push(cert_info);
                }
                Err(e) => {
                    return Err(CertError::X509Parse(format!(
                        "Failed to parse certificate: {}",
                        e
                    )));
                }
            }
        }
        Ok(certificates)
    } else {
        Err(CertError::X509Parse(
            "No certificates found in TLS handshake".to_string(),
        ))
    }
}

pub fn parse_certificate(data: &[u8]) -> Result<CertificateInfo, CertError> {
    // Try PEM first
    if let Ok((_, pem)) = pem::parse_x509_pem(data) {
        let cert = pem
            .parse_x509()
            .map_err(|e| CertError::X509Parse(e.to_string()))?;
        return extract_cert_info(&cert);
    }

    // Try DER
    let cert = X509Certificate::from_der(data)
        .map_err(|e| CertError::X509Parse(e.to_string()))?
        .1;

    extract_cert_info(&cert)
}

pub fn parse_certificate_chain(data: &[u8]) -> Result<Vec<CertificateInfo>, CertError> {
    let mut certificates = Vec::new();

    // Try to parse as PEM with multiple certificates
    let mut remaining = data;
    while !remaining.is_empty() {
        match pem::parse_x509_pem(remaining) {
            Ok((rest, pem)) => {
                remaining = rest;
                match pem.parse_x509() {
                    Ok(cert) => {
                        let cert_info = extract_cert_info(&cert)?;
                        certificates.push(cert_info);
                    }
                    Err(_) => break, // Stop if we can't parse a certificate
                }
            }
            Err(_) => break, // No more PEM blocks
        }
    }

    // If no PEM certificates found, try single DER
    if certificates.is_empty() {
        match X509Certificate::from_der(data) {
            Ok((_, cert)) => {
                let cert_info = extract_cert_info(&cert)?;
                certificates.push(cert_info);
            }
            Err(e) => return Err(CertError::X509Parse(e.to_string())),
        }
    }

    Ok(certificates)
}

pub fn build_certificate_tree(certificates: Vec<CertificateInfo>) -> CertificateTree {
    let mut cert_map: HashMap<String, CertificateInfo> = HashMap::new();
    let mut issuer_map: HashMap<String, Vec<String>> = HashMap::new();

    // Build maps for quick lookup
    for cert in &certificates {
        cert_map.insert(cert.subject.clone(), cert.clone());

        // Group certificates by issuer
        issuer_map
            .entry(cert.issuer.clone())
            .or_insert_with(Vec::new)
            .push(cert.subject.clone());
    }

    // Find root certificates (self-signed or where issuer is not in our set)
    let mut roots = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for cert in &certificates {
        if !cert_map.contains_key(&cert.issuer) || cert.subject == cert.issuer {
            // This is a root certificate
            if !processed.contains(&cert.subject) {
                let node = build_tree_node(cert, &cert_map, &issuer_map, &mut processed, 1);
                roots.push(node);
            }
        }
    }

    // Handle any remaining certificates that might not have been processed
    for cert in &certificates {
        if !processed.contains(&cert.subject) {
            let node = build_tree_node(cert, &cert_map, &issuer_map, &mut processed, 1);
            roots.push(node);
        }
    }

    let mut tree = CertificateTree { roots };
    validate_certificate_chain(&mut tree);
    tree
}

#[derive(Clone)]
struct CertificateDisplayItem {
    display_name: String,
    valid_until: String,
    validity_status: ValidityStatus,
    validation_status: ValidationStatus,
    certificate_info: CertificateInfo,
}

fn flatten_certificate_tree(tree: &CertificateTree) -> Vec<CertificateDisplayItem> {
    let mut certificates = Vec::new();
    for root in &tree.roots {
        flatten_node(root, &mut certificates, 0);
    }
    certificates
}

fn flatten_node(
    node: &CertificateNode,
    certificates: &mut Vec<CertificateDisplayItem>,
    depth: usize,
) {
    // Format certificate name with indentation - use only CN
    let indent = "  ".repeat(depth);
    let cn = extract_cn(&node.cert.subject);
    let display_name = format!("{}{}", indent, cn);

    // Date is already in the correct format (YYYY-MM-DD HH:MM:SS)
    let valid_until = node.cert.not_after.clone();

    certificates.push(CertificateDisplayItem {
        display_name,
        valid_until,
        validity_status: node.validity_status.clone(),
        validation_status: node.validation_status.clone(),
        certificate_info: node.cert.clone(),
    });

    // Add children
    for child in &node.children {
        flatten_node(child, certificates, depth + 1);
    }
}

fn build_tree_node(
    cert: &CertificateInfo,
    cert_map: &HashMap<String, CertificateInfo>,
    issuer_map: &HashMap<String, Vec<String>>,
    processed: &mut std::collections::HashSet<String>,
    index: usize,
) -> CertificateNode {
    processed.insert(cert.subject.clone());

    let validity_status = ValidityStatus::from_dates(&cert.not_after);

    let mut children = Vec::new();
    if let Some(issued_certs) = issuer_map.get(&cert.subject) {
        for (i, subject) in issued_certs.iter().enumerate() {
            if let Some(child_cert) = cert_map.get(subject) {
                if !processed.contains(subject) {
                    let child_node =
                        build_tree_node(child_cert, cert_map, issuer_map, processed, index + i + 1);
                    children.push(child_node);
                }
            }
        }
    }

    CertificateNode {
        cert: cert.clone(),
        index,
        children,
        validity_status,
        validation_status: ValidationStatus::Valid,
    }
}

fn validate_certificate_chain(tree: &mut CertificateTree) {
    for root in &mut tree.roots {
        validate_node(root, None);
    }
}

fn validate_node(node: &mut CertificateNode, parent_cert: Option<&CertificateInfo>) {
    if let Some(parent) = parent_cert {
        if parent.subject == node.cert.issuer {
            node.validation_status = ValidationStatus::Valid;
        } else {
            node.validation_status = ValidationStatus::InvalidChain;
        }
    } else {
        if node.cert.subject == node.cert.issuer {
            node.validation_status = ValidationStatus::Valid;
        } else {
            node.validation_status = ValidationStatus::InvalidChain;
        }
    }

    for child in &mut node.children {
        validate_node(child, Some(&node.cert));
    }
}

fn extract_cert_info(cert: &X509Certificate) -> Result<CertificateInfo, CertError> {
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = format!("{:x}", cert.serial)
        .as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or("??"))
        .collect::<Vec<_>>()
        .join(" ");
    // Store dates in RFC 2822 format initially, then convert to display format
    let not_before_rfc = cert.validity().not_before.to_rfc2822().unwrap_or_else(|_| "Invalid date".to_string());
    let not_after_rfc = cert.validity().not_after.to_rfc2822().unwrap_or_else(|_| "Invalid date".to_string());

    // Convert to display format
    let not_before = if let Ok(dt) = DateTime::parse_from_rfc2822(&not_before_rfc) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        not_before_rfc
    };
    let not_after = if let Ok(dt) = DateTime::parse_from_rfc2822(&not_after_rfc) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        not_after_rfc
    };

    let public_key_alg = match cert.public_key().parsed() {
        Ok(pk) => match pk {
            x509_parser::public_key::PublicKey::RSA(rsa_key) => {
                let key_size = rsa_key.modulus.len() * 8;
                format!("RSA ({} bits)", key_size)
            }
            x509_parser::public_key::PublicKey::EC(_) => "ECDSA".to_string(),
            x509_parser::public_key::PublicKey::DSA(_) => "DSA".to_string(),
            x509_parser::public_key::PublicKey::GostR3410(_) => "GOST R 34.10".to_string(),
            x509_parser::public_key::PublicKey::GostR3410_2012(_) => {
                "GOST R 34.10-2012".to_string()
            }
            _ => "Unknown".to_string(),
        },
        Err(_) => "Unknown".to_string(),
    };

    let sig_alg_oid = cert.signature_algorithm.algorithm.to_string();
    let signature_algorithm = signature_alg_to_name(&sig_alg_oid)
        .unwrap_or_else(|| format!("{:?}", cert.signature_algorithm.algorithm));

    let mut extensions = Vec::new();
    let key_usage = None;
    let subject_alt_names = Vec::new();

    for ext in cert.extensions() {
        let oid_str = ext.oid.to_string();
        let critical = ext.critical;
        let value = format!("{:?}", ext.value);

        extensions.push(ExtensionInfo {
            oid: oid_str.clone(),
            name: oid_to_name(&oid_str),
            critical,
            value,
        });
    }

    let is_ca = cert.is_ca();

    Ok(CertificateInfo {
        subject,
        issuer,
        serial_number: serial,
        not_before,
        not_after,
        public_key_algorithm: public_key_alg,
        signature_algorithm,
        version: cert.version.0,
        extensions,
        is_ca,
        key_usage,
        subject_alt_names,
    })
}

pub fn display_tree(cert: &CertificateInfo, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    let cn = extract_cn(&cert.subject);
    println!("{}{}{}", prefix, connector, cn);

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    println!(
        "{}{}Issuer: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.issuer
    );
    println!(
        "{}{}Serial: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.serial_number
    );
    println!(
        "{}{}Valid: {} to {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.not_before,
        cert.not_after
    );
    println!(
        "{}{}Public Key: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.public_key_algorithm
    );
    println!(
        "{}{}Signature: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.signature_algorithm
    );
    println!(
        "{}{}Version: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.version
    );
    println!(
        "{}{}Is CA: {}",
        new_prefix,
        if cert.extensions.is_empty() {
            "└── "
        } else {
            "├── "
        },
        cert.is_ca
    );

    if let Some(ku) = &cert.key_usage {
        println!(
            "{}{}Key Usage: {}",
            new_prefix,
            if cert.extensions.is_empty() {
                "└── "
            } else {
                "├── "
            },
            ku
        );
    }

    if !cert.subject_alt_names.is_empty() {
        println!(
            "{}{}Subject Alt Names:",
            new_prefix,
            if cert.extensions.is_empty() {
                "└── "
            } else {
                "├── "
            }
        );
        for (i, san) in cert.subject_alt_names.iter().enumerate() {
            let san_connector =
                if i == cert.subject_alt_names.len() - 1 && cert.extensions.is_empty() {
                    "└── "
                } else {
                    "├── "
                };
            println!(
                "{}{}{}{}",
                new_prefix,
                if cert.extensions.is_empty() {
                    "    "
                } else {
                    "│   "
                },
                san_connector,
                san
            );
        }
    }

    if !cert.extensions.is_empty() {
        println!("{}{}Extensions:", new_prefix, "└── ");
        for (i, ext) in cert.extensions.iter().enumerate() {
            let ext_connector = if i == cert.extensions.len() - 1 {
                "└── "
            } else {
                "├── "
            };
            let ext_prefix = format!("{}{}", new_prefix, "    ");
            let ext_name = ext.name.as_deref().unwrap_or(&ext.oid);
            println!(
                "{}{}{} ({})",
                ext_prefix,
                ext_connector,
                ext_name,
                if ext.critical {
                    "critical"
                } else {
                    "non-critical"
                }
            );
        }
    }
}

pub fn display_verbose(cert: &CertificateInfo) {
    println!("Certificate Information:");
    println!("======================");
    let cn = extract_cn(&cert.subject);
    println!("CN: {}", cn);
    println!("Issuer: {}", cert.issuer);
    println!("Serial Number: {}", cert.serial_number);
    println!("Validity:");
    println!("  Not Before: {}", cert.not_before);
    println!("  Not After: {}", cert.not_after);
    println!("Public Key Algorithm: {}", cert.public_key_algorithm);
    println!("Signature Algorithm: {}", cert.signature_algorithm);
    println!("Version: {}", cert.version);
    println!("Is CA: {}", cert.is_ca);

    if let Some(ku) = &cert.key_usage {
        println!("Key Usage: {}", ku);
    }

    if !cert.subject_alt_names.is_empty() {
        println!("Subject Alternative Names:");
        for san in &cert.subject_alt_names {
            println!("  {}", san);
        }
    }


    println!("Extensions:");
    for ext in &cert.extensions {
        println!(
            "  {} ({}) - {}",
            ext.name.as_deref().unwrap_or(&ext.oid),
            if ext.critical {
                "critical"
            } else {
                "non-critical"
            },
            ext.value
        );
    }
}

#[derive(Debug, Clone)]
pub enum ValidityStatus {
    Valid,
    ExpiringSoon, // within 30 days
    Expired,
}

impl ValidityStatus {
    fn from_dates(not_after: &str) -> Self {
        // Try parsing as YYYY-MM-DD HH:MM:SS format first
        if let Ok(expiry) = DateTime::parse_from_str(not_after, "%Y-%m-%d %H:%M:%S") {
            let expiry_utc = expiry.with_timezone(&Utc);
            let now = Utc::now();
            let days_until_expiry = (expiry_utc - now).num_days();

            if days_until_expiry < 0 {
                ValidityStatus::Expired
            } else if days_until_expiry <= 30 {
                ValidityStatus::ExpiringSoon
            } else {
                ValidityStatus::Valid
            }
        } else if let Ok(expiry) = DateTime::parse_from_rfc2822(not_after) {
            // Fallback to RFC 2822 format for backward compatibility
            let expiry_utc = expiry.with_timezone(&Utc);
            let now = Utc::now();
            let days_until_expiry = (expiry_utc - now).num_days();

            if days_until_expiry < 0 {
                ValidityStatus::Expired
            } else if days_until_expiry <= 30 {
                ValidityStatus::ExpiringSoon
            } else {
                ValidityStatus::Valid
            }
        } else {
            ValidityStatus::Valid // fallback if date parsing fails
        }
    }

    fn color(&self) -> Color {
        match self {
            ValidityStatus::Valid => Color::Green,
            ValidityStatus::ExpiringSoon => Color::Yellow,
            ValidityStatus::Expired => Color::Red,
        }
    }

    fn text(&self) -> &'static str {
        match self {
            ValidityStatus::Valid => "✓ Valid",
            ValidityStatus::ExpiringSoon => "⚠ Expiring Soon",
            ValidityStatus::Expired => "✗ Expired",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValidationStatus {
    Valid,
    InvalidChain,
}


impl ValidationStatus {
    fn text(&self) -> &'static str {
        match self {
            ValidationStatus::Valid => "✓ Valid Chain",
            ValidationStatus::InvalidChain => "✗ Invalid Chain",
        }
    }

    fn color(&self) -> Color {
        match self {
            ValidationStatus::Valid => Color::Green,
            ValidationStatus::InvalidChain => Color::Red,
        }
    }
}

fn display_tui(cert: &CertificateInfo) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let validity_status = ValidityStatus::from_dates(&cert.not_after);

    // Force initial clear and small delay to ensure proper layout on startup
    terminal.clear()?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(10),   // Certificate info
                    Constraint::Length(3), // Footer
                ])
                .split(size);

            // Title block
            let title = Paragraph::new("🔐 Certificate Inspector")
                .style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
                .block(Block::default().borders(Borders::ALL).title("cert-tree.rs"));
            f.render_widget(title, chunks[0]);

            // Certificate information
            let cn = extract_cn(&cert.subject);
            let sig_explanation = explain_signature_algorithm(&cert.signature_algorithm);
            let mut cert_info = vec![
                Line::from(vec![
                    Span::styled("CN: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cn, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Issuer: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.issuer, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Serial: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.serial_number, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Validity: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.not_before, Style::default().fg(Color::White)),
                    Span::raw(" → "),
                    Span::styled(&cert.not_after, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        validity_status.text(),
                        Style::default().fg(validity_status.color()),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Public Key: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        &cert.public_key_algorithm,
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Signature Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(sig_explanation.as_str(), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.version.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Is CA: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        cert.is_ca.to_string(),
                        Style::default().fg(if cert.is_ca {
                            Color::Yellow
                        } else {
                            Color::White
                        }),
                    ),
                ]),
            ];

            if let Some(ku) = &cert.key_usage {
                cert_info.push(Line::from(vec![
                    Span::styled("Key Usage: ", Style::default().fg(Color::Blue)),
                    Span::styled(ku, Style::default().fg(Color::Magenta)),
                ]));
            }

            if !cert.subject_alt_names.is_empty() {
                cert_info.push(Line::from(vec![
                    Span::styled("Subject Alt Names: ", Style::default().fg(Color::Blue)),
                    Span::styled(
                        cert.subject_alt_names.join(", "),
                        Style::default().fg(Color::Cyan),
                    ),
                ]));
            }

            let cert_paragraph = Paragraph::new(cert_info).wrap(Wrap { trim: true }).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Certificate Details"),
            );
            f.render_widget(cert_paragraph, chunks[1]);

            // Footer with instructions
            let footer = Paragraph::new("Press 'q' to quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn display_certificate_tree_text(tree: &CertificateTree) {
    let mut sequence_num = 0;
    for (i, root) in tree.roots.iter().enumerate() {
        let prefix = "━ ";
        display_tree_node_text(
            root,
            prefix,
            0,
            &mut sequence_num,
            i == tree.roots.len() - 1,
        );
    }
}

fn display_tree_node_text(
    node: &CertificateNode,
    prefix: &str,
    depth: usize,
    sequence_num: &mut usize,
    _is_last: bool,
) {
    // Increment sequence number for this certificate
    *sequence_num += 1;

    // Fixed column positions - dates should align regardless of tree depth
    let date_column_start: usize = 93; // Fixed position for date column (adjusted for seconds in time format)

    // Get certificate name (without sequence number) - use only CN
    let cn = extract_cn(&node.cert.subject);
    let available_name_space = date_column_start.saturating_sub(prefix.len()) - 5; // Leave space for brackets and content
    let display_name = if cn.len() > available_name_space {
        let truncate_len = if available_name_space > 3 {
            available_name_space - 3
        } else {
            available_name_space
        };
        format!("{}...", cn.chars().take(truncate_len).collect::<String>())
    } else {
        cn.clone()
    };

    // Date is already in the correct format
    let date_str = node.cert.not_after.clone();

    // Calculate exact padding to align date column
    let name_end_pos = prefix.len() + display_name.len();
    let padding_needed = if name_end_pos < date_column_start {
        date_column_start - name_end_pos
    } else {
        1 // minimum space
    };
    let padding = " ".repeat(padding_needed);

    // Color codes for terminal output
    let (status_text, color_code) = match node.validity_status {
        ValidityStatus::Expired => ("EXPIRED", "\x1b[31m"), // Red
        ValidityStatus::ExpiringSoon => ("EXPIRES SOON", "\x1b[33m"), // Yellow
        ValidityStatus::Valid => ("VALID", "\x1b[32m"),     // Green
    };

    // Use white for certificate names, color only the status/date part
    println!(
        "\x1b[37m{}{}{}\x1b[0m{}[{}] [{} until: {}]\x1b[0m",
        prefix, display_name, padding, color_code, sequence_num, status_text, date_str
    );

    // Display children with cascading tree structure
    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;

        // Create cascading indentation for child level (4 spaces per level)
        let child_indent = " ".repeat(5 + (depth * 4)); // 5 spaces base + 4 per depth level
        let child_prefix = format!("{}└ ", child_indent);

        display_tree_node_text(child, &child_prefix, depth + 1, sequence_num, is_last_child);
    }
}

fn display_certificate_tree_tui(tree: &CertificateTree) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Flatten the certificate tree into a list
    let certificates = flatten_certificate_tree(tree);
    let mut list_state = ratatui::widgets::ListState::default();
    list_state.select(Some(0));

    // Scroll state for certificate details pane
    let mut details_scroll: u16 = 0;

    // State to track if details pane is active for focused navigation
    // When active, arrow keys control details scrolling instead of list navigation
    // Toggle with Tab key for better accessibility and usability
    let mut details_pane_active = false;

    // Force initial clear and small delay to ensure proper layout on startup
    terminal.clear()?;
    std::thread::sleep(std::time::Duration::from_millis(50));

    loop {
        terminal.draw(|f| {
            let size = f.size();

            // Create main layout
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // Title
                    Constraint::Min(5),    // Certificate list
                    Constraint::Min(5),    // Certificate details
                    Constraint::Length(3), // Footer
                ])
                .split(size);

            // Title block with version
            let title_text = format!("🔐 Certificate Chain Inspector{:>width$}", env!("CARGO_PKG_VERSION"), width = size.width as usize - 35);
            let title = Paragraph::new(title_text)
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("cert-tree.rs"));
            f.render_widget(title, chunks[0]);

            // Calculate dynamic column widths based on terminal size
            let terminal_width = size.width as usize;
            let min_gap = 2; // Minimum gap between columns
            let min_name_width = 8; // Minimum width for certificate names

            // Adaptive date formatting based on terminal width
            let (date_format, date_width) = if terminal_width < 80 {
                ("%m-%d %H:%M", 11)
            } else if terminal_width < 100 {
                ("%Y-%m-%d %H:%M", 16)
            } else {
                ("%Y-%m-%d %H:%M:%S", 19)
            };

            let padding_after_date = 3;

            let list_area = chunks[1];
            let effective_width = (list_area.width as usize).saturating_sub(2); // Subtract border width (1 left + 1 right)
            let available_name_width = effective_width.saturating_sub(date_width + min_gap + padding_after_date).max(min_name_width);

            // Create list items
            let items: Vec<ListItem> = certificates
                .iter()
                .enumerate()
                .map(|(_i, item)| {
                    // Truncate long names if necessary
                    let display_name = if item.display_name.len() > available_name_width {
                        if available_name_width > 3 {
                            format!("{}...", item.display_name.chars().take(available_name_width-3).collect::<String>())
                        } else {
                            item.display_name.chars().take(available_name_width).collect::<String>()
                        }
                    } else {
                        item.display_name.clone()
                    };

                    // Reformat date using adaptive format
                    let formatted_date = if let Ok(dt) = DateTime::parse_from_str(&item.valid_until, "%Y-%m-%d %H:%M:%S") {
                        dt.format(date_format).to_string()
                    } else {
                        item.valid_until.clone()
                    };

                    // Create formatted strings for each column
                    let name_part = format!("{:<width$}", display_name, width = available_name_width);
                    let safe_date_width = date_width.max(formatted_date.len());
                    let date_part = format!("{:>width$}", formatted_date, width = safe_date_width);

                    let line = Line::from(vec![
                        Span::styled(name_part, Style::default().fg(Color::White)),
                        Span::styled(date_part, Style::default().fg(item.validity_status.color())),
                        Span::raw("   "), // Add 3 spaces padding after date
                    ]);

                    ListItem::new(line)
                })
                .collect();

            // Create the list widget with visual feedback for active state
            let list_title = if !details_pane_active {
                "Certificates (Active - Use ↑/↓/PgUp/PgDn to navigate)"
            } else {
                "Certificates (Press Tab to activate)"
            };

            let list_block = if !details_pane_active {
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
                    .border_style(Style::default().fg(Color::Yellow))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .title(list_title)
            };

            let list = List::new(items)
                .block(list_block)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, list_area, &mut list_state);

            // Certificate details section
            let selected_index = list_state.selected().unwrap_or(0);
            let selected_cert = &certificates[selected_index];
            let cert = &selected_cert.certificate_info;
            let sig_explanation = explain_signature_algorithm(&cert.signature_algorithm);

            let mut details_lines = vec![
                Line::from(vec![
                    Span::styled("Subject: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.subject, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Issuer: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.issuer, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Serial Number: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.serial_number, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Validity Period: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.not_before, Style::default().fg(Color::White)),
                    Span::raw(" → "),
                    Span::styled(&cert.not_after, Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Status: ", Style::default().fg(Color::Blue)),
                    Span::styled(selected_cert.validity_status.text(), Style::default().fg(selected_cert.validity_status.color())),
                ]),
                Line::from(vec![
                    Span::styled("Chain Validation: ", Style::default().fg(Color::Blue)),
                    Span::styled(selected_cert.validation_status.text(), Style::default().fg(selected_cert.validation_status.color())),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.version.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Public Key Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.public_key_algorithm, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Signature Algorithm: ", Style::default().fg(Color::Blue)),
                    Span::styled(sig_explanation.as_str(), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Is CA: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.is_ca.to_string(), Style::default().fg(if cert.is_ca { Color::Yellow } else { Color::White })),
                ]),
            ];

            if let Some(ku) = &cert.key_usage {
                details_lines.push(Line::from(vec![
                    Span::styled("Key Usage: ", Style::default().fg(Color::Blue)),
                    Span::styled(ku, Style::default().fg(Color::Magenta)),
                ]));
            }

            if !cert.subject_alt_names.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Subject Alternative Names: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.subject_alt_names.join(", "), Style::default().fg(Color::Cyan)),
                ]));
            }

            if !cert.extensions.is_empty() {
                details_lines.push(Line::from(vec![
                    Span::styled("Extensions:", Style::default().fg(Color::Blue)),
                ]));
                for ext in &cert.extensions {
                    let ext_name = ext.name.as_deref().unwrap_or(&ext.oid);
                    details_lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(ext_name, Style::default().fg(Color::Cyan)),
                        Span::raw(" ("),
                        Span::styled(if ext.critical { "critical" } else { "non-critical" }, Style::default().fg(if ext.critical { Color::Red } else { Color::Green })),
                        Span::raw(")"),
                    ]));
                }
            }

            // Create details paragraph with visual feedback for active state
            let details_title = if details_pane_active {
                "Certificate Details (Active - Use ↑/↓ to scroll)"
            } else {
                "Certificate Details (Press Tab to activate)"
            };

            let details_block = if details_pane_active {
                Block::default()
                    .borders(Borders::ALL)
                    .title(details_title)
                    .border_style(Style::default().fg(Color::Yellow))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .title(details_title)
            };

            let details_paragraph = Paragraph::new(details_lines)
                .wrap(Wrap { trim: true })
                .block(details_block)
                .scroll((details_scroll, 0));
            f.render_widget(details_paragraph, chunks[2]);

            // Footer with instructions - dynamic based on details pane state
            let footer_text = if details_pane_active {
                "Tab: Deactivate Details | ↑/↓: Scroll Details | PgUp/PgDn: Navigate List | 'q' Quit | 't' Text Mode"
            } else {
                "↑/↓/PgUp/PgDn: Navigate List | Tab: Activate Details | 'q' Quit | 't' Text Mode"
            };

            let footer = Paragraph::new(footer_text)
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        // Enhanced Navigation System:
        // - Tab: Toggle details pane activation/deactivation
        // - When details pane inactive: ↑/↓/PgUp/PgDn navigate certificate list
        // - When details pane active: ↑/↓ scroll certificate details, PgUp/PgDn disabled
        // - 'q'/Esc: Quit application
        // - 't': Switch to text mode
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,

                    // Tab key toggles details pane activation
                    KeyCode::Tab => {
                        details_pane_active = !details_pane_active;
                    }

                    // Navigation keys - behavior depends on details pane state
                    KeyCode::Up => {
                        if details_pane_active {
                            // Scroll details up when details pane is active
                            if details_scroll > 0 {
                                details_scroll = details_scroll.saturating_sub(1);
                            }
                        } else {
                            // Navigate list up when details pane is inactive
                            let i = list_state.selected().unwrap_or(0);
                            if i > 0 {
                                list_state.select(Some(i - 1));
                            }
                        }
                    }
                    KeyCode::Down => {
                        if details_pane_active {
                            // Scroll details down when details pane is active
                            if details_scroll < 50 {
                                // Arbitrary max scroll limit
                                details_scroll += 1;
                            }
                        } else {
                            // Navigate list down when details pane is inactive
                            let i = list_state.selected().unwrap_or(0);
                            if i < certificates.len() - 1 {
                                list_state.select(Some(i + 1));
                            }
                        }
                    }

                    // Page Up/Page Down for fast list navigation (only when details pane inactive)
                    KeyCode::PageUp => {
                        if !details_pane_active {
                            let i = list_state.selected().unwrap_or(0);
                            let page_size = 10; // Scroll by 10 items
                            let new_index = i.saturating_sub(page_size);
                            list_state.select(Some(new_index));
                        }
                    }
                    KeyCode::PageDown => {
                        if !details_pane_active {
                            let i = list_state.selected().unwrap_or(0);
                            let page_size = 10; // Scroll by 10 items
                            let new_index = (i + page_size).min(certificates.len() - 1);
                            list_state.select(Some(new_index));
                        }
                    }

                    // Text mode switch
                    KeyCode::Char('t') => {
                        // Switch to text mode
                        disable_raw_mode()?;
                        execute!(
                            terminal.backend_mut(),
                            LeaveAlternateScreen,
                            DisableMouseCapture
                        )?;
                        terminal.show_cursor()?;
                        display_certificate_tree_text(tree);
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // If no input arguments provided, show help
    if args.file.is_none() && args.url.is_none() {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }

    let certificates = if let Some(file) = &args.file {
        let data = load_certificate_from_file(file)?;
        parse_certificate_chain(&data)?
    } else if let Some(url) = &args.url {
        fetch_certificate_chain_from_url(url)?
    } else {
        eprintln!("Error: Must provide --file or --url");
        std::process::exit(1);
    };


    if certificates.len() == 1 {
        // Single certificate - use existing logic
        let cert_info = &certificates[0];

        if args.interactive {
            display_tui(cert_info)?;
        } else {
            display_verbose(cert_info);
        }
    } else {
        // Multiple certificates - build and display tree
        let tree = build_certificate_tree(certificates);

        if args.interactive {
            // TUI mode for certificate chains
            display_certificate_tree_tui(&tree)?;
        } else {
            // Default to text mode
            display_certificate_tree_text(&tree);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
                ExtensionInfo {
                    oid: "2.5.29.14".to_string(),
                    name: oid_to_name("2.5.29.14"),
                    critical: false,
                    value: "KeyIdentifier(...)".to_string(),
                },
                ExtensionInfo {
                    oid: "2.5.29.17".to_string(),
                    name: oid_to_name("2.5.29.17"),
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
        display_tree(&cert, "", true);
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
