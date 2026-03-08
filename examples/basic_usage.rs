use sanctum_ai::SanctumClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the vault over TCP (default port 7600)
    let client = SanctumClient::connect("127.0.0.1:7600").await?;

    // List available credentials
    let credentials = client.list().await?;
    println!("Available credentials:");
    for cred in &credentials {
        println!("  - {}", cred.path);
    }

    // Use a credential without ever seeing it (the proxy pattern)
    // The vault makes the HTTP request on your behalf
    let result = client
        .use_credential(
            "openai/api-key",
            "http_request",
            json!({
                "method": "POST",
                "url": "https://api.openai.com/v1/chat/completions",
                "headers": {"Content-Type": "application/json"},
                "body": "{\"model\": \"gpt-4\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello\"}]}",
                "header_type": "bearer"
            }),
        )
        .await?;
    println!("Response: {:?}", result.output);

    // Or just get the auth header without making a request
    let header = client
        .use_credential(
            "github/token",
            "http_header",
            json!({ "header_type": "bearer" }),
        )
        .await?;
    println!("Header: {:?}", header.output);

    // If you do need the raw value (try to avoid this)
    let cred = client.retrieve("database/password", 300).await?;
    println!("Retrieved: {} (lease: {})", cred.path, cred.lease_id);
    client.release_lease(&cred.lease_id).await?;

    Ok(())
}
