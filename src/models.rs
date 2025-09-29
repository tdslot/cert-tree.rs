use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone)]
pub struct CertificateDisplayItem {
    pub display_name: String,
    pub valid_until: String,
    pub validity_status: ValidityStatus,
    pub validation_status: ValidationStatus,
    pub certificate_info: CertificateInfo,
}

#[derive(Debug, Clone)]
pub enum ValidityStatus {
    Valid,
    ExpiringSoon, // within 30 days
    Expired,
}

impl ValidityStatus {
    pub fn from_dates(not_after: &str) -> Self {
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

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            ValidityStatus::Valid => ratatui::style::Color::Green,
            ValidityStatus::ExpiringSoon => ratatui::style::Color::Yellow,
            ValidityStatus::Expired => ratatui::style::Color::Red,
        }
    }

    pub fn text(&self) -> &'static str {
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
    pub fn text(&self) -> &'static str {
        match self {
            ValidationStatus::Valid => "✓ Valid Chain",
            ValidationStatus::InvalidChain => "✗ Invalid Chain",
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            ValidationStatus::Valid => ratatui::style::Color::Green,
            ValidationStatus::InvalidChain => ratatui::style::Color::Red,
        }
    }
}
