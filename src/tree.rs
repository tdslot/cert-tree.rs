//! Deterministic certificate relationship construction.
//!
//! This module links certificates by issuer/subject names and AKI/SKI identifiers.
//! It does not perform PKIX path validation or establish trust.

use crate::models::{
    CertificateInfo, CertificateNode, CertificateTree, ConstructionDiagnostic, ConstructionStatus,
    SelfSignatureStatus, SubjectAlternativeNameType, ValidityStatus,
};
use chrono::{DateTime, Utc};
/// Deterministically build certificate relationships at `now`.
pub fn build_certificate_tree_at(
    certificates: &[CertificateInfo],
    now: DateTime<Utc>,
) -> CertificateTree {
    let mut diagnostics: Vec<ConstructionDiagnostic> = certificates
        .iter()
        .enumerate()
        .map(|(index, cert)| diagnose(index, cert, certificates))
        .collect();
    let mut parents: Vec<Option<usize>> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.selected_issuer_index)
        .collect();

    break_cycles(&mut parents, &mut diagnostics);

    let mut children = vec![Vec::new(); certificates.len()];
    for (child, parent) in parents.iter().enumerate() {
        if let Some(parent) = parent {
            children[*parent].push(child);
        }
    }

    let roots = parents
        .iter()
        .enumerate()
        .filter_map(|(index, parent)| parent.is_none().then_some(index))
        .map(|index| build_node(index, certificates, &children, &diagnostics, now))
        .collect();

    CertificateTree { roots, diagnostics }
}

fn diagnose(
    index: usize,
    certificate: &CertificateInfo,
    certificates: &[CertificateInfo],
) -> ConstructionDiagnostic {
    match certificate.self_signature_status {
        SelfSignatureStatus::Verified => {
            return diagnostic(
                index,
                ConstructionStatus::SelfSigned,
                None,
                Vec::new(),
                "self_signature_verified",
            );
        }
        SelfSignatureStatus::Invalid => {
            return diagnostic(
                index,
                ConstructionStatus::SelfIssuedUnverified,
                None,
                Vec::new(),
                "self_signature_invalid",
            );
        }
        SelfSignatureStatus::Unsupported => {
            return diagnostic(
                index,
                ConstructionStatus::SelfIssuedUnverified,
                None,
                Vec::new(),
                "self_signature_unsupported",
            );
        }
        SelfSignatureStatus::NotSelfIssued => {}
    }

    let dn_candidates: Vec<usize> = certificates
        .iter()
        .enumerate()
        .filter_map(|(candidate_index, candidate)| {
            (candidate_index != index && candidate.subject_der == certificate.issuer_der)
                .then_some(candidate_index)
        })
        .collect();
    if dn_candidates.is_empty() {
        return diagnostic(
            index,
            ConstructionStatus::MissingIssuer,
            None,
            dn_candidates,
            "no_issuer_candidate",
        );
    }

    let candidates = match certificate.authority_key_identifier.as_ref() {
        Some(aki) if authority_key_identifier_has_fields(aki) => {
            let matching: Vec<usize> = dn_candidates
                .iter()
                .copied()
                .filter(|candidate| {
                    authority_key_identifier_matches(aki, &certificates[*candidate])
                })
                .collect();
            if matching.is_empty() {
                return diagnostic(
                    index,
                    ConstructionStatus::MissingIssuer,
                    None,
                    dn_candidates,
                    "authority_key_identifier_mismatch",
                );
            }
            matching
        }
        _ => dn_candidates.clone(),
    };

    if candidates.len() == 1 {
        diagnostic(
            index,
            ConstructionStatus::Linked,
            Some(candidates[0]),
            dn_candidates,
            "issuer_selected",
        )
    } else {
        diagnostic(
            index,
            ConstructionStatus::AmbiguousIssuer,
            None,
            candidates,
            "multiple_issuer_candidates",
        )
    }
}

