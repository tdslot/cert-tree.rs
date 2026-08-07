//! X.509 certificate parsing and structured metadata extraction.

use crate::error::CertError;
use crate::models::{
    AuthorityKeyIdentifierInfo, BasicConstraintsInfo, CertificateInfo, ExtendedKeyUsage,
    ExtensionInfo, KeyUsage, SelfSignatureStatus, SubjectAlternativeName,
    SubjectAlternativeNameType,
};
use chrono::{TimeZone, Utc};
use pem::parse_many;
use sha2::{Digest, Sha256};
use std::net::{Ipv4Addr, Ipv6Addr};
use x509_parser::error::X509Error;
use x509_parser::extensions::{GeneralName, ParsedExtension};
use x509_parser::prelude::{FromDer, X509Certificate};

/// Extract the common name from a distinguished name for compact display.
pub fn extract_cn(subject: &str) -> String {
    subject
        .split(',')
        .map(str::trim)
        .find_map(|part| part.strip_prefix("CN="))
        .unwrap_or(subject)
        .to_string()
}

/// Map well-known extension OIDs to display names.
pub fn oid_to_name(oid: &str) -> Option<String> {
    let name = match oid {
        "2.5.29.14" => "Subject Key Identifier",
        "2.5.29.15" => "Key Usage",
        "2.5.29.16" => "Private Key Usage Period",
        "2.5.29.17" => "Subject Alternative Name",
        "2.5.29.18" => "Issuer Alternative Name",
        "2.5.29.19" => "Basic Constraints",
        "2.5.29.30" => "Name Constraints",
        "2.5.29.31" => "CRL Distribution Points",
        "2.5.29.32" => "Certificate Policies",
        "2.5.29.33" => "Policy Mappings",
        "2.5.29.35" => "Authority Key Identifier",
        "2.5.29.36" => "Policy Constraints",
        "2.5.29.37" => "Extended Key Usage",
        "2.5.29.46" => "Freshest CRL",
        "1.3.6.1.5.5.7.1.1" => "Authority Information Access",
        "1.3.6.1.4.1.311.20.2" => "Microsoft Smart Card Login",
        "1.3.6.1.4.1.311.21.1" => "Microsoft Individual Code Signing",
        "1.2.840.113533.7.65.0" => "Entrust Version Information",
        "2.16.840.1.113730.1.1" => "Netscape Certificate Type",
        "2.23.42.7.0" => "VeriSign Individual SHA1 Hash",
        "1.3.6.1.4.1.11129.2.4.2" => "Signed Certificate Timestamp",
        _ => return None,
    };
    Some(name.to_string())
}

/// Map signature algorithm OIDs to concise names.
pub fn signature_alg_to_name(oid: &str) -> Option<String> {
    let name = match oid {
        "1.2.840.113549.1.1.4" => "RSA with MD5",
        "1.2.840.113549.1.1.5" | "1.3.14.3.2.29" => "SHA1 with RSA",
        "1.2.840.113549.1.1.11" => "SHA256 with RSA",
        "1.2.840.113549.1.1.12" => "SHA384 with RSA",
        "1.2.840.113549.1.1.13" => "SHA512 with RSA",
        "1.2.840.10045.4.1" => "SHA1 with ECDSA",
        "1.2.840.10045.4.3.2" => "SHA256 with ECDSA",
        "1.2.840.10045.4.3.3" => "SHA384 with ECDSA",
        "1.2.840.10045.4.3.4" => "SHA512 with ECDSA",
        "1.2.840.10040.4.3" => "SHA1 with DSA",
        _ => return None,
    };
    Some(name.to_string())
}

/// Explain a signature algorithm for the interactive display.
pub fn explain_signature_algorithm(algorithm: &str) -> String {
    format!("Cryptographic certificate signature: {algorithm}")
}

