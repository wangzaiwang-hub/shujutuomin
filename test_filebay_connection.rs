use std::time::Duration;

#[tokio::main]
async fn main() {
    println!("=== FileBay Connection Test ===\n");
    
    let url = "https://uat-filebay.cheersai.cloud";
    let api_endpoint = format!("{}/api/v1/repos/junqianxi/cheersAI", url);
    
    println!("Testing URL: {}", api_endpoint);
    println!("Testing different TLS configurations...\n");
    
    // Test 1: Default reqwest (native-tls/schannel on Windows)
    println!("--- Test 1: Default native-tls ---");
    test_connection_native(&api_endpoint).await;
    
    // Test 2: rustls with danger_accept_invalid_certs
    println!("\n--- Test 2: rustls + accept_invalid_certs ---");
    test_connection_rustls(&api_endpoint).await;
    
    // Test 3: rustls + accept_invalid_certs + no_proxy
    println!("\n--- Test 3: rustls + accept_invalid_certs + no_proxy ---");
    test_connection_rustls_no_proxy(&api_endpoint).await;
    
    // Test 4: Simple GET to base URL
    println!("\n--- Test 4: Base URL test ---");
    test_base_url(url).await;
}

async fn test_connection_native(url: &str) {
    match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => {
            match client.get(url).send().await {
                Ok(resp) => println!("✅ Success! Status: {}", resp.status()),
                Err(e) => println!("❌ Failed: {}", e),
            }
        }
        Err(e) => println!("❌ Client build failed: {}", e),
    }
}

async fn test_connection_rustls(url: &str) {
    match reqwest::Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => {
            match client.get(url).send().await {
                Ok(resp) => println!("✅ Success! Status: {}", resp.status()),
                Err(e) => println!("❌ Failed: {}", e),
            }
        }
        Err(e) => println!("❌ Client build failed: {}", e),
    }
}

async fn test_connection_rustls_no_proxy(url: &str) {
    match reqwest::Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .connect_timeout(Duration::from_secs(5))
        .build()
    {
        Ok(client) => {
            match client.get(url).send().await {
                Ok(resp) => {
                    println!("✅ Success! Status: {}", resp.status());
                    if let Ok(text) = resp.text().await {
                        println!("Response preview: {}", &text[..text.len().min(200)]);
                    }
                }
                Err(e) => println!("❌ Failed: {}", e),
            }
        }
        Err(e) => println!("❌ Client build failed: {}", e),
    }
}

async fn test_base_url(url: &str) {
    match reqwest::Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(client) => {
            match client.get(url).send().await {
                Ok(resp) => {
                    println!("✅ Success! Status: {}", resp.status());
                    println!("Headers: {:?}", resp.headers());
                }
                Err(e) => println!("❌ Failed: {}", e),
            }
        }
        Err(e) => println!("❌ Client build failed: {}", e),
    }
}
