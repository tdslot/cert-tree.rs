//! Core certificate, validity, and chain-construction data models.

use chrono::{DateTime, Duration, Utc};
use std::fmt;

/// Parsed X.509 certificate metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CertificateInfo {
    /// Human-readable subject, used only for display.
    pub subject: String,
    /// Canonical encoded subject identity used for relationship matching.
    pub subject_der: Vec<u8>,
    /// Human-readable issuer, used only for display.
    pub issuer: String,
    /// Canonical encoded issuer identity used for relationship matching.
    pub issuer_der: Vec<u8>,
    pub serial_number: String,
    pub not_before: Option<DateTime<Utc>>,
    pub not_after: Option<DateTime<Utc>>,
    pub public_key_algorithm: String,
    pub signature_algorithm: String,
    pub version: u32,
    pub extensions: Vec<ExtensionInfo>,
    pub is_ca: bool,
    pub basic_constraints: Option<BasicConstraintsInfo>,
    pub key_usage: Vec<KeyUsage>,
    pub extended_key_usage: Vec<ExtendedKeyUsage>,
    pub subject_alt_names: Vec<SubjectAlternativeName>,
    pub subject_key_identifier: Option<String>,
    pub authority_key_identifier: Option<AuthorityKeyIdentifierInfo>,
    pub sha256_fingerprint: String,
    pub self_signature_status: SelfSignatureStatus,
}

/// Complete Authority Key Identifier extension fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityKeyIdentifierInfo {
    pub key_identifier: Option<String>,
    pub authority_cert_issuer: Vec<SubjectAlternativeName>,
    pub authority_cert_serial_number: Option<String>,
}

/// Result of checking the signature of a self-issued certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelfSignatureStatus {
    NotSelfIssued,
    Verified,
    Invalid,
    Unsupported,
}

/// Basic Constraints extension fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicConstraintsInfo {
    pub ca: bool,
    pub path_len_constraint: Option<u32>,
}

/// X.509 Key Usage values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
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

impl fmt::Display for KeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::DigitalSignature => "digital_signature",
            Self::NonRepudiation => "non_repudiation",
            Self::KeyEncipherment => "key_encipherment",
            Self::DataEncipherment => "data_encipherment",
            Self::KeyAgreement => "key_agreement",
            Self::KeyCertSign => "key_cert_sign",
            Self::CrlSign => "crl_sign",
            Self::EncipherOnly => "encipher_only",
            Self::DecipherOnly => "decipher_only",
        };
        f.write_str(label)
    }
}

/// X.509 Extended Key Usage values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtendedKeyUsage {
    Any,
    ServerAuth,
    ClientAuth,
    CodeSigning,
    EmailProtection,
    TimeStamping,
    OcspSigning,
    Other(String),
}

impl fmt::Display for ExtendedKeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Any => "any",
            Self::ServerAuth => "server_auth",
            Self::ClientAuth => "client_auth",
            Self::CodeSigning => "code_signing",
            Self::EmailProtection => "email_protection",
            Self::TimeStamping => "time_stamping",
            Self::OcspSigning => "ocsp_signing",
            Self::Other(oid) => return write!(f, "other ({oid})"),
        };
        f.write_str(label)
    }
}

/// Subject Alternative Name type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectAlternativeNameType {
    Dns,
    Ip,
    Email,
    Uri,
    DirectoryName,
    RegisteredId,
    Other,
}

/// One Subject Alternative Name entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubjectAlternativeName {
    pub san_type: SubjectAlternativeNameType,
    pub value: String,
    /// OID for SAN forms such as otherName or registeredID.
    pub oid: Option<String>,
    /// Raw DER content for unsupported or encoded SAN forms.
    pub raw_der_hex: Option<String>,
}

impl fmt::Display for SubjectAlternativeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.san_type, self.value)?;
        if let Some(oid) = &self.oid {
            write!(f, " ({oid})")?;
        }
        Ok(())
    }
}

/// Raw extension metadata retained for deterministic inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionInfo {
    pub oid: String,
    pub name: Option<String>,
    pub critical: bool,
    pub value_hex: String,
}

/// Certificate validity at one captured instant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidityStatus {
    NotYetValid,
    Valid,
    ExpiringSoon,
    Expired,
    Unknown,
}

impl ValidityStatus {
    /// Calculate validity using inclusive certificate boundaries.
    pub fn at(
        not_before: Option<DateTime<Utc>>,
        not_after: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Self {
        let (Some(not_before), Some(not_after)) = (not_before, not_after) else {
            return Self::Unknown;
        };
        if now < not_before {
            Self::NotYetValid
        } else if now > not_after {
            Self::Expired
        } else if not_after - now <= Duration::days(30) {
            Self::ExpiringSoon
        } else {
            Self::Valid
        }
    }
}

/// Result of linking a certificate to another input certificate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstructionStatus {
    Linked,
    MissingIssuer,
    AmbiguousIssuer,
    SelfSigned,
    SelfIssuedUnverified,
}

/// Per-certificate chain construction explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructionDiagnostic {
    pub certificate_index: usize,
    pub status: ConstructionStatus,
    pub selected_issuer_index: Option<usize>,
    pub issuer_candidate_indices: Vec<usize>,
    pub reason: Option<String>,
}

/// One node in a constructed certificate relationship tree.
#[derive(Debug, Clone)]
pub struct CertificateNode {
    pub certificate_index: usize,
    pub cert: CertificateInfo,
    pub children: Vec<CertificateNode>,
    pub validity_status: ValidityStatus,
    pub construction_status: ConstructionStatus,
}

/// Constructed certificate relationships plus diagnostics.
#[derive(Debug, Clone)]
pub struct CertificateTree {
    pub roots: Vec<CertificateNode>,
    pub diagnostics: Vec<ConstructionDiagnostic>,
}

/// Flattened display-only certificate entry.
#[derive(Debug, Clone)]
pub struct CertificateDisplayItem {
    pub display_name: String,
    pub valid_until: Option<DateTime<Utc>>,
    pub validity_status: ValidityStatus,
    pub construction_status: ConstructionStatus,
    pub certificate_info: CertificateInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn instant(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, 0, 0, 0).unwrap()
    }

    #[test]
    fn test_validity_all_states_and_boundaries() {
        let now = instant(10);
        assert_eq!(
            ValidityStatus::at(None, Some(instant(20)), now),
            ValidityStatus::Unknown
        );
        assert_eq!(
            ValidityStatus::at(Some(instant(11)), Some(instant(20)), now),
            ValidityStatus::NotYetValid
        );
        assert_eq!(
            ValidityStatus::at(Some(now), Some(instant(20)), now),
            ValidityStatus::ExpiringSoon
        );
        assert_eq!(
            ValidityStatus::at(Some(instant(1)), Some(now), now),
            ValidityStatus::ExpiringSoon
        );
        assert_eq!(
            ValidityStatus::at(Some(instant(1)), Some(now - Duration::seconds(1)), now),
            ValidityStatus::Expired
        );
        assert_eq!(
            ValidityStatus::at(Some(instant(1)), Some(now + Duration::days(30)), now),
            ValidityStatus::ExpiringSoon
        );
        assert_eq!(
            ValidityStatus::at(Some(instant(1)), Some(now + Duration::days(31)), now),
            ValidityStatus::Valid
        );
    }
}