/// Parse one or more PEM certificates, or exactly one DER certificate.
pub fn parse_certificate_chain(data: &[u8]) -> Result<Vec<CertificateInfo>, CertError> {
    let mut certificates = Vec::new();
    if let Ok(pems) = parse_many(data) {
        for pem in pems.into_iter().filter(|pem| pem.tag() == "CERTIFICATE") {
            certificates.push(parse_single_der(pem.contents())?);
        }
    }
    if certificates.is_empty() {
        certificates.push(parse_single_der(data)?);
    }
    Ok(certificates)
}

fn parse_single_der(der: &[u8]) -> Result<CertificateInfo, CertError> {
    let (remaining, cert) =
        X509Certificate::from_der(der).map_err(|error| CertError::X509Parse(error.to_string()))?;
    if !remaining.is_empty() {
        return Err(CertError::X509Parse(format!(
            "trailing data after DER certificate ({} bytes)",
            remaining.len()
        )));
    }
    Ok(extract_cert_info(&cert, der))
}

/// Extract stable structured metadata from a parsed certificate and its original DER.
pub fn extract_cert_info(cert: &X509Certificate<'_>, der: &[u8]) -> CertificateInfo {
    let mut basic_constraints = None;
    let mut key_usage = Vec::new();
    let mut extended_key_usage = Vec::new();
    let mut subject_alt_names = Vec::new();
    let mut subject_key_identifier = None;
    let mut authority_key_identifier = None;
    let extensions = cert
        .extensions()
        .iter()
        .map(|extension| {
            match extension.parsed_extension() {
                ParsedExtension::BasicConstraints(constraints) => {
                    basic_constraints = Some(BasicConstraintsInfo {
                        ca: constraints.ca,
                        path_len_constraint: constraints.path_len_constraint,
                    });
                }
                ParsedExtension::KeyUsage(usage) => map_key_usage(*usage, &mut key_usage),
                ParsedExtension::ExtendedKeyUsage(usage) => {
                    extended_key_usage = map_extended_key_usage(usage);
                }
                ParsedExtension::SubjectAlternativeName(names) => {
                    subject_alt_names = names.general_names.iter().map(map_general_name).collect();
                }
                ParsedExtension::SubjectKeyIdentifier(identifier) => {
                    subject_key_identifier = Some(to_hex(identifier.0));
                }
                ParsedExtension::AuthorityKeyIdentifier(identifier) => {
                    authority_key_identifier = Some(AuthorityKeyIdentifierInfo {
                        key_identifier: identifier
                            .key_identifier
                            .as_ref()
                            .map(|key_identifier| to_hex(key_identifier.0)),
                        authority_cert_issuer: identifier
                            .authority_cert_issuer
                            .as_deref()
                            .unwrap_or_default()
                            .iter()
                            .map(map_general_name)
                            .collect(),
                        authority_cert_serial_number: identifier.authority_cert_serial.map(to_hex),
                    });
                }
                _ => {}
            }
            let oid = extension.oid.to_string();
            ExtensionInfo {
                name: oid_to_name(&oid),
                oid,
                critical: extension.critical,
                value_hex: to_hex(extension.value),
            }
        })
        .collect();

    let subject = cert.subject().to_string();
    let issuer = cert.issuer().to_string();
    let self_signature_status = if cert.subject().as_raw() == cert.issuer().as_raw() {
        match cert.verify_signature(None) {
            Ok(()) => SelfSignatureStatus::Verified,
            Err(X509Error::SignatureUnsupportedAlgorithm) => SelfSignatureStatus::Unsupported,
            Err(_) => SelfSignatureStatus::Invalid,
        }
    } else {
        SelfSignatureStatus::NotSelfIssued
    };
    CertificateInfo {
        subject,
        subject_der: cert.subject().as_raw().to_vec(),
        issuer,
        issuer_der: cert.issuer().as_raw().to_vec(),
        serial_number: cert.raw_serial_as_string().to_lowercase(),
        not_before: Utc
            .timestamp_opt(cert.validity().not_before.timestamp(), 0)
            .single(),
        not_after: Utc
            .timestamp_opt(cert.validity().not_after.timestamp(), 0)
            .single(),
        public_key_algorithm: public_key_algorithm(cert),
        signature_algorithm: signature_alg_to_name(&cert.signature_algorithm.algorithm.to_string())
            .unwrap_or_else(|| cert.signature_algorithm.algorithm.to_string()),
        version: cert.version.0 + 1,
        extensions,
        is_ca: cert.is_ca(),
        basic_constraints,
        key_usage,
        extended_key_usage,
        subject_alt_names,
        subject_key_identifier,
        authority_key_identifier,
        sha256_fingerprint: to_hex(&Sha256::digest(der)),
        self_signature_status,
    }
}

