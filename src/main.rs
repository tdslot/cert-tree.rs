use clap::{Parser, CommandFactory};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;
use x509_parser::prelude::*;
use chrono::{DateTime, Utc};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use std::collections::HashMap;
use rustls::{ClientConfig, RootCertStore};
use webpki_roots::TLS_SERVER_ROOTS;
use std::sync::Arc;

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

    /// Certificate data as string (PEM or DER)
    #[arg(short, long)]
    pub data: Option<String>,

    /// Output format: tree (default), json, verbose, tui
    #[arg(short, long, default_value = "tree")]
    pub format: String,

    /// Output file (optional)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Text output mode (non-interactive)
    #[arg(short = 't', long)]
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
    use std::io::{Read, Write};
    use rustls::client::ClientConnection;

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

    let server_name = rustls::ServerName::try_from(hostname)
        .map_err(|_| CertError::InvalidFormat)?;

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
                    return Err(CertError::X509Parse(format!("Failed to parse certificate: {}", e)));
                }
            }
        }
        Ok(certificates)
    } else {
        Err(CertError::X509Parse("No certificates found in TLS handshake".to_string()))
    }
}

pub fn parse_certificate(data: &[u8]) -> Result<CertificateInfo, CertError> {
    // Try PEM first
    if let Ok((_, pem)) = pem::parse_x509_pem(data) {
        let cert = pem.parse_x509().map_err(|e| CertError::X509Parse(e.to_string()))?;
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
        issuer_map.entry(cert.issuer.clone())
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

    CertificateTree { roots }
}

fn flatten_certificate_tree(tree: &CertificateTree) -> Vec<(String, String, ValidityStatus)> {
    let mut certificates = Vec::new();
    for root in &tree.roots {
        flatten_node(root, &mut certificates, 0);
    }
    certificates
}

fn flatten_node(node: &CertificateNode, certificates: &mut Vec<(String, String, ValidityStatus)>, depth: usize) {
    // Format certificate name with indentation
    let indent = "  ".repeat(depth);
    let name = format!("{}{}", indent, node.cert.subject);

    // Format validity date
    let valid_until = if let Ok(expiry) = DateTime::parse_from_rfc2822(&node.cert.not_after) {
        expiry.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid Date".to_string()
    };

    certificates.push((name, valid_until, node.validity_status.clone()));

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
                    let child_node = build_tree_node(child_cert, cert_map, issuer_map, processed, index + i + 1);
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
    }
}

fn extract_cert_info(cert: &X509Certificate) -> Result<CertificateInfo, CertError> {
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = format!("{:x}", cert.serial).as_bytes().chunks(2).map(|chunk| {
        std::str::from_utf8(chunk).unwrap_or("??")
    }).collect::<Vec<_>>().join(" ");
    let not_before = cert.validity().not_before.to_rfc2822().unwrap_or_else(|_| "Invalid date".to_string());
    let not_after = cert.validity().not_after.to_rfc2822().unwrap_or_else(|_| "Invalid date".to_string());

    let public_key_alg = match cert.public_key().parsed() {
        Ok(pk) => {
            match pk {
                x509_parser::public_key::PublicKey::RSA(rsa_key) => {
                    let key_size = rsa_key.modulus.len() * 8;
                    format!("RSA ({} bits)", key_size)
                }
                x509_parser::public_key::PublicKey::EC(_) => "ECDSA".to_string(),
                x509_parser::public_key::PublicKey::DSA(_) => "DSA".to_string(),
                x509_parser::public_key::PublicKey::GostR3410(_) => "GOST R 34.10".to_string(),
                x509_parser::public_key::PublicKey::GostR3410_2012(_) => "GOST R 34.10-2012".to_string(),
                _ => "Unknown".to_string(),
            }
        }
        Err(_) => "Unknown".to_string(),
    };

    let signature_alg = format!("{:?}", cert.signature_algorithm.algorithm);

    let mut extensions = Vec::new();
    let key_usage = None;
    let subject_alt_names = Vec::new();

    for ext in cert.extensions() {
        let oid_str = ext.oid.to_string();
        let _name = ext.oid.to_id_string();
        let critical = ext.critical;
        let value = format!("{:?}", ext.value);

        extensions.push(ExtensionInfo {
            oid: oid_str.clone(),
            name: None,
            critical,
            value,
        });

        // Extract specific extensions
        // TODO: Fix parsed_extension API
        // if let Some(parsed_ext) = ext.parsed_extension() {
        //     match parsed_ext {
        //         x509_parser::extensions::ParsedExtension::KeyUsage(ku) => {
        //             key_usage = Some(format!("{:?}", ku));
        //         }
        //         x509_parser::extensions::ParsedExtension::SubjectAlternativeName(san) => {
        //             for name in &san.general_names {
        //                 subject_alt_names.push(format!("{:?}", name));
        //             }
        //         }
        //         _ => {}
        //     }
        // }
    }

    let is_ca = cert.is_ca();

    Ok(CertificateInfo {
        subject,
        issuer,
        serial_number: serial,
        not_before,
        not_after,
        public_key_algorithm: public_key_alg,
        signature_algorithm: signature_alg,
        version: cert.version.0,
        extensions,
        is_ca,
        key_usage,
        subject_alt_names,
    })
}

