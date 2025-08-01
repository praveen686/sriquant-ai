//! Simple test for our monoio-native TLS implementation

use sriquant_exchanges::MonoioHttpsClient;

#[monoio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing monoio-native TLS implementation");
    
    // Create HTTPS client
    let client = MonoioHttpsClient::new()?;
    println!("✅ HTTPS client created successfully");
    
    // Test with Binance API ping
    println!("🏓 Testing connectivity to Binance API...");
    let response = client.get("https://api.binance.com/api/v3/ping").await?;
    
    println!("📡 Response Status: {}", response.status);
    println!("📊 Response Headers: {:?}", response.headers);
    println!("📄 Response Body: {}", response.body);
    
    if response.status == 200 {
        println!("✅ TLS integration working correctly!");
    } else {
        println!("❌ Unexpected response status: {}", response.status);
    }
    
    Ok(())
}