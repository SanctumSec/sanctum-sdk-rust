# SanctumAI Rust SDK

[![crates.io](https://img.shields.io/crates/v/sanctum-ai.svg)](https://crates.io/crates/sanctum-ai)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI](https://github.com/jwgale/sanctum-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/jwgale/sanctum-sdk-rust/actions/workflows/ci.yml)

> Part of the [SanctumAI](https://github.com/jwgale/sanctum) ecosystem — secure credential management for AI agents.

Async Rust SDK for interacting with a SanctumAI vault. Supports Unix sockets and TCP, Ed25519 authentication, structured error types, and the **use-not-retrieve** pattern.

## Installation

```toml
[dependencies]
sanctum-ai = "0.1"
tokio = { version = "1", features = ["full"] }
```

Requires **Rust 1.75+** (MSRV).

## Quick Start

```rust
use sanctum_ai::SanctumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SanctumClient::connect("/var/run/sanctum.sock").await?;

    // List available credentials
    let creds = client.list().await?;
    for c in &creds {
        println!("  {} (tags: {:?})", c.path, c.tags);
    }

    // Retrieve a credential (lease auto-tracked)
    let cred = client.retrieve("openai/api_key", 300).await?;
    println!("Key starts with: {}...", &cred.value[..8]);

    // Use-not-retrieve — credential never leaves the vault
    let result = client.use_credential("openai/api_key", "http_header", None).await?;
    // result.data["header"] → "Authorization: Bearer sk-..."

    // Leases released on drop, or explicitly:
    client.release_lease(&cred.lease_id).await?;

    Ok(())
}
```

## Connecting

```rust
// Unix socket (default)
let client = SanctumClient::connect("/var/run/sanctum.sock").await?;

// TCP connection
let client = SanctumClient::connect("127.0.0.1:8200").await?;

// With explicit authentication
use ed25519_dalek::SigningKey;

let client = SanctumClient::connect("127.0.0.1:8200").await?;
let signing_key = SigningKey::from_bytes(&key_bytes);
client.authenticate("my-agent", &signing_key).await?;
```

## Use-Not-Retrieve

The **use-not-retrieve** pattern lets agents perform operations that require a credential without ever exposing the secret to the agent process. The vault executes the operation server-side and returns only the result.

```rust
use std::collections::HashMap;

// Sign a request — private key never leaves the vault
let mut params = HashMap::new();
params.insert("payload".into(), "data-to-sign".into());
let signed = client.use_credential("signing/key", "sign_payload", Some(params)).await?;

// Inject as HTTP header — agent never sees the raw token
let header = client.use_credential("openai/api_key", "http_header", None).await?;

// Encrypt data — encryption key stays in the vault
let mut params = HashMap::new();
params.insert("plaintext".into(), "sensitive data".into());
let encrypted = client.use_credential("encryption/key", "encrypt", Some(params)).await?;
```

This is the recommended pattern for production agents. Secrets never exist in agent memory.

## Error Handling

Errors are structured with actionable context:

```rust
use sanctum_ai::{SanctumClient, VaultError};

#[tokio::main]
async fn main() {
    let client = SanctumClient::connect("/var/run/sanctum.sock").await.unwrap();

    match client.retrieve("openai/api_key", 300).await {
        Ok(cred) => println!("Got: {}", cred.path),
        Err(VaultError::AccessDenied { detail, suggestion, .. }) => {
            eprintln!("No access: {detail}");
            if let Some(s) = suggestion {
                eprintln!("Suggestion: {s}");
            }
        }
        Err(VaultError::CredentialNotFound { detail, .. }) => {
            eprintln!("Not found: {detail}");
        }
        Err(VaultError::AuthFailed { .. }) => {
            eprintln!("Authentication failed — check your Ed25519 key");
        }
        Err(VaultError::VaultLocked { .. }) => {
            eprintln!("Vault is sealed — an operator needs to unseal it");
        }
        Err(e) => {
            eprintln!("[{}] {}", e.code(), e.detail());
            if let Some(url) = e.docs_url() {
                eprintln!("Docs: {url}");
            }
        }
    }
}
```

### Error Variants

| Variant | Code | Description |
|---|---|---|
| `VaultError::AuthFailed` | `AUTH_FAILED` | Authentication failed |
| `VaultError::AccessDenied` | `ACCESS_DENIED` | Insufficient permissions |
| `VaultError::CredentialNotFound` | `CREDENTIAL_NOT_FOUND` | Path doesn't exist |
| `VaultError::VaultLocked` | `VAULT_LOCKED` | Vault is sealed |
| `VaultError::LeaseExpired` | `LEASE_EXPIRED` | Lease timed out |
| `VaultError::RateLimited` | `RATE_LIMITED` | Too many requests |
| `VaultError::SessionExpired` | `SESSION_EXPIRED` | Re-authenticate needed |

All variants carry `detail`, `suggestion`, `docs_url`, and `context` fields.

## API Reference

| Method | Description |
|---|---|
| `SanctumClient::connect(addr)` | Connect via Unix socket or TCP |
| `authenticate(agent, key)` | Ed25519 challenge-response auth |
| `retrieve(path, ttl)` | Get credential with lease |
| `list()` | List available credentials |
| `release_lease(id)` | Release a credential lease |
| `use_credential(path, op, params)` | Use without retrieving |

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Run `cargo clippy` and `cargo fmt`
6. Submit a pull request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

MIT — see [LICENSE](LICENSE).

## Links

- 🏠 **Main project:** [github.com/jwgale/sanctum](https://github.com/jwgale/sanctum)
- 🌐 **Website:** [sanctumai.dev](https://sanctumai.dev)
- 🐍 **Python SDK:** [sanctum-sdk-python](https://github.com/jwgale/sanctum-sdk-python)
- 📦 **Node.js SDK:** [sanctum-sdk-node](https://github.com/jwgale/sanctum-sdk-node)
- 🐹 **Go SDK:** [sanctum-sdk-go](https://github.com/jwgale/sanctum-sdk-go)
- 🐛 **Issues:** [github.com/jwgale/sanctum-sdk-rust/issues](https://github.com/jwgale/sanctum-sdk-rust/issues)
