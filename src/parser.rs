use crate::error::CertError;
use crate::models::{CertificateInfo, ExtensionInfo};
use pem::parse_many;
use std::str;
use x509_parser::prelude::FromDer;
use x509_parser::prelude::X509Certificate;

pub fn extract_cn(subject: &str) -> String {
    // Parse the DN format: C=US, ST=New Jersey, L=Jersey City, O=The USERTRUST Network, CN=USERTrust RSA Cer...
    let parts: Vec<&str> = subject.split(',').collect();

    for part in parts {
        let trimmed = part.trim();
        if let Some(stripped) = trimmed.strip_prefix("CN=") {
            return stripped.to_string(); // Remove "CN=" prefix
        }
    }

    // If no CN found, return the whole subject as fallback
    subject.to_string()
}

// Function to map OID to human-readable extension name
pub fn oid_to_name(oid: &str) -> Option<String> {
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
pub fn signature_alg_to_name(oid_str: &str) -> Option<String> {
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
pub fn explain_signature_algorithm(alg: &str) -> String {
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

pub fn parse_certificate_chain(data: &[u8]) -> Result<Vec<CertificateInfo>, CertError> {
    let mut certificates = Vec::new();

    // Try to parse as PEM with multiple certificates
    if let Ok(pems) = parse_many(data) {
        for pem in pems {
            if pem.tag() == "CERTIFICATE" {
                let (_, cert) = X509Certificate::from_der(pem.contents())
                    .map_err(|e| CertError::X509Parse(e.to_string()))?;
                let cert_info = extract_cert_info(&cert)?;
                certificates.push(cert_info);
            }
        }
    }

    // If no PEM certificates found, try single DER
    if certificates.is_empty() {
        let (_, cert) =
            X509Certificate::from_der(data).map_err(|e| CertError::X509Parse(e.to_string()))?;
        let cert_info = extract_cert_info(&cert)?;
        certificates.push(cert_info);
    }

    Ok(certificates)
}

pub fn extract_cert_info(cert: &X509Certificate) -> Result<CertificateInfo, CertError> {
    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let serial = format!("{:x}", cert.serial)
        .as_bytes()
        .chunks(2)
        .map(|chunk| str::from_utf8(chunk).unwrap_or("??"))
        .collect::<Vec<_>>()
        .join(" ");
    // Store dates in RFC 2822 format initially, then convert to display format
    let not_before_rfc = cert
        .validity()
        .not_before
        .to_rfc2822()
        .unwrap_or_else(|_| "Invalid date".to_string());
    let not_after_rfc = cert
        .validity()
        .not_after
        .to_rfc2822()
        .unwrap_or_else(|_| "Invalid date".to_string());

    // Convert to display format
    let not_before = if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&not_before_rfc) {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        not_before_rfc
    };
    let not_after = if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&not_after_rfc) {
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
