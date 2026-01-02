use crate::models::{CertificateInfo, CertificateNode, CertificateTree, ValidationStatus};
use std::collections::HashMap;

pub fn build_certificate_tree(certificates: &[CertificateInfo]) -> CertificateTree {
    let mut cert_map: HashMap<String, CertificateInfo> = HashMap::new();
    let mut issuer_map: HashMap<String, Vec<String>> = HashMap::new();

    // Build maps for quick lookup
    for cert in certificates {
        cert_map.insert(cert.subject.clone(), cert.clone());

        // Group certificates by issuer
        issuer_map
            .entry(cert.issuer.clone())
            .or_default()
            .push(cert.subject.clone());
    }

    // Find root certificates (self-signed or where issuer is not in our set)
    let mut roots = Vec::new();
    let mut processed = std::collections::HashSet::new();

    for cert in certificates {
        if !cert_map.contains_key(&cert.issuer) || cert.subject == cert.issuer {
            // This is a root certificate
            if !processed.contains(&cert.subject) {
                let node = build_tree_node(cert, &cert_map, &issuer_map, &mut processed);
                roots.push(node);
            }
        }
    }

    // Handle any remaining certificates that might not have been processed
    for cert in certificates {
        if !processed.contains(&cert.subject) {
            let node = build_tree_node(cert, &cert_map, &issuer_map, &mut processed);
            roots.push(node);
        }
    }

    let mut tree = CertificateTree { roots };
    validate_certificate_chain(&mut tree);
    tree
}

fn build_tree_node(
    cert: &CertificateInfo,
    cert_map: &HashMap<String, CertificateInfo>,
    issuer_map: &HashMap<String, Vec<String>>,
    processed: &mut std::collections::HashSet<String>,
) -> CertificateNode {
    processed.insert(cert.subject.clone());

    let validity_status = crate::models::ValidityStatus::from_dates(&cert.not_after);

    let mut children = Vec::new();
    if let Some(issued_certs) = issuer_map.get(&cert.subject) {
        for subject in issued_certs {
            if let Some(child_cert) = cert_map.get(subject) {
                if !processed.contains(subject) {
                    let child_node = build_tree_node(child_cert, cert_map, issuer_map, processed);
                    children.push(child_node);
                }
            }
        }
    }

    CertificateNode {
        cert: cert.clone(),
        children,
        validity_status,
        validation_status: ValidationStatus::Valid,
    }
}

pub fn validate_certificate_chain(tree: &mut CertificateTree) {
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
    } else if node.cert.subject == node.cert.issuer {
        node.validation_status = ValidationStatus::Valid;
    } else {
        node.validation_status = ValidationStatus::InvalidChain;
    }

    for child in &mut node.children {
        validate_node(child, Some(&node.cert));
    }
}