fn public_key_algorithm(cert: &X509Certificate<'_>) -> String {
    match cert.public_key().parsed() {
        Ok(x509_parser::public_key::PublicKey::RSA(key)) => {
            format!("RSA ({} bits)", key.key_size())
        }
        Ok(x509_parser::public_key::PublicKey::EC(_)) => "ECDSA".to_string(),
        Ok(x509_parser::public_key::PublicKey::DSA(_)) => "DSA".to_string(),
        Ok(x509_parser::public_key::PublicKey::GostR3410(_)) => "GOST R 34.10".to_string(),
        Ok(x509_parser::public_key::PublicKey::GostR3410_2012(_)) => {
            "GOST R 34.10-2012".to_string()
        }
        _ => "Unknown".to_string(),
    }
}

fn map_key_usage(usage: x509_parser::extensions::KeyUsage, output: &mut Vec<KeyUsage>) {
    let candidates = [
        (usage.digital_signature(), KeyUsage::DigitalSignature),
        (usage.non_repudiation(), KeyUsage::NonRepudiation),
        (usage.key_encipherment(), KeyUsage::KeyEncipherment),
        (usage.data_encipherment(), KeyUsage::DataEncipherment),
        (usage.key_agreement(), KeyUsage::KeyAgreement),
        (usage.key_cert_sign(), KeyUsage::KeyCertSign),
        (usage.crl_sign(), KeyUsage::CrlSign),
        (usage.encipher_only(), KeyUsage::EncipherOnly),
        (usage.decipher_only(), KeyUsage::DecipherOnly),
    ];
    output.extend(
        candidates
            .into_iter()
            .filter_map(|(set, value)| set.then_some(value)),
    );
}

fn map_extended_key_usage(
    usage: &x509_parser::extensions::ExtendedKeyUsage<'_>,
) -> Vec<ExtendedKeyUsage> {
    let mut output = Vec::new();
    let known = [
        (usage.any, ExtendedKeyUsage::Any),
        (usage.server_auth, ExtendedKeyUsage::ServerAuth),
        (usage.client_auth, ExtendedKeyUsage::ClientAuth),
        (usage.code_signing, ExtendedKeyUsage::CodeSigning),
        (usage.email_protection, ExtendedKeyUsage::EmailProtection),
        (usage.time_stamping, ExtendedKeyUsage::TimeStamping),
        (usage.ocsp_signing, ExtendedKeyUsage::OcspSigning),
    ];
    output.extend(
        known
            .into_iter()
            .filter_map(|(set, value)| set.then_some(value)),
    );
    output.extend(
        usage
            .other
            .iter()
            .map(|oid| ExtendedKeyUsage::Other(oid.to_string())),
    );
    output
}

