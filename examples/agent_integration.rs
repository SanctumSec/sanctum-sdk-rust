use ed25519_dalek::SigningKey;
use serde_json::json;
use sanctum_ai::SanctumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect over TCP to the SanctumAI daemon
    let client = SanctumClient::connect("127.0.0.1:7600").await?;

    // Load signing key (in production, load from secure storage)
    let key_bytes = hex::decode(
        "your-64-hex-char-private-key-here-replace-me-000000000000000000"
    ).expect("invalid hex key");
    let signing_key = SigningKey::from_bytes(
        key_bytes.as_slice().try_into().expect("key must be 32 bytes"),
    );

    // Authenticate with Ed25519 challenge-response
    let auth = client.authenticate("my-agent", &signing_key).await?;
    println!("Authenticated! Session: {:?}", auth.session_id);

    // Proxy an HTTP request through the vault — agent never sees the API key
    let result = client
        .use_credential("openai/api-key", "http_request", json!({
            "method": "POST",
            "url": "https://api.openai.com/v1/chat/completions",
            "headers": {"Content-Type": "application/json"},
            "body": "{\"model\": \"gpt-4\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello!\"}]}",
            "header_type": "bearer"
        }))
        .await?;
    println!("HTTP response: {:?}", result.output);

    // HMAC signing — signing key never leaves the vault
    let signed = client
        .use_credential("webhook/secret", "sign", json!({
            "algorithm": "hmac-sha256",
            "data": "payload-to-sign"
        }))
        .await?;
    println!("Signature: {:?}", signed.output);

    // Get just the auth header for use with your own HTTP client
    let header = client
        .use_credential("github/token", "http_header", json!({
            "header_type": "bearer"
        }))
        .await?;
    println!("Auth header: {:?}", header.output);

    Ok(())
}
