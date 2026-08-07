//! Certificate input sources.
//!
//! Loads certificates from local PEM/DER files and fetches certificate
//! chains from remote HTTPS endpoints, returning parsed certificate
//! information ready for display.

use crate::error::CertError;
use crate::models::CertificateInfo;
use crate::parser::extract_cert_info;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use url::Url;
use x509_parser::prelude::{FromDer, X509Certificate};

/// Buffer size for reading certificate data from network
const BUFFER_SIZE: usize = 1024;

/// Standard HTTPS port number
const HTTPS_PORT: u16 = 443;

/// Connection timeout in seconds for network operations
const CONNECTION_TIMEOUT_SECS: u64 = 10;

pub fn load_certificate_from_file(path: &str) -> Result<Vec<u8>, CertError> {
    let path = Path::new(path);
    if !path.exists() {
        return Err(CertError::NotFound);
    }

    let data = fs::read(path)?;
    Ok(data)
}

pub fn fetch_certificate_chain_from_url(url: &str) -> Result<Vec<CertificateInfo>, CertError> {
    let url_parsed = parse_https_url(url)?;
    let hostname = url_parsed.host_str().ok_or(CertError::InvalidFormat)?;

    // First, try to fetch direct PEM or DER certificate data.
    let client = reqwest::blocking::Client::new();
    if let Ok(response) = client
        .get(url)
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
    {
        let data = response.bytes()?;
        if let Ok(certificates) = parse_downloaded_certificate(&data) {
            return Ok(certificates);
        }
    }

    // Otherwise, inspect the certificate chain served by the HTTPS endpoint.
    fetch_certificate_chain_via_tls(hostname)
}

fn parse_https_url(url: &str) -> Result<Url, CertError> {
    let parsed = Url::parse(url).map_err(|_| CertError::InvalidFormat)?;
    if parsed.scheme() != "https" {
        return Err(CertError::InvalidArguments(
            "--url only supports HTTPS URLs".to_string(),
        ));
    }
    Ok(parsed)
}

fn parse_downloaded_certificate(data: &[u8]) -> Result<Vec<CertificateInfo>, CertError> {
    crate::parser::parse_certificate_chain(data)
}

fn fetch_certificate_chain_via_tls(hostname: &str) -> Result<Vec<CertificateInfo>, CertError> {
    use rustls::client::ClientConnection;
    use rustls::{ClientConfig, RootCertStore};
    use webpki_roots::TLS_SERVER_ROOTS;

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
    let mut socket = TcpStream::connect((hostname, HTTPS_PORT))?;
    socket.set_read_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS)))?;
    socket.set_write_timeout(Some(Duration::from_secs(CONNECTION_TIMEOUT_SECS)))?;

    let server_name =
        rustls::ServerName::try_from(hostname).map_err(|_| CertError::InvalidFormat)?;

    let mut conn = ClientConnection::new(Arc::new(config), server_name)?;

    // Perform TLS handshake
    let mut tls_stream = rustls::Stream::new(&mut conn, &mut socket);

    // Send a minimal HTTP request to trigger the handshake
    let request = format!("GET / HTTP/1.0\r\nHost: {hostname}\r\n\r\n");
    tls_stream.write_all(request.as_bytes())?;

    // Read response to complete handshake
    let mut buffer = [0u8; BUFFER_SIZE];
    let _ = tls_stream.read(&mut buffer);

    // Extract certificate chain from the connection
    if let Some(certs) = conn.peer_certificates() {
        let mut certificates = Vec::new();
        for cert_der in certs {
            let (remaining, cert) = X509Certificate::from_der(cert_der.as_ref())
                .map_err(|e| CertError::X509Parse(format!("Failed to parse certificate: {e}")))?;
            if !remaining.is_empty() {
                return Err(CertError::X509Parse(format!(
                    "trailing data after TLS DER certificate ({} bytes)",
                    remaining.len()
                )));
            }

            let cert_info = extract_cert_info(&cert, cert_der.as_ref());
            certificates.push(cert_info);
        }
        Ok(certificates)
    } else {
        Err(CertError::X509Parse(
            "No certificates found in TLS handshake".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_downloaded_der_body_is_parsed_without_network() {
        let pem = pem::parse(include_bytes!("../test/single_cert.pem")).unwrap();
        let certificates = parse_downloaded_certificate(pem.contents()).unwrap();
        assert_eq!(certificates.len(), 1);
    }

    #[test]
    fn test_url_scheme_validation_accepts_only_https_without_network() {
        let parsed = parse_https_url("https://example.test/cert.pem").unwrap();
        assert_eq!(parsed.scheme(), "https");

        for unsupported in [
            "http://example.test/cert.pem",
            "ftp://example.test/cert.pem",
            "file:///tmp/cert.pem",
        ] {
            let error = parse_https_url(unsupported).unwrap_err();
            assert!(matches!(error, CertError::InvalidArguments(_)));
            assert!(error.to_string().contains("only supports HTTPS"));
        }
    }
}
