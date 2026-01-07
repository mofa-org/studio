# How to Enable True Streaming in dora-maas-client

## Current Implementation
The current implementation uses `outfox-openai` which doesn't support streaming. Responses are received in full before being sent.

## Streaming Implementation Available
A full streaming implementation using `async-openai` is provided in:
- `Cargo_streaming.toml` - Dependencies with async-openai
- `src/main_streaming.rs` - Full streaming implementation

## How to Switch to Streaming

### 1. Backup current files
```bash
cp Cargo.toml Cargo_original.toml
cp src/main.rs src/main_original.rs
```

### 2. Switch to streaming version
```bash
cp Cargo_streaming.toml Cargo.toml
cp src/main_streaming.rs src/main.rs
```

### 3. Set OpenAI API Key
The async-openai library reads from environment variable:
```bash
export OPENAI_API_KEY="your-api-key-here"
```

### 4. Rebuild
```bash
cargo build --release -p dora-maas-client
```

## What Changes with Streaming

### Benefits:
1. **Lower latency** - First token arrives in ~200ms instead of waiting for full response
2. **Real-time feel** - Users see text appearing word by word
3. **Better for voice** - TTS can start speaking earlier
4. **Memory efficient** - No need to buffer entire response

### Behavior Changes:
- Text output sends chunks as they arrive (not complete sentences)
- Multiple small outputs instead of one large output
- Text-segmenter might need adjustment to handle streaming chunks

### Configuration:
```toml
enable_streaming = true  # Enable streaming (default)
enable_streaming = false # Disable streaming (fallback to batch mode)
```

## Streaming Output Format

With streaming enabled, the `text` output will send:
1. Multiple small chunks (words/partial sentences)
2. Each chunk as a separate output event
3. Chunks arrive in order but at variable rates

Example sequence:
```
Output 1: "The capital"
Output 2: " of France"
Output 3: " is Paris"
Output 4: ", which is"
Output 5: " known for"
...
```

## Compatibility Notes

### Works Well With:
- Text display nodes that accumulate chunks
- Voice chat with segment-aware TTS
- Real-time UI updates

### May Need Adjustment:
- Text-segmenter (might need to accumulate before segmenting)
- Nodes expecting complete sentences
- Logging/recording nodes

## Reverting to Original

To go back to non-streaming:
```bash
cp Cargo_original.toml Cargo.toml
cp src/main_original.rs src/main.rs
cargo build --release -p dora-maas-client
```

## Testing Streaming

1. Enable in config:
```toml
enable_streaming = true
```

2. Run test:
```bash
cd examples/test-maas-client
dora start dataflow.yml --attach
```

3. Watch for streaming chunks in output

## Performance Comparison

| Metric | Non-Streaming | Streaming |
|--------|--------------|-----------|
| First Token | 1-3s | 200-500ms |
| Total Time | Same | Same |
| Memory Use | Higher | Lower |
| Network | One request | SSE stream |
| User Experience | Wait then see all | See as it generates |