use crate::error::Error;

/// Fires a Discord webhook message. Fire-and-forget by design -- callers
/// should tokio::spawn this rather than await it inline, since a slow or
/// down webhook must never stall event processing.
pub async fn notify(webhook_url: &str, content: &str) -> Result<(), Error> {
    let client = reqwest::Client::new();
    let resp = client
        .post(webhook_url)
        .json(&serde_json::json!({ "content": content }))
        .send()
        .await
        .map_err(|e| Error::new(&format!("discord webhook request failed: {}", e)))?;

    if !resp.status().is_success() {
        return Err(Error::new(&format!(
            "discord webhook returned status {}",
            resp.status()
        )));
    }

    Ok(())
}