pub fn display_tree(cert: &CertificateInfo, prefix: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, connector, cert.subject);

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    println!("{}{}Issuer: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.issuer);
    println!("{}{}Serial: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.serial_number);
    println!("{}{}Valid: {} to {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.not_before, cert.not_after);
    println!("{}{}Public Key: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.public_key_algorithm);
    println!("{}{}Signature: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.signature_algorithm);
    println!("{}{}Version: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.version);
    println!("{}{}Is CA: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, cert.is_ca);

    if let Some(ku) = &cert.key_usage {
        println!("{}{}Key Usage: {}", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " }, ku);
    }

    if !cert.subject_alt_names.is_empty() {
        println!("{}{}Subject Alt Names:", new_prefix, if cert.extensions.is_empty() { "└── " } else { "├── " });
        for (i, san) in cert.subject_alt_names.iter().enumerate() {
            let san_connector = if i == cert.subject_alt_names.len() - 1 && cert.extensions.is_empty() { "└── " } else { "├── " };
            println!("{}{}{}{}", new_prefix, if cert.extensions.is_empty() { "    " } else { "│   " }, san_connector, san);
        }
    }

    if !cert.extensions.is_empty() {
        println!("{}{}Extensions:", new_prefix, "└── ");
        for (i, ext) in cert.extensions.iter().enumerate() {
            let ext_connector = if i == cert.extensions.len() - 1 { "└── " } else { "├── " };
            let ext_prefix = format!("{}{}", new_prefix, "    ");
            let ext_name = ext.name.as_deref().unwrap_or(&ext.oid);
            println!("{}{}{} ({})", ext_prefix, ext_connector, ext_name, if ext.critical { "critical" } else { "non-critical" });
        }
    }
}

pub fn display_verbose(cert: &CertificateInfo) {
    println!("Certificate Information:");
    println!("======================");
    println!("Subject: {}", cert.subject);
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
        println!("  {} ({}) - {}", ext.name.as_deref().unwrap_or(&ext.oid), if ext.critical { "critical" } else { "non-critical" }, ext.value);
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
        if let Ok(expiry) = DateTime::parse_from_rfc2822(not_after) {
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
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .block(Block::default().borders(Borders::ALL).title("cert-tree.rs"));
            f.render_widget(title, chunks[0]);

            // Certificate information
            let mut cert_info = vec![
                Line::from(vec![
                    Span::styled("Subject: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.subject, Style::default().fg(Color::White)),
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
                    Span::styled(validity_status.text(), Style::default().fg(validity_status.color())),
                ]),
                Line::from(vec![
                    Span::styled("Public Key: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.public_key_algorithm, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Signature: ", Style::default().fg(Color::Blue)),
                    Span::styled(&cert.signature_algorithm, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("Version: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.version.to_string(), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("Is CA: ", Style::default().fg(Color::Blue)),
                    Span::styled(cert.is_ca.to_string(), Style::default().fg(if cert.is_ca { Color::Yellow } else { Color::White })),
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
                    Span::styled(cert.subject_alt_names.join(", "), Style::default().fg(Color::Cyan)),
                ]));
            }

            let cert_paragraph = Paragraph::new(cert_info)
                .block(Block::default().borders(Borders::ALL).title("Certificate Details"));
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
        display_tree_node_text(root, prefix, 0, &mut sequence_num, i == tree.roots.len() - 1);
    }
}

fn display_tree_node_text(node: &CertificateNode, prefix: &str, depth: usize, sequence_num: &mut usize, _is_last: bool) {
    // Increment sequence number for this certificate
    *sequence_num += 1;

    // Fixed column positions - dates should align regardless of tree depth
    let date_column_start: usize = 93; // Fixed position for date column (adjusted for seconds in time format)

    // Get certificate name (without sequence number)
    let available_name_space = date_column_start.saturating_sub(prefix.len()) - 5; // Leave space for brackets and content
    let display_name = if node.cert.subject.len() > available_name_space {
        let truncate_len = if available_name_space > 3 { available_name_space - 3 } else { available_name_space };
        format!("{}...", node.cert.subject.chars().take(truncate_len).collect::<String>())
    } else {
        node.cert.subject.clone()
    };

    // Format validity date with time
    let date_str = if let Ok(expiry) = DateTime::parse_from_rfc2822(&node.cert.not_after) {
        expiry.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Invalid".to_string()
    };

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
        ValidityStatus::Valid => ("VALID", "\x1b[32m"), // Green
    };

    // Print the line with sequence number and status/date in separate square brackets
    println!("{}{}{}{}[{}] [{} until: {}]\x1b[0m", prefix, display_name, padding, color_code, sequence_num, status_text, date_str);

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
                    Constraint::Min(10),   // Certificate list
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

            // Choose date format based on available space - prioritize showing seconds
            let (date_width, date_format) = if terminal_width >= 17 + min_gap + min_name_width {
                (19, "%Y-%m-%d %H:%M:%S") // Full format with seconds (try to fit even if slightly tight)
            } else if terminal_width >= 14 + min_gap + min_name_width {
                (16, "%Y-%m-%d %H:%M") // Shorter format without seconds
            } else if terminal_width >= 8 + min_gap + min_name_width {
                (10, "%Y-%m-%d") // Date only for narrow terminals
            } else {
                // Extremely narrow terminal - use minimal format
                (8, "%Y-%m-%d") // Short date for very narrow terminals
            };

            let available_name_width = terminal_width.saturating_sub(date_width + min_gap).max(min_name_width);

            // Create list items
            let items: Vec<ListItem> = certificates
                .iter()
                .enumerate()
                .map(|(_i, (name, valid_until, status))| {
                    // Truncate long names if necessary
                    let display_name = if name.len() > available_name_width {
                        if available_name_width > 3 {
                            format!("{}...", name.chars().take(available_name_width-3).collect::<String>())
                        } else {
                            name.chars().take(available_name_width).collect::<String>()
                        }
                    } else {
                        name.clone()
                    };

                    // Format date according to available space
                    let formatted_date = if let Ok(expiry) = DateTime::parse_from_rfc2822(valid_until) {
                        expiry.format(date_format).to_string()
                    } else {
                        valid_until.clone()
                    };

                    // Create formatted strings for each column
                    let name_part = format!("{:<width$}", display_name, width = available_name_width);
                    let safe_date_width = date_width.max(formatted_date.len());
                    let date_part = format!("{:>width$}", formatted_date, width = safe_date_width.saturating_sub(8));

                    let line = Line::from(vec![
                        Span::styled(name_part, Style::default().fg(Color::White)),
                        Span::styled(date_part, Style::default().fg(status.color())),
                    ]);

                    ListItem::new(line)
                })
                .collect();

            // Create the list widget with scrolling
            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Certificates"))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol(">> ");

            // Render list with scrolling state
            let list_area = chunks[1];
            f.render_stateful_widget(list, list_area, &mut list_state);

            // Footer with instructions
            let footer = Paragraph::new("↑/↓ Navigate | 'q' Quit | 't' Text Mode")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up => {
                        let i = list_state.selected().unwrap_or(0);
                        if i > 0 {
                            list_state.select(Some(i - 1));
                        }
                    }
                    KeyCode::Down => {
                        let i = list_state.selected().unwrap_or(0);
                        if i < certificates.len() - 1 {
                            list_state.select(Some(i + 1));
                        }
                    }
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
    if args.file.is_none() && args.url.is_none() && args.data.is_none() {
        Args::command().print_help().unwrap();
        std::process::exit(0);
    }

    let certificates = if let Some(file) = &args.file {
        let data = load_certificate_from_file(file)?;
        parse_certificate_chain(&data)?
    } else if let Some(url) = &args.url {
        fetch_certificate_chain_from_url(url)?
    } else if let Some(data_str) = &args.data {
        let data = data_str.as_bytes().to_vec();
        parse_certificate_chain(&data)?
    } else {
        eprintln!("Error: Must provide --file, --url, or --data");
        std::process::exit(1);
    };

    if certificates.len() == 1 {
        // Single certificate - use existing logic
        let cert_info = &certificates[0];

        match args.format.as_str() {
            "tree" => {
                display_tree(cert_info, "", true);
            }
            "json" => {
                let json = serde_json::to_string_pretty(cert_info)?;
                if let Some(output_file) = &args.output {
                    fs::write(output_file, &json)?;
                } else {
                    println!("{}", json);
                }
            }
            "verbose" => {
                display_verbose(cert_info);
            }
            "tui" => {
                display_tui(cert_info)?;
            }
            _ => {
                eprintln!("Error: Invalid format. Use 'tree', 'json', 'verbose', or 'tui'");
                std::process::exit(1);
            }
        }
    } else {
        // Multiple certificates - build and display tree
        let tree = build_certificate_tree(certificates);

        if args.text {
            // Text mode for certificate chains
            display_certificate_tree_text(&tree);
        } else {
            // TUI mode for certificate chains (default)
            display_certificate_tree_tui(&tree)?;
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
            extensions: vec![],
            is_ca: false,
            key_usage: Some("Digital Signature".to_string()),
            subject_alt_names: vec!["example.com".to_string()],
        };

        // This will print to stdout, but we can't easily test output
        // In a real scenario, we'd capture stdout or use a different approach
        display_tree(&cert, "", true);
    }

    #[test]
    fn test_certificate_info_serialization() {
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

        let json = serde_json::to_string(&cert).unwrap();
        let deserialized: CertificateInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(cert.subject, deserialized.subject);
        assert_eq!(cert.is_ca, deserialized.is_ca);
    }
}