fn map_general_name(name: &GeneralName<'_>) -> SubjectAlternativeName {
    match name {
        GeneralName::DNSName(value) => san(SubjectAlternativeNameType::Dns, value),
        GeneralName::IPAddress(bytes) if bytes.len() == 4 => san(
            SubjectAlternativeNameType::Ip,
            &Ipv4Addr::new(bytes[0], bytes[1], bytes[2], bytes[3]).to_string(),
        ),
        GeneralName::IPAddress(bytes) if bytes.len() == 16 => {
            let mut octets = [0_u8; 16];
            octets.copy_from_slice(bytes);
            san(
                SubjectAlternativeNameType::Ip,
                &Ipv6Addr::from(octets).to_string(),
            )
        }
        GeneralName::IPAddress(bytes) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::Other,
            value: "invalid_ip_address".to_string(),
            oid: None,
            raw_der_hex: Some(to_hex(bytes)),
        },
        GeneralName::RFC822Name(value) => san(SubjectAlternativeNameType::Email, value),
        GeneralName::URI(value) => san(SubjectAlternativeNameType::Uri, value),
        GeneralName::DirectoryName(value) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::DirectoryName,
            value: value.to_string(),
            oid: None,
            raw_der_hex: Some(to_hex(value.as_raw())),
        },
        GeneralName::RegisteredID(value) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::RegisteredId,
            value: value.to_string(),
            oid: Some(value.to_string()),
            raw_der_hex: None,
        },
        GeneralName::OtherName(oid, raw) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::Other,
            value: "other_name".to_string(),
            oid: Some(oid.to_string()),
            raw_der_hex: Some(to_hex(raw)),
        },
        GeneralName::X400Address(any) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::Other,
            value: "x400_address".to_string(),
            oid: None,
            raw_der_hex: Some(to_hex(any.data)),
        },
        GeneralName::EDIPartyName(any) => SubjectAlternativeName {
            san_type: SubjectAlternativeNameType::Other,
            value: "edi_party_name".to_string(),
            oid: None,
            raw_der_hex: Some(to_hex(any.data)),
        },
    }
}

fn san(san_type: SubjectAlternativeNameType, value: &str) -> SubjectAlternativeName {
    SubjectAlternativeName {
        san_type,
        value: value.to_string(),
        oid: None,
        raw_der_hex: None,
    }
}

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_and_ca_bundle() {
        let single = parse_certificate_chain(include_bytes!("../test/single_cert.pem")).unwrap();
        assert_eq!(single.len(), 1);
        assert_eq!(single[0].sha256_fingerprint.len(), 64);
        assert!(single[0].not_before.is_some());
        assert!(single[0].not_after.is_some());
        assert!(single[0].is_ca);
        assert!(single[0]
            .basic_constraints
            .as_ref()
            .is_some_and(|value| value.ca));
        assert!(single[0].subject_key_identifier.is_some());
        assert!(single[0].authority_key_identifier.is_some());
        assert_eq!(
            single[0].self_signature_status,
            SelfSignatureStatus::Verified
        );

        let bundle = parse_certificate_chain(include_bytes!("../test/cacert.pem")).unwrap();
        assert_eq!(bundle.len(), 146);
        assert!(bundle.iter().all(|cert| {
            cert.sha256_fingerprint.len() == 64
                && cert
                    .sha256_fingerprint
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
        }));
    }

    #[test]
    fn test_structured_leaf_fields() {
        let certs =
            parse_certificate_chain(include_bytes!("../test/leaf_without_issuer.pem")).unwrap();
        let cert = &certs[0];
        assert!(cert
            .subject_alt_names
            .iter()
            .any(|san| san.value == "example.test"));
        assert!(cert
            .subject_alt_names
            .iter()
            .any(|san| san.value == "192.0.2.10"));
        assert!(cert.key_usage.contains(&KeyUsage::DigitalSignature));
        assert!(cert
            .extended_key_usage
            .contains(&ExtendedKeyUsage::ServerAuth));
        assert!(cert.subject_key_identifier.is_some());
        assert!(cert.authority_key_identifier.is_some());
    }

    #[test]
    fn test_authority_extension_oids_are_distinct() {
        assert_eq!(
            oid_to_name("2.5.29.35").as_deref(),
            Some("Authority Key Identifier")
        );
        assert_eq!(
            oid_to_name("1.3.6.1.5.5.7.1.1").as_deref(),
            Some("Authority Information Access")
        );
    }

    #[test]
    fn test_malformed_input_is_error() {
        assert!(parse_certificate_chain(b"not a certificate").is_err());
    }

    #[test]
    fn test_der_with_trailing_data_is_rejected() {
        let pem = pem::parse(include_bytes!("../test/single_cert.pem")).unwrap();
        let mut der = pem.into_contents();
        der.extend_from_slice(b"trailing");
        let error = parse_certificate_chain(&der).unwrap_err().to_string();
        assert!(error.contains("trailing data"));
    }
}
