use sanctum_ai::SanctumClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to the vault via Unix socket
    let client = SanctumClient::connect("/var/run/sanctum.sock").await?;

    // List available credentials
    let credentials = client.list().await?;
    println!("Available credentials:");
    for cred in &credentials {
        println!("  - {}", cred.path);
    }

    // Retrieve a credential with a 300-second TTL
    let cred = client.retrieve("database/primary", 300).await?;
    println!("Retrieved: {} (lease: {})", cred.path, cred.lease_id);

    // Release the lease when done
    client.release_lease(&cred.lease_id).await?;
    println!("Lease released.");

    Ok(())
}
