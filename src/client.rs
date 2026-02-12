#[cfg(unix)]
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use ed25519_dalek::{SigningKey, Signer};
use serde_json::{json, Value};
use tokio::io::{BufReader, BufWriter};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::sync::Mutex;

use crate::error::{SanctumError, Result, VaultError};
use crate::protocol;
use crate::types::*;

enum Transport {
    #[cfg(unix)]
    Unix {
        reader: BufReader<ReadHalf<UnixStream>>,
        writer: BufWriter<WriteHalf<UnixStream>>,
    },
    Tcp {
        reader: BufReader<ReadHalf<TcpStream>>,
        writer: BufWriter<WriteHalf<TcpStream>>,
    },
}

/// Client for communicating with a SanctumAI vault.
pub struct SanctumClient {
    transport: Mutex<Transport>,
    next_id: AtomicU64,
}

impl SanctumClient {
    /// Connect to a SanctumAI vault via Unix socket path or TCP address.
    ///
    /// If `addr` starts with `/` or `.`, it is treated as a Unix socket path.
    /// Otherwise it is treated as a TCP address (e.g. `127.0.0.1:9090`).
    pub async fn connect(addr: &str) -> Result<Self> {
        #[cfg(unix)]
        let is_unix_path = addr.starts_with('/') || addr.starts_with('.') || Path::new(addr).exists();
        #[cfg(not(unix))]
        let is_unix_path = false;

        let transport = if is_unix_path {
            #[cfg(unix)]
            {
                let stream = UnixStream::connect(addr).await?;
                let (r, w) = tokio::io::split(stream);
                Transport::Unix {
                    reader: BufReader::new(r),
                    writer: BufWriter::new(w),
                }
            }
            #[cfg(not(unix))]
            {
                return Err(SanctumError::Protocol("Unix sockets not supported on Windows".into()));
            }
        } else {
            let stream = TcpStream::connect(addr).await?;
            let (r, w) = tokio::io::split(stream);
            Transport::Tcp {
                reader: BufReader::new(r),
                writer: BufWriter::new(w),
            }
        };

        Ok(Self {
            transport: Mutex::new(transport),
            next_id: AtomicU64::new(1),
        })
    }

    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let req = RpcRequest {
            id: self.next_id(),
            method: method.to_string(),
            params,
        };

        let mut transport = self.transport.lock().await;
        match &mut *transport {
            #[cfg(unix)]
            Transport::Unix { reader, writer } => {
                protocol::write_frame(writer, &req).await?;
                let resp = protocol::read_frame(reader).await?;
                Self::handle_response(resp)
            }
            Transport::Tcp { reader, writer } => {
                protocol::write_frame(writer, &req).await?;
                let resp = protocol::read_frame(reader).await?;
                Self::handle_response(resp)
            }
        }
    }

    fn handle_response(resp: RpcResponse) -> Result<Value> {
        if let Some(err) = resp.error {
            let vault_err: VaultError = serde_json::from_value(err)
                .unwrap_or_else(|_| VaultError {
                    code: crate::error::ErrorCode::Unknown,
                    message: "Unknown error".into(),
                    detail: None,
                    suggestion: None,
                    docs_url: None,
                    context: None,
                });
            return Err(SanctumError::Vault(Box::new(vault_err)));
        }
        resp.result.ok_or_else(|| SanctumError::Protocol("response missing both result and error".into()))
    }

    /// Authenticate with the vault using Ed25519 challenge-response.
    pub async fn authenticate(&self, agent_name: &str, signing_key: &SigningKey) -> Result<AuthResult> {
        // Step 1: Request challenge
        let challenge_resp = self.call("auth.challenge", json!({ "agent": agent_name })).await?;
        let challenge: AuthChallenge = serde_json::from_value(challenge_resp)?;

        // Step 2: Sign challenge
        let challenge_bytes = hex::decode(&challenge.challenge)
            .map_err(|e| SanctumError::Protocol(format!("invalid challenge hex: {e}")))?;
        let signature = signing_key.sign(&challenge_bytes);

        // Step 3: Submit signature
        let result = self.call("auth.verify", json!({
            "agent": agent_name,
            "signature": hex::encode(signature.to_bytes()),
        })).await?;

        let auth_result: AuthResult = serde_json::from_value(result)?;
        if !auth_result.authenticated {
            return Err(SanctumError::Auth("server rejected authentication".into()));
        }
        Ok(auth_result)
    }

    /// Retrieve a credential by path with a TTL in seconds.
    pub async fn retrieve(&self, path: &str, ttl: u64) -> Result<Credential> {
        let result = self.call("credential.retrieve", json!({ "path": path, "ttl": ttl })).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// List available credentials.
    pub async fn list(&self) -> Result<Vec<CredentialInfo>> {
        let result = self.call("credential.list", json!({})).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Release a credential lease.
    pub async fn release_lease(&self, lease_id: &str) -> Result<()> {
        self.call("lease.release", json!({ "lease_id": lease_id })).await?;
        Ok(())
    }

    /// Use a credential without retrieving it (use-not-retrieve pattern).
    pub async fn use_credential(&self, path: &str, operation: &str, params: Value) -> Result<UseResult> {
        let result = self.call("credential.use", json!({
            "path": path,
            "operation": operation,
            "params": params,
        })).await?;
        Ok(serde_json::from_value(result)?)
    }
}
