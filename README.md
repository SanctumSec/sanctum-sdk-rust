# sanctum-ai

Rust SDK for [SanctumAI](https://sanctumai.dev) — credential management for AI agents.

## Features

- Async client over Unix sockets or TCP
- JSON-RPC with 4-byte length-prefix framing
- Ed25519 challenge-response authentication
- Structured error types with actionable suggestions
- Use-not-retrieve pattern for secure credential operations

## Installation

```toml
[dependencies]
sanctum-ai = "0.1"
```

## Quick Start

```rust
use sanctum_ai::SanctumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SanctumClient::connect("/var/run/sanctum.sock").await?;

    let creds = client.list().await?;
    for c in &creds {
        println!("{}", c.path);
    }

    let cred = client.retrieve("database/primary", 300).await?;
    println!("Got: {} (lease: {})", cred.path, cred.lease_id);

    client.release_lease(&cred.lease_id).await?;
    Ok(())
}
```

## Authentication

```rust
use ed25519_dalek::SigningKey;

let client = SanctumClient::connect("127.0.0.1:9090").await?;
let signing_key = SigningKey::from_bytes(&key_bytes);
client.authenticate("my-agent", &signing_key).await?;
```

## API

| Method | Description |
|--------|-------------|
| `SanctumClient::connect(addr)` | Connect via Unix socket or TCP |
| `authenticate(agent, key)` | Ed25519 challenge-response auth |
| `retrieve(path, ttl)` | Get credential with lease |
| `list()` | List available credentials |
| `release_lease(id)` | Release a credential lease |
| `use_credential(path, op, params)` | Use without retrieving |

## License

MIT
