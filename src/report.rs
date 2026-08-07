//! Stable, explicit JSON inspection report schema.

use crate::error::CertError;
use crate::models::{
    BasicConstraintsInfo, CertificateInfo, CertificateTree, ConstructionDiagnostic,
    ConstructionStatus, ExtendedKeyUsage, ExtensionInfo, KeyUsage, SelfSignatureStatus,
    SubjectAlternativeName, SubjectAlternativeNameType, ValidityStatus,
};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Serialize;
use std::io::Write;
use url::Url;

/// Current machine-readable report schema version.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub enum InputKind {
    File,
    Url,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ReportInputKind {
    File,
    Url,
}

#[derive(Debug, Serialize)]
pub struct InputMetadata {
    kind: ReportInputKind,
    source: String,
}

#[derive(Debug, Serialize)]
pub struct InspectionReport {
    schema_version: u32,
    input: InputMetadata,
    certificates: Vec<CertificateReport>,
    construction: ConstructionReport,
}

#[derive(Debug, Serialize)]
struct CertificateReport {
    index: usize,
    sha256_fingerprint: String,
    subject: String,
    issuer: String,
    serial_number: String,
    validity: ValidityReport,
    public_key_algorithm: String,
    signature_algorithm: String,
    version: u32,
    is_ca: bool,
    basic_constraints: Option<BasicConstraintsReport>,
    key_usage: Vec<KeyUsageReport>,
    extended_key_usage: Vec<ExtendedKeyUsageReport>,
    subject_alt_names: Vec<SubjectAlternativeNameReport>,
    subject_key_identifier: Option<String>,
    authority_key_identifier: Option<AuthorityKeyIdentifierReport>,
    self_signature: SelfSignatureStatusReport,
    extensions: Vec<ExtensionReport>,
}