fn authority_key_identifier_has_fields(aki: &crate::models::AuthorityKeyIdentifierInfo) -> bool {
    aki.key_identifier.is_some()
        || aki.authority_cert_serial_number.is_some()
        || !aki.authority_cert_issuer.is_empty()
}

fn authority_key_identifier_matches(
    aki: &crate::models::AuthorityKeyIdentifierInfo,
    candidate: &CertificateInfo,
) -> bool {
    let key_identifier_matches = aki
        .key_identifier
        .as_ref()
        .is_none_or(|identifier| candidate.subject_key_identifier.as_ref() == Some(identifier));
    let serial_matches = aki
        .authority_cert_serial_number
        .as_ref()
        .is_none_or(|serial| {
            normalize_serial(serial) == normalize_serial(&candidate.serial_number)
        });
    let issuer_der_hex = crate::parser::to_hex(&candidate.issuer_der);
    let issuer_matches = aki.authority_cert_issuer.is_empty()
        || aki.authority_cert_issuer.iter().any(|name| {
            name.san_type == SubjectAlternativeNameType::DirectoryName
                && name.raw_der_hex.as_deref() == Some(issuer_der_hex.as_str())
        });
    key_identifier_matches && serial_matches && issuer_matches
}

fn normalize_serial(serial: &str) -> String {
    serial
        .chars()
        .filter(char::is_ascii_hexdigit)
        .flat_map(char::to_lowercase)
        .collect()
}

fn diagnostic(
    certificate_index: usize,
    status: ConstructionStatus,
    selected_issuer_index: Option<usize>,
    issuer_candidate_indices: Vec<usize>,
    reason: &str,
) -> ConstructionDiagnostic {
    ConstructionDiagnostic {
        certificate_index,
        status,
        selected_issuer_index,
        issuer_candidate_indices,
        reason: Some(reason.to_string()),
    }
}

fn break_cycles(parents: &mut [Option<usize>], diagnostics: &mut [ConstructionDiagnostic]) {
    loop {
        let mut cycle_member = None;
        for start in 0..parents.len() {
            let mut path = Vec::new();
            let mut current = Some(start);
            while let Some(index) = current {
                if let Some(position) = path.iter().position(|visited| *visited == index) {
                    cycle_member = path[position..].iter().min().copied();
                    break;
                }
                path.push(index);
                current = parents[index];
            }
            if cycle_member.is_some() {
                break;
            }
        }
        let Some(index) = cycle_member else { break };
        parents[index] = None;
        diagnostics[index].status = ConstructionStatus::AmbiguousIssuer;
        diagnostics[index].selected_issuer_index = None;
        diagnostics[index].reason = Some("cycle_detected".to_string());
    }
}

