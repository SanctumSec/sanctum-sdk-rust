use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A JSON-RPC request.
#[derive(Debug, Serialize)]
pub struct RpcRequest {
    pub id: u64,
    pub method: String,
    pub params: Value,
}

/// A JSON-RPC response.
#[derive(Debug, Deserialize)]
pub struct RpcResponse {
    pub id: u64,
    pub result: Option<Value>,
    pub error: Option<Value>,
}

/// A retrieved credential with lease information.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Credential {
    pub path: String,
    pub value: Value,
    pub lease_id: String,
    pub ttl: u64,
}

/// Summary info for a credential (returned by list).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CredentialInfo {
    pub path: String,
    #[serde(rename = "type")]
    pub credential_type: Option<String>,
    pub description: Option<String>,
}

/// Result of a use-not-retrieve operation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UseResult {
    pub success: bool,
    pub output: Option<Value>,
}

/// Authentication challenge from the server.
#[derive(Debug, Deserialize)]
pub struct AuthChallenge {
    pub challenge: String,
}

/// Authentication result.
#[derive(Debug, Deserialize)]
pub struct AuthResult {
    pub authenticated: bool,
    pub session_id: Option<String>,
}