#[derive(Debug, Serialize)]
struct BasicConstraintsReport {
    ca: bool,
    path_len_constraint: Option<u32>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum KeyUsageReport {
    DigitalSignature,
    NonRepudiation,
    KeyEncipherment,
    DataEncipherment,
    KeyAgreement,
    KeyCertSign,
    CrlSign,
    EncipherOnly,
    DecipherOnly,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExtendedKeyUsageNameReport {
    Any,
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OcspSigning,
    Other,
}

#[derive(Debug, Serialize)]
struct ExtendedKeyUsageReport {
    name: ExtendedKeyUsageNameReport,
    oid: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum SubjectAlternativeNameTypeReport {
    Dns,
    Ip,
    Email,
    Uri,
    DirectoryName,
    RegisteredId,
    Other,
}

#[derive(Debug, Serialize)]
struct SubjectAlternativeNameReport {
    #[serde(rename = "type")]
    san_type: SubjectAlternativeNameTypeReport,
    value: String,
    oid: Option<String>,
    raw_der_hex: Option<String>,
}

#[derive(Debug, Serialize)]
struct AuthorityKeyIdentifierReport {
    key_identifier: Option<String>,
    authority_cert_issuer: Vec<SubjectAlternativeNameReport>,
    authority_cert_serial_number: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum SelfSignatureStatusReport {
    NotSelfIssued,
    Verified,
    Invalid,
    Unsupported,
}

#[derive(Debug, Serialize)]
struct ExtensionReport {
    oid: String,
    name: Option<String>,
    critical: bool,
    value_hex: String,
}

#[derive(Debug, Serialize)]
struct ValidityReport {
    not_before: Option<String>,
    not_after: Option<String>,
    status: ValidityStatusReport,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ValidityStatusReport {
    NotYetValid,
    Valid,
    ExpiringSoon,
    Expired,
    Unknown,
}

#[derive(Debug, Serialize)]
struct ConstructionReport {
    roots: Vec<usize>,
    diagnostics: Vec<ConstructionDiagnosticReport>,
}

#[derive(Debug, Serialize)]
struct ConstructionDiagnosticReport {
    certificate_index: usize,
    status: ConstructionStatusReport,
    selected_issuer_index: Option<usize>,
    issuer_candidate_indices: Vec<usize>,
    reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum ConstructionStatusReport {
    Linked,
    MissingIssuer,
    AmbiguousIssuer,
    SelfSigned,
    SelfIssuedUnverified,
}

impl InspectionReport {
    pub fn new(
        kind: InputKind,
        source: &str,
        certificates: &[CertificateInfo],
        tree: &CertificateTree,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            input: InputMetadata {
                kind: map_input_kind(kind),
                source: sanitize_source(kind, source),
            },
            certificates: certificates
                .iter()
                .enumerate()
                .map(|(index, cert)| certificate_report(index, cert, now))
                .collect(),
            construction: ConstructionReport {
                roots: tree
                    .roots
                    .iter()
                    .map(|root| root.certificate_index)
                    .collect(),
                diagnostics: tree.diagnostics.iter().map(diagnostic_report).collect(),
            },
        }
    }
}

fn certificate_report(
    index: usize,
    cert: &CertificateInfo,
    now: DateTime<Utc>,
) -> CertificateReport {
    CertificateReport {
        index,
        sha256_fingerprint: cert.sha256_fingerprint.clone(),
        subject: cert.subject.clone(),
        issuer: cert.issuer.clone(),
        serial_number: cert.serial_number.clone(),
        validity: ValidityReport {
            not_before: rfc3339(cert.not_before),
            not_after: rfc3339(cert.not_after),
            status: map_validity(ValidityStatus::at(cert.not_before, cert.not_after, now)),
        },
        public_key_algorithm: cert.public_key_algorithm.clone(),
        signature_algorithm: cert.signature_algorithm.clone(),
        version: cert.version,
        is_ca: cert.is_ca,
        basic_constraints: cert.basic_constraints.as_ref().map(map_basic_constraints),
        key_usage: cert.key_usage.iter().copied().map(map_key_usage).collect(),
        extended_key_usage: cert
            .extended_key_usage
            .iter()
            .map(map_extended_key_usage)
            .collect(),
        subject_alt_names: cert
            .subject_alt_names
            .iter()
            .map(map_subject_alt_name)
            .collect(),
        subject_key_identifier: cert.subject_key_identifier.clone(),
        authority_key_identifier: cert.authority_key_identifier.as_ref().map(|aki| {
            AuthorityKeyIdentifierReport {
                key_identifier: aki.key_identifier.clone(),
                authority_cert_issuer: aki
                    .authority_cert_issuer
                    .iter()
                    .map(map_subject_alt_name)
                    .collect(),
                authority_cert_serial_number: aki.authority_cert_serial_number.clone(),
            }
        }),
        self_signature: map_self_signature(cert.self_signature_status),
        extensions: cert.extensions.iter().map(map_extension).collect(),
    }
}

fn diagnostic_report(diagnostic: &ConstructionDiagnostic) -> ConstructionDiagnosticReport {
    ConstructionDiagnosticReport {
        certificate_index: diagnostic.certificate_index,
        status: map_construction(diagnostic.status),
        selected_issuer_index: diagnostic.selected_issuer_index,
        issuer_candidate_indices: diagnostic.issuer_candidate_indices.clone(),
        reason: diagnostic.reason.clone(),
    }
}

fn map_input_kind(kind: InputKind) -> ReportInputKind {
    match kind {
        InputKind::File => ReportInputKind::File,
        InputKind::Url => ReportInputKind::Url,
    }
}

fn map_basic_constraints(value: &BasicConstraintsInfo) -> BasicConstraintsReport {
    BasicConstraintsReport {
        ca: value.ca,
        path_len_constraint: value.path_len_constraint,
    }
}

fn map_key_usage(value: KeyUsage) -> KeyUsageReport {
    match value {
        KeyUsage::DigitalSignature => KeyUsageReport::DigitalSignature,
        KeyUsage::NonRepudiation => KeyUsageReport::NonRepudiation,
        KeyUsage::KeyEncipherment => KeyUsageReport::KeyEncipherment,
        KeyUsage::DataEncipherment => KeyUsageReport::DataEncipherment,
        KeyUsage::KeyAgreement => KeyUsageReport::KeyAgreement,
        KeyUsage::KeyCertSign => KeyUsageReport::KeyCertSign,
        KeyUsage::CrlSign => KeyUsageReport::CrlSign,
        KeyUsage::EncipherOnly => KeyUsageReport::EncipherOnly,
        KeyUsage::DecipherOnly => KeyUsageReport::DecipherOnly,
    }
}

fn map_extended_key_usage(value: &ExtendedKeyUsage) -> ExtendedKeyUsageReport {
    let (name, oid) = match value {
        ExtendedKeyUsage::Any => (ExtendedKeyUsageNameReport::Any, None),
        ExtendedKeyUsage::ServerAuth => (ExtendedKeyUsageNameReport::ServerAuth, None),
        ExtendedKeyUsage::ClientAuth => (ExtendedKeyUsageNameReport::ClientAuth, None),
        ExtendedKeyUsage::CodeSigning => (ExtendedKeyUsageNameReport::CodeSigning, None),
        ExtendedKeyUsage::EmailProtection => (ExtendedKeyUsageNameReport::EmailProtection, None),
        ExtendedKeyUsage::TimeStamping => (ExtendedKeyUsageNameReport::TimeStamping, None),
        ExtendedKeyUsage::OcspSigning => (ExtendedKeyUsageNameReport::OcspSigning, None),
        ExtendedKeyUsage::Other(oid) => (ExtendedKeyUsageNameReport::Other, Some(oid.clone())),
    };
    ExtendedKeyUsageReport { name, oid }
}

fn map_subject_alt_name(value: &SubjectAlternativeName) -> SubjectAlternativeNameReport {
    let san_type = match value.san_type {
        SubjectAlternativeNameType::Dns => SubjectAlternativeNameTypeReport::Dns,
        SubjectAlternativeNameType::Ip => SubjectAlternativeNameTypeReport::Ip,
        SubjectAlternativeNameType::Email => SubjectAlternativeNameTypeReport::Email,
        SubjectAlternativeNameType::Uri => SubjectAlternativeNameTypeReport::Uri,
        SubjectAlternativeNameType::DirectoryName => {
            SubjectAlternativeNameTypeReport::DirectoryName
        }
        SubjectAlternativeNameType::RegisteredId => SubjectAlternativeNameTypeReport::RegisteredId,
        SubjectAlternativeNameType::Other => SubjectAlternativeNameTypeReport::Other,
    };
    SubjectAlternativeNameReport {
        san_type,
        value: value.value.clone(),
        oid: value.oid.clone(),
        raw_der_hex: value.raw_der_hex.clone(),
    }
}

fn map_self_signature(value: SelfSignatureStatus) -> SelfSignatureStatusReport {
    match value {
        SelfSignatureStatus::NotSelfIssued => SelfSignatureStatusReport::NotSelfIssued,
        SelfSignatureStatus::Verified => SelfSignatureStatusReport::Verified,
        SelfSignatureStatus::Invalid => SelfSignatureStatusReport::Invalid,
        SelfSignatureStatus::Unsupported => SelfSignatureStatusReport::Unsupported,
    }
}

fn map_extension(value: &ExtensionInfo) -> ExtensionReport {
    ExtensionReport {
        oid: value.oid.clone(),
        name: value.name.clone(),
        critical: value.critical,
        value_hex: value.value_hex.clone(),
    }
}

fn map_validity(value: ValidityStatus) -> ValidityStatusReport {
    match value {
        ValidityStatus::NotYetValid => ValidityStatusReport::NotYetValid,
        ValidityStatus::Valid => ValidityStatusReport::Valid,
        ValidityStatus::ExpiringSoon => ValidityStatusReport::ExpiringSoon,
        ValidityStatus::Expired => ValidityStatusReport::Expired,
        ValidityStatus::Unknown => ValidityStatusReport::Unknown,
    }
}

fn map_construction(value: ConstructionStatus) -> ConstructionStatusReport {
    match value {
        ConstructionStatus::Linked => ConstructionStatusReport::Linked,
        ConstructionStatus::MissingIssuer => ConstructionStatusReport::MissingIssuer,
        ConstructionStatus::AmbiguousIssuer => ConstructionStatusReport::AmbiguousIssuer,
        ConstructionStatus::SelfSigned => ConstructionStatusReport::SelfSigned,
        ConstructionStatus::SelfIssuedUnverified => ConstructionStatusReport::SelfIssuedUnverified,
    }
}

fn rfc3339(date: Option<DateTime<Utc>>) -> Option<String> {
    date.map(|date| date.to_rfc3339_opts(SecondsFormat::Secs, true))
}

fn sanitize_source(kind: InputKind, source: &str) -> String {
    if !matches!(kind, InputKind::Url) {
        return source.to_string();
    }
    let Ok(mut url) = Url::parse(source) else {
        return source.to_string();
    };
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

/// Write pretty JSON followed by one newline.
pub fn write_json<W: Write>(writer: &mut W, report: &InspectionReport) -> Result<(), CertError> {
    serde_json::to_writer_pretty(&mut *writer, report)?;
    writeln!(writer)?;
    Ok(())
}

/// Return true when opt-in inspection policy finds a problem.
pub fn policy_failed(tree: &CertificateTree) -> bool {
    fn node_failed(node: &crate::models::CertificateNode) -> bool {
        !matches!(node.validity_status, ValidityStatus::Valid)
            || node.children.iter().any(node_failed)
    }
    tree.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic.status,
            ConstructionStatus::MissingIssuer
                | ConstructionStatus::AmbiguousIssuer
                | ConstructionStatus::SelfIssuedUnverified
        )
    }) || tree.roots.iter().any(node_failed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_certificate_chain;
    use crate::tree::build_certificate_tree_at;
    use chrono::TimeZone;
    use serde_json::json;

    fn report_value(source: &str) -> serde_json::Value {
        let mut certs =
            parse_certificate_chain(include_bytes!("../test/leaf_without_issuer.pem")).unwrap();
        certs[0]
            .extended_key_usage
            .push(ExtendedKeyUsage::Other("1.2.3.4".to_string()));
        certs[0].basic_constraints = None;
        let now = Utc.with_ymd_and_hms(2026, 8, 6, 0, 0, 0).unwrap();
        let tree = build_certificate_tree_at(&certs, now);
        serde_json::to_value(InspectionReport::new(
            InputKind::Url,
            source,
            &certs,
            &tree,
            now,
        ))
        .unwrap()
    }

    #[test]
    fn test_json_v1_exact_key_sets_and_unknown_eku() {
        let value = report_value("https://user:secret@example.test/cert.pem?token=abc#fragment");
        assert_eq!(
            value
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec!["certificates", "construction", "input", "schema_version"]
        );
        let certificate = &value["certificates"][0];
        assert_eq!(
            certificate
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>(),
            vec![
                "authority_key_identifier",
                "basic_constraints",
                "extended_key_usage",
                "extensions",
                "index",
                "is_ca",
                "issuer",
                "key_usage",
                "public_key_algorithm",
                "self_signature",
                "serial_number",
                "sha256_fingerprint",
                "signature_algorithm",
                "subject",
                "subject_alt_names",
                "subject_key_identifier",
                "validity",
                "version",
            ]
        );
        assert!(certificate["basic_constraints"].is_null());
        assert!(certificate["extended_key_usage"]
            .as_array()
            .unwrap()
            .contains(&json!({"name": "other", "oid": "1.2.3.4"})));
        assert_eq!(
            value["input"],
            json!({"kind": "url", "source": "https://example.test/cert.pem"})
        );
        assert_eq!(certificate["self_signature"], json!("not_self_issued"));
        assert_eq!(
            value["construction"]["diagnostics"][0],
            json!({
                "certificate_index": 0,
                "status": "missing_issuer",
                "selected_issuer_index": null,
                "issuer_candidate_indices": [],
                "reason": "no_issuer_candidate"
            })
        );
    }

    #[test]
    fn test_json_unknown_dates_are_explicit_null() {
        let mut certs =
            parse_certificate_chain(include_bytes!("../test/leaf_without_issuer.pem")).unwrap();
        certs[0].not_before = None;
        certs[0].not_after = None;
        let now = Utc.with_ymd_and_hms(2026, 8, 6, 0, 0, 0).unwrap();
        let tree = build_certificate_tree_at(&certs, now);
        let value = serde_json::to_value(InspectionReport::new(
            InputKind::File,
            "leaf.pem",
            &certs,
            &tree,
            now,
        ))
        .unwrap();
        assert!(value["certificates"][0]["validity"]["not_before"].is_null());
        assert_eq!(value["certificates"][0]["validity"]["status"], "unknown");
    }

    #[test]
    fn test_policy_failed_flags_every_non_valid_status() {
        use crate::models::CertificateInfo;
        use chrono::Duration;

        let certs = parse_certificate_chain(include_bytes!("../test/single_cert.pem")).unwrap();
        let base: CertificateInfo = certs[0].clone();
        let now = Utc.with_ymd_and_hms(2026, 8, 6, 0, 0, 0).unwrap();

        // Valid certificate passes the policy.
        let mut valid = base.clone();
        valid.not_before = Some(now - Duration::days(1));
        valid.not_after = Some(now + Duration::days(365));
        let tree = build_certificate_tree_at(&[valid], now);
        assert!(!policy_failed(&tree));

        // Every non-valid status fails the policy.
        for (before, after, label) in [
            (
                Some(now + Duration::days(1)),
                Some(now + Duration::days(365)),
                "not_yet_valid",
            ),
            (
                Some(now - Duration::days(365)),
                Some(now - Duration::days(1)),
                "expired",
            ),
            (
                Some(now - Duration::days(1)),
                Some(now + Duration::days(30)),
                "expiring_soon",
            ),
            (None, None, "unknown"),
        ] {
            let mut cert = base.clone();
            cert.not_before = before;
            cert.not_after = after;
            let tree = build_certificate_tree_at(&[cert], now);
            assert!(policy_failed(&tree), "{label} should fail the policy");
        }
    }

    #[test]
    fn test_json_writer_has_no_ansi_and_ends_with_newline() {
        let certs =
            parse_certificate_chain(include_bytes!("../test/leaf_without_issuer.pem")).unwrap();
        let now = Utc.with_ymd_and_hms(2026, 8, 6, 0, 0, 0).unwrap();
        let tree = build_certificate_tree_at(&certs, now);
        let report = InspectionReport::new(InputKind::File, "leaf.pem", &certs, &tree, now);
        let mut bytes = Vec::new();
        write_json(&mut bytes, &report).unwrap();
        assert!(bytes.ends_with(b"\n"));
        assert!(!String::from_utf8(bytes).unwrap().contains("\u{1b}["));
    }
}
