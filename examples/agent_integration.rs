use ed25519_dalek::SigningKey;
use serde_json::json;
use sanctum_ai::SanctumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect over TCP
    let client = SanctumClient::connect("127.0.0.1:9090").await?;

    // Load signing key (in production, load from secure storage)
    let key_bytes = hex::decode("your-64-hex-char-private-key-here-replace-me-000000000000000000")
        .expect("invalid hex key");
    let signing_key = SigningKey::from_bytes(
        key_bytes.as_slice().try_into().expect("key must be 32 bytes"),
    );

    // Authenticate
    let auth = client.authenticate("my-agent", &signing_key).await?;
    println!("Authenticated! Session: {:?}", auth.session_id);

    // Use a credential without retrieving it (use-not-retrieve pattern)
    let result = client
        .use_credential("api/openai", "chat.completions", json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "Hello!"}]
        }))
        .await?;
    println!("Use result: {:?}", result);

    Ok(())
}
