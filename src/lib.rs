//! # sanctum-ai
//!
//! Rust SDK for SanctumAI — credential management for AI agents.
//!
//! Provides an async client for communicating with a SanctumAI vault over
//! Unix sockets or TCP using JSON-RPC with length-prefix framing and
//! Ed25519 challenge-response authentication.

pub mod client;
pub mod error;
pub mod protocol;
pub mod types;

pub use client::SanctumClient;
pub use error::{ErrorCode, SanctumError, VaultError};
pub use types::{Credential, CredentialInfo, UseResult};