fn build_node(
    index: usize,
    certificates: &[CertificateInfo],
    children: &[Vec<usize>],
    diagnostics: &[ConstructionDiagnostic],
    now: DateTime<Utc>,
) -> CertificateNode {
    CertificateNode {
        certificate_index: index,
        cert: certificates[index].clone(),
        children: children[index]
            .iter()
            .map(|child| build_node(*child, certificates, children, diagnostics, now))
            .collect(),
        validity_status: ValidityStatus::at(
            certificates[index].not_before,
            certificates[index].not_after,
            now,
        ),
        construction_status: diagnostics[index].status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AuthorityKeyIdentifierInfo, BasicConstraintsInfo, SelfSignatureStatus,
        SubjectAlternativeName,
    };
    use chrono::TimeZone;

    fn cert(subject: &str, issuer: &str, ski: Option<&str>, aki: Option<&str>) -> CertificateInfo {
        CertificateInfo {
            subject: subject.to_string(),
            subject_der: subject.as_bytes().to_vec(),
            issuer: issuer.to_string(),
            issuer_der: issuer.as_bytes().to_vec(),
            serial_number: "01".to_string(),
            not_before: None,
            not_after: None,
            public_key_algorithm: "RSA".to_string(),
            signature_algorithm: "SHA256 with RSA".to_string(),
            version: 3,
            extensions: vec![],
            is_ca: false,
            basic_constraints: Some(BasicConstraintsInfo {
                ca: false,
                path_len_constraint: None,
            }),
            key_usage: vec![],
            extended_key_usage: vec![],
            subject_alt_names: vec![],
            subject_key_identifier: ski.map(str::to_string),
            authority_key_identifier: aki.map(|key_identifier| AuthorityKeyIdentifierInfo {
                key_identifier: Some(key_identifier.to_string()),
                authority_cert_issuer: vec![],
                authority_cert_serial_number: None,
            }),
            sha256_fingerprint: "00".repeat(32),
            self_signature_status: SelfSignatureStatus::NotSelfIssued,
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn test_self_signed_and_missing_issuer() {
        let mut root = cert("CN=root", "CN=root", Some("aa"), Some("aa"));
        root.self_signature_status = SelfSignatureStatus::Verified;
        let missing = cert("CN=leaf", "CN=missing", None, None);
        let tree = build_certificate_tree_at(&[root, missing], now());
        assert_eq!(tree.roots.len(), 2);
        assert_eq!(tree.diagnostics[0].status, ConstructionStatus::SelfSigned);
        assert_eq!(
            tree.diagnostics[1].status,
            ConstructionStatus::MissingIssuer
        );
    }

    #[test]
    fn test_self_issued_without_verified_signature_is_not_self_signed() {
        let mut certificate = cert("CN=root", "CN=root", None, None);
        certificate.self_signature_status = SelfSignatureStatus::Unsupported;
        let tree = build_certificate_tree_at(&[certificate], now());
        assert_eq!(
            tree.diagnostics[0].status,
            ConstructionStatus::SelfIssuedUnverified
        );
        assert_eq!(
            tree.diagnostics[0].reason.as_deref(),
            Some("self_signature_unsupported")
        );
    }

    #[test]
    fn test_unique_issuer_is_linked() {
        let certificates = [
            cert("CN=issuer", "CN=missing", None, None),
            cert("CN=leaf", "CN=issuer", None, None),
        ];
        let tree = build_certificate_tree_at(&certificates, now());
        assert_eq!(tree.diagnostics[1].status, ConstructionStatus::Linked);
        assert_eq!(tree.diagnostics[1].selected_issuer_index, Some(0));
    }

    #[test]
    fn test_duplicate_subject_preserved_and_aki_selects_candidate() {
        let issuer_a = cert("CN=issuer", "CN=other", Some("aa"), None);
        let issuer_b = cert("CN=issuer", "CN=other", Some("bb"), None);
        let leaf = cert("CN=leaf", "CN=issuer", None, Some("bb"));
        let tree = build_certificate_tree_at(&[issuer_a, issuer_b, leaf], now());
        assert_eq!(tree.diagnostics.len(), 3);
        assert_eq!(tree.diagnostics[2].selected_issuer_index, Some(1));
    }

    #[test]
    fn test_duplicate_issuer_without_identifiers_is_ambiguous() {
        let certs = [
            cert("CN=issuer", "CN=none", None, None),
            cert("CN=issuer", "CN=none", None, None),
            cert("CN=leaf", "CN=issuer", None, None),
        ];
        let tree = build_certificate_tree_at(&certs, now());
        assert_eq!(
            tree.diagnostics[2].status,
            ConstructionStatus::AmbiguousIssuer
        );
        assert_eq!(tree.roots.len(), 3);
    }

    #[test]
    fn test_aki_mismatch_is_missing() {
        let certs = [
            cert("CN=issuer", "CN=none", Some("aa"), None),
            cert("CN=leaf", "CN=issuer", None, Some("bb")),
        ];
        let tree = build_certificate_tree_at(&certs, now());
        assert_eq!(
            tree.diagnostics[1].status,
            ConstructionStatus::MissingIssuer
        );
    }

    #[test]
    fn test_display_name_equality_does_not_override_der_identity() {
        let mut issuer = cert("CN=issuer", "CN=other", None, None);
        issuer.subject_der = b"encoded-subject-a".to_vec();
        let mut leaf = cert("CN=leaf", "CN=issuer", None, None);
        leaf.issuer_der = b"encoded-subject-b".to_vec();
        let tree = build_certificate_tree_at(&[issuer, leaf], now());
        assert_eq!(
            tree.diagnostics[1].status,
            ConstructionStatus::MissingIssuer
        );
    }

    #[test]
    fn test_all_present_aki_fields_must_match_together() {
        let mut wrong = cert("CN=issuer", "CN=root-wrong", Some("aa"), None);
        wrong.subject_der = b"issuer-der".to_vec();
        wrong.issuer_der = b"root-wrong-der".to_vec();
        wrong.serial_number = "01".to_string();
        let mut correct = cert("CN=issuer", "CN=root-correct", Some("aa"), None);
        correct.subject_der = b"issuer-der".to_vec();
        correct.issuer_der = b"root-correct-der".to_vec();
        correct.serial_number = "02".to_string();
        let mut leaf = cert("CN=leaf", "CN=issuer", None, None);
        leaf.issuer_der = b"issuer-der".to_vec();
        leaf.authority_key_identifier = Some(AuthorityKeyIdentifierInfo {
            key_identifier: Some("aa".to_string()),
            authority_cert_issuer: vec![SubjectAlternativeName {
                san_type: SubjectAlternativeNameType::DirectoryName,
                value: "CN=root-correct".to_string(),
                oid: None,
                raw_der_hex: Some(crate::parser::to_hex(b"root-correct-der")),
            }],
            authority_cert_serial_number: Some("02".to_string()),
        });

        let tree = build_certificate_tree_at(&[wrong, correct, leaf], now());
        assert_eq!(tree.diagnostics[2].selected_issuer_index, Some(1));
        assert_eq!(tree.diagnostics[2].status, ConstructionStatus::Linked);
    }

    #[test]
    fn test_aki_issuer_and_serial_disambiguate_without_key_identifier() {
        let mut issuer_a = cert("CN=issuer", "CN=root-a", None, None);
        issuer_a.subject_der = b"issuer-der".to_vec();
        issuer_a.issuer_der = b"root-a-der".to_vec();
        issuer_a.serial_number = "01".to_string();
        let mut issuer_b = cert("CN=issuer", "CN=root-b", None, None);
        issuer_b.subject_der = b"issuer-der".to_vec();
        issuer_b.issuer_der = b"root-b-der".to_vec();
        issuer_b.serial_number = "02".to_string();
        let mut leaf = cert("CN=leaf", "CN=issuer", None, None);
        leaf.issuer_der = b"issuer-der".to_vec();
        leaf.authority_key_identifier = Some(AuthorityKeyIdentifierInfo {
            key_identifier: None,
            authority_cert_issuer: vec![SubjectAlternativeName {
                san_type: SubjectAlternativeNameType::DirectoryName,
                value: "CN=root-b".to_string(),
                oid: None,
                raw_der_hex: Some(crate::parser::to_hex(b"root-b-der")),
            }],
            authority_cert_serial_number: Some("02".to_string()),
        });
        let tree = build_certificate_tree_at(&[issuer_a, issuer_b, leaf], now());
        assert_eq!(tree.diagnostics[2].selected_issuer_index, Some(1));
    }

    #[test]
    fn test_cycle_is_broken_deterministically() {
        let certs = [
            cert("CN=a", "CN=b", None, None),
            cert("CN=b", "CN=a", None, None),
        ];
        let tree = build_certificate_tree_at(&certs, now());
        assert!(tree
            .diagnostics
            .iter()
            .any(|item| item.reason.as_deref() == Some("cycle_detected")));
        assert_eq!(tree.roots.len(), 1);
    }
}
