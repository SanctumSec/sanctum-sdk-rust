use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error codes returned by the SanctumAI vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    AuthFailed,
    AccessDenied,
    CredentialNotFound,
    VaultLocked,
    LeaseExpired,
    RateLimited,
    SessionExpired,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthFailed => write!(f, "AUTH_FAILED"),
            Self::AccessDenied => write!(f, "ACCESS_DENIED"),
            Self::CredentialNotFound => write!(f, "CREDENTIAL_NOT_FOUND"),
            Self::VaultLocked => write!(f, "VAULT_LOCKED"),
            Self::LeaseExpired => write!(f, "LEASE_EXPIRED"),
            Self::RateLimited => write!(f, "RATE_LIMITED"),
            Self::SessionExpired => write!(f, "SESSION_EXPIRED"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// Structured error from the vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref detail) = self.detail {
            write!(f, " — {detail}")?;
        }
        Ok(())
    }
}

/// Top-level SDK error type.
#[derive(Debug, Error)]
pub enum SanctumError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Vault error: {0}")]
    Vault(Box<VaultError>),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Signature error: {0}")]
    Signature(#[from] ed25519_dalek::SignatureError),
}

pub type Result<T> = std::result::Result<T, SanctumError>;
