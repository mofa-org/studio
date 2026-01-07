# Streaming Support in dora-maas-client

## Current Status

The `enable_streaming` configuration option is available but not fully implemented. Currently:
- The flag is passed to the OpenAI API request
- The full response is still received and sent as a single chunk
- The text-segmenter downstream handles breaking it into smaller pieces for TTS

## Why Full Streaming Isn't Implemented Yet

1. **Library Limitation**: The `outfox-openai` library doesn't expose streaming API
2. **Complexity**: Proper SSE (Server-Sent Events) handling requires:
   - Parsing `data: [DONE]` and `data: {...}` chunks
   - Handling partial JSON responses
   - Managing connection state
   - Accumulating deltas into complete text

## How to Implement Full Streaming

### Option 1: Direct reqwest Implementation
```rust
use futures::StreamExt;
use reqwest::Client;

async fn stream_completion(&self, request: CreateChatCompletionRequest) -> Result<()> {
    let mut request_json = serde_json::to_value(&request)?;
    request_json["stream"] = serde_json::json!(true);
    
    let mut response = self.client
        .post(format!("{}/chat/completions", self.api_url))
        .header("Authorization", format!("Bearer {}", self.api_key))
        .json(&request_json)
        .send()
        .await?
        .bytes_stream();
    
    let mut buffer = String::new();
    while let Some(chunk) = response.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        
        // Parse SSE format
        for line in text.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data == "[DONE]" {
                    // Stream complete
                    break;
                }
                
                // Parse JSON delta
                if let Ok(delta) = serde_json::from_str::<StreamDelta>(data) {
                    if let Some(content) = delta.choices[0].delta.content {
                        // Send partial text
                        node.send_output("text", content)?;
                    }
                }
            }
        }
    }
}
```

### Option 2: Use async-openai crate
Replace `outfox-openai` with `async-openai` which has native streaming support:

```toml
[dependencies]
async-openai = "0.20"
```

```rust
use async_openai::{Client, types::{CreateChatCompletionRequestArgs}};
use futures::StreamExt;

let client = Client::new();
let request = CreateChatCompletionRequestArgs::default()
    .model("gpt-4")
    .messages(messages)
    .stream(true)
    .build()?;

let mut stream = client.chat().create_stream(request).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(response) => {
            if let Some(content) = response.choices[0].delta.content {
                // Send chunk
                node.send_output("text", content)?;
            }
        }
        Err(e) => // handle error
    }
}
```

## Benefits of Full Streaming

1. **Lower latency**: First token arrives faster
2. **Better UX**: User sees response being generated
3. **Memory efficiency**: No need to buffer entire response
4. **Natural pacing**: TTS can start speaking earlier

## Current Workaround

The text-segmenter node provides pseudo-streaming by:
1. Receiving the complete response
2. Breaking it into sentences/segments
3. Sending segments one at a time to TTS
4. Waiting for TTS completion before sending next segment

This achieves similar user experience but with higher initial latency.