# SanctumAI Rust SDK

[![crates.io](https://img.shields.io/crates/v/sanctum-ai.svg)](https://crates.io/crates/sanctum-ai)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust SDK for [SanctumAI](https://github.com/SanctumSec/sanctum) — a local-first credential vault for AI agents. Your agent authenticates, requests credentials through the vault, and never handles raw secrets directly.

## Install

```toml
[dependencies]
sanctum-ai = "0.4"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

Requires **Rust 1.75+**.

## Quick Start

```rust
use sanctum_ai::SanctumClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the SanctumAI daemon (TCP, default port 7600)
    let client = SanctumClient::connect("127.0.0.1:7600").await?;

    // Authenticate with Ed25519 challenge-response
    use ed25519_dalek::SigningKey;
    let signing_key = SigningKey::from_bytes(&key_bytes);
    client.authenticate("my-agent", &signing_key).await?;

    // Make an API call through the vault — your agent never sees the key
    let result = client
        .use_credential("openai/api-key", "http_request", json!({
            "method": "POST",
            "url": "https://api.openai.com/v1/chat/completions",
            "headers": {"Content-Type": "application/json"},
            "body": "{\"model\": \"gpt-4\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello\"}]}",
            "header_type": "bearer"
        }))
        .await?;

    println!("Status: {:?}", result.output);
    Ok(())
}
```

## Use Don't Retrieve

The core pattern: instead of fetching a secret and using it yourself, tell the vault *what you want to do* and let it handle the credential. The secret never enters your agent's memory.

### Proxy an HTTP Request

The vault injects the credential and makes the request for you:

```rust
let result = client
    .use_credential("openai/api-key", "http_request", json!({
        "method": "POST",
        "url": "https://api.openai.com/v1/chat/completions",
        "headers": {"Content-Type": "application/json"},
        "body": "{\"model\": \"gpt-4\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello\"}]}",
        "header_type": "bearer"  // bearer, api_key, basic, custom
    }))
    .await?;

// result.output contains: { "status": 200, "headers": {...}, "body": "..." }
// Your agent NEVER sees the API key.
```

### Get an Auth Header

If you need to make the HTTP request yourself but still want the vault to construct the auth header:

```rust
let header = client
    .use_credential("github/token", "http_header", json!({
        "header_type": "bearer"
    }))
    .await?;

// header.output contains: { "header_name": "Authorization", "header_value": "Bearer ghp_..." }
```

### HMAC Signing

Sign a payload without the signing key leaving the vault:

```rust
let signed = client
    .use_credential("webhook/secret", "sign", json!({
        "algorithm": "hmac-sha256",
        "data": "payload-to-sign"
    }))
    .await?;

// signed.output contains: { "signature": "base64-encoded-signature" }
```

### Encrypt / Decrypt

```rust
// Encrypt — encryption key stays in the vault
let encrypted = client
    .use_credential("encryption/key", "encrypt", json!({
        "data": "sensitive-payload"
    }))
    .await?;

// Decrypt
let decrypted = client
    .use_credential("encryption/key", "decrypt", json!({
        "data": encrypted.output.unwrap()["ciphertext"].as_str().unwrap()
    }))
    .await?;
```

## API Reference

### `SanctumClient::connect(addr)`

Connect to the SanctumAI daemon. Accepts a TCP address (e.g. `"127.0.0.1:7600"`) or a Unix socket path on supported platforms.

```rust
let client = SanctumClient::connect("127.0.0.1:7600").await?;
```

### `authenticate(agent_name, signing_key)`

Ed25519 challenge-response authentication. The daemon sends a challenge, the SDK signs it, and returns the signed response.

```rust
use ed25519_dalek::SigningKey;

let signing_key = SigningKey::from_bytes(&key_bytes);
let auth = client.authenticate("my-agent", &signing_key).await?;
// auth.session_id contains the session token
```

### `retrieve(path, ttl)`

Get a credential's raw value with a lease. The lease auto-expires after `ttl` seconds. **Prefer `use_credential` when possible.**

```rust
let cred = client.retrieve("database/password", 300).await?;
println!("value: {:?}, lease: {}", cred.value, cred.lease_id);
```

### `list()`

List all credentials the authenticated agent has access to.

```rust
let credentials = client.list().await?;
for c in &credentials {
    println!("{} ({:?})", c.path, c.credential_type);
}
```

### `use_credential(path, operation, params)`

Use a credential without seeing it. This is the recommended pattern. See [Use Don't Retrieve](#use-dont-retrieve) for full examples.

| Operation | Description | Key Params |
|---|---|---|
| `http_request` | Vault makes an HTTP request with the credential injected | `method`, `url`, `headers`, `body`, `header_type` |
| `http_header` | Get the auth header without making a request | `header_type` (`bearer`, `api_key`, `basic`, `custom`) |
| `sign` | HMAC-sign a payload | `algorithm`, `data` |
| `encrypt` | Encrypt data with the credential | `data` |
| `decrypt` | Decrypt data with the credential | `data` |

### `release_lease(lease_id)`

Release a credential lease early (before TTL expiry).

```rust
client.release_lease(&cred.lease_id).await?;
```

## Error Handling

All methods return `Result<T, SanctumError>`. Vault-specific errors are wrapped in `SanctumError::Vault` with structured fields:

```rust
use sanctum_ai::{SanctumClient, SanctumError, VaultError};

match client.retrieve("openai/api-key", 300).await {
    Ok(cred) => println!("Got: {}", cred.path),
    Err(SanctumError::Vault(err)) => {
        eprintln!("[{}] {}", err.code, err.message);
        if let Some(detail) = &err.detail {
            eprintln!("  Detail: {detail}");
        }
        if let Some(suggestion) = &err.suggestion {
            eprintln!("  Suggestion: {suggestion}");
        }
    }
    Err(SanctumError::Io(err)) => eprintln!("Connection error: {err}"),
    Err(e) => eprintln!("Error: {e}"),
}
```

### Error Codes

| Code | Meaning |
|---|---|
| `AUTH_FAILED` | Ed25519 authentication failed |
| `ACCESS_DENIED` | Agent lacks permission for this credential |
| `CREDENTIAL_NOT_FOUND` | No credential at this path |
| `VAULT_LOCKED` | Vault is sealed — an operator must unseal it |
| `LEASE_EXPIRED` | Lease timed out, re-retrieve the credential |
| `RATE_LIMITED` | Too many requests, back off and retry |
| `SESSION_EXPIRED` | Session expired, re-authenticate |

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Write tests for new functionality
4. Ensure `cargo test`, `cargo clippy`, and `cargo fmt --check` pass
5. Submit a pull request

## License

MIT — see [LICENSE](LICENSE).
