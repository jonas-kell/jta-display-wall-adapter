use reqwest::Client;

/// Send a single TCP message to the Dockerized NRBF service
pub async fn decode_single_nrbf(packet: &[u8], service_url: &str) -> String {
    let client = Client::new();
    match client.post(service_url).body(packet.to_vec()).send().await {
        Ok(resp) if resp.status().is_success() => resp
            .text()
            .await
            .unwrap_or_else(|_| "<empty response>".into()),
        Ok(resp) => {
            format!(
                "Deserialization failed: {}",
                resp.text().await.unwrap_or_default()
            )
        }
        Err(e) => format!("Error sending to NRBF service: {}", e),
    }
}
