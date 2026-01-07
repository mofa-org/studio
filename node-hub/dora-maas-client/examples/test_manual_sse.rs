//! Standalone test for manual SSE streaming
//! Run with: cargo run --example test_manual_sse

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!("Starting manual SSE test...");

    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    eprintln!("API key length: {}", api_key.len());

    let url = "https://api.openai.com/v1/chat/completions".to_string();

    let request_body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant. Be brief."},
            {"role": "user", "content": "What is 2+2? Answer in one word."}
        ],
        "stream": true
    });

    eprintln!("Request body: {}", serde_json::to_string_pretty(&request_body)?);

    // Create HTTP client
    eprintln!("Creating HTTP client...");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .expect("Failed to create client");
    eprintln!("HTTP client created successfully");

    // Build and send request
    eprintln!("Building request...");
    let request = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&request_body);

    eprintln!("Sending request...");
    let response = request.send().await?;

    eprintln!("Response status: {}", response.status());
    eprintln!("Response headers:");
    for (key, value) in response.headers() {
        eprintln!("  {}: {:?}", key, value);
    }

    if !response.status().is_success() {
        let error_text = response.text().await?;
        eprintln!("API error: {}", error_text);
        return Err(eyre::eyre!("API error: {}", error_text));
    }

    // Process the byte stream as SSE
    use futures::TryStreamExt;
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut chunk_count = 0;

    eprintln!("Processing SSE stream...");
    while let Some(chunk_result) = stream.try_next().await? {
        chunk_count += 1;
        let chunk_str = String::from_utf8_lossy(&chunk_result);
        buffer.push_str(&chunk_str);

        // Process complete SSE events (lines ending with \n\n)
        while let Some(pos) = buffer.find("\n\n") {
            let event = buffer[..pos].to_string();
            buffer = buffer[pos + 2..].to_string();

            // Parse SSE data lines
            for line in event.lines() {
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        eprintln!("Stream complete!");
                        continue;
                    }

                    // Parse JSON and extract content
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                        if let Some(content) = json
                            .get("choices")
                            .and_then(|c| c.get(0))
                            .and_then(|c| c.get("delta"))
                            .and_then(|d| d.get("content"))
                            .and_then(|c| c.as_str())
                        {
                            print!("{}", content);
                            std::io::Write::flush(&mut std::io::stdout())?;
                        }
                    }
                }
            }
        }
    }

    println!();
    eprintln!("Processed {} chunks", chunk_count);
    eprintln!("Test complete!");
    Ok(())
}
