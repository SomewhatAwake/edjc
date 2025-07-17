use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test basic connection to Inara API without authentication
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Elite Dangerous Jump Calculator/0.1.0")
        .build()?;

    println!("Testing basic connectivity to Inara API...");

    // Try a simple request to see what we get
    let test_request = json!({
        "header": {
            "appName": "Elite Dangerous Jump Calculator",
            "appVersion": "0.1.0",
            "isDeveloped": true,
            "APIkey": "dummy_key_for_test"
        },
        "events": [
            {
                "eventName": "getCommanderProfile",
                "eventTimestamp": "2024-01-01T00:00:00Z",
                "eventData": {
                    "commanderName": "TestCMDR"
                }
            }
        ]
    });

    println!("Sending test request: {}", serde_json::to_string_pretty(&test_request)?);

    match client
        .post("https://inara.cz/inapi/v1/")
        .json(&test_request)
        .send()
    {
        Ok(response) => {
            println!("Status: {}", response.status());
            let text = response.text()?;
            println!("Response: {}", text);
        }
        Err(e) => {
            println!("Connection error: {}", e);
        }
    }

    Ok(())
}
