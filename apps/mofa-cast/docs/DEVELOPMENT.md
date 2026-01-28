# MoFA Cast - Development Guide

**Version**: 0.6.3
**Last Updated**: 2026-01-17

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Code Organization](#code-organization)
5. [Key Components](#key-components)
6. [Adding Features](#adding-features)
7. [Testing](#testing)
8. [Build and Release](#build-and-release)

---

## Getting Started

### Prerequisites

```bash
# Rust toolchain
rustc --version  # 1.70 or higher
cargo --version

# Dora dataflow system
dora --version  # 0.3.0 or higher

# Python (for voice router)
python3 --version  # 3.8 or higher
pip3 install -e node-hub/dora-voice-router

# FFmpeg (for MP3 export)
ffmpeg -version  # Optional, 4.0 or higher
```

### Development Environment Setup

```bash
# Clone repository
cd /path/to/mofa-studio

# Install dependencies
cargo build

# Run application
cargo run -p mofa-studio

# Run tests
cargo test -p mofa-cast
```

### IDE Configuration

**VS Code** (recommended):
- Extensions: rust-analyzer, CodeLLDB
- Settings:
  ```json
  {
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy"
  }
  ```

---

## Project Structure

```
apps/mofa-cast/
├── Cargo.toml                   # Package dependencies
├── README.md                    # Project overview
├── ARCHITECTURE.md              # Technical architecture
├── CHANGELOG.md                 # Version history
│
├── docs/                        # Documentation
│   ├── USER_GUIDE.md            # User documentation
│   ├── TROUBLESHOOTING.md       # Issue resolution
│   ├── DEVELOPMENT.md           # This file
│   └── HISTORY.md               # Development history
│
├── dataflow/                    # Dora dataflow configs
│   ├── multi-voice-batch-tts.yml     # Main multi-voice pipeline
│   └── batch-tts.yml                 # Legacy single-voice (deprecated)
│
├── test_samples/                # Test files
│   ├── sample_plain.txt
│   ├── sample_json.json
│   └── sample_markdown.md
│
└── src/                         # Source code
    ├── lib.rs                   # MoFA Studio app integration
    ├── screen/
    │   ├── mod.rs               # Screen module exports
    │   ├── main.rs              # Main UI screen (2400+ lines)
    │   └── design.rs            # Live design definitions
    ├── transcript_parser.rs     # Parse transcript formats
    ├── script_templates.rs      # Pre-built script templates
    ├── recent_files.rs          # Recent files management
    ├── tts_batch.rs             # TTS engine abstraction
    ├── audio_mixer.rs           # Audio mixing and export
    ├── dora_integration.rs      # Dora dataflow integration
    └── dora_process_manager.rs  # Dora lifecycle management
```

### Module Dependencies

```
screen/main.rs (UI)
    ├─→ transcript_parser.rs   (Parse scripts)
    ├─→ dora_integration.rs     (TTS synthesis)
    │   ├─→ tts_batch.rs        (TTS engines)
    │   └─→ dora_process_manager.rs
    ├─→ audio_mixer.rs          (Export audio)
    ├─→ script_templates.rs     (Templates)
    └─→ recent_files.rs         (File history)
```

---

## Development Workflow

### 1. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes
# Edit source files in src/

# Test locally
cargo run -p mofa-studio

# Run tests
cargo test -p mofa-cast

# Commit changes
git add .
git commit -m "feat: description of your feature"
```

### 2. Code Review Checklist

Before submitting PR, verify:

- [ ] Code compiles without warnings
- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Formatted (`cargo fmt`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Tested manually with sample scripts

### 3. Testing Strategy

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transcript_parsing() {
        let parser = ParserFactory::new();
        let result = parser.parse_auto("Speaker: Hello");
        assert!(result.is_ok());
    }
}
```

**Integration Tests**:
```bash
# Test with sample scripts
./test_scripts.sh

# Manual testing checklist:
# 1. Import each format (txt, json, md)
# 2. Test all templates
# 3. Synthesize audio
# 4. Export to WAV and MP3
# 5. Test audio playback
```

---

## Code Organization

### Screen Architecture (Makepad)

The UI is built using Makepad's declarative live design:

```rust
// screen/design.rs
live_design! {
    CastScreen = <View> {
        header = <Header> { ... }
        main_content = <View> {
            left_panel = <View> { ... }
            right_panel = <View> {
                control_bar = <ControlBar> { ... }
                content_area = <View> { ... }
            }
        }
    }
}
```

**Key Patterns**:

1. **Separation of Concerns**:
   - `design.rs`: UI structure (live design)
   - `main.rs`: Event handlers and logic
   - Separate widgets into sub-views

2. **State Management**:
   ```rust
   #[rust]
   script: Option<String>,

   #[rust]
   is_synthesizing: bool,

   #[rust]
   collected_audio_segments: Vec<AudioSegmentInfo>,
   ```

3. **Event Handling**:
   ```rust
   fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
       // Button clicks
       if self.view.button(ids!(import_button)).clicked(actions) {
           self.handle_file_import(cx);
       }

       // Dropdown changes
       if let Some(id) = self.view.drop_down(ids!(format_dropdown)).selected(actions) {
           self.selected_format_id = id;
       }
   }
   ```

### Component Patterns

#### 1. Parser Pattern

```rust
// transcript_parser.rs
pub trait TranscriptParser {
    fn parse(&self, content: &str) -> Result<Transcript, ParseError>;
}

pub struct ParserFactory {
    parsers: HashMap<TranscriptFormat, Box<dyn TranscriptParser>>,
}
```

#### 2. TTS Abstraction

```rust
// tts_batch.rs
pub trait TtsEngine {
    fn synthesize(&self, text: &str, speaker: &str) -> Result<Vec<i16>, TtsError>;
}

pub enum TtsEngineWrapper {
    Kokoro(Box<KokoroTtsEngine>),
    Mock(Box<MockTtsEngine>),
}
```

#### 3. State Machine

```rust
// dora_integration.rs
enum DoraCommand {
    StartDataflow { dataflow_path: PathBuf },
    SendScriptSegments { segments: Vec<ScriptSegment> },
    StopDataflow,
}

enum DoraEvent {
    DataflowStarted { dataflow_id: String },
    AudioSegment { data: AudioData },
    Progress { current: usize, total: usize },
}
```

---

## Key Components

### 1. Transcript Parser

**Purpose**: Parse various transcript formats

**Formats Supported**:
- Plain Text: `Speaker: Message`
- JSON: Array of `{speaker, text, timestamp}`
- Markdown: `## Speaker` headers

**Key Functions**:
```rust
ParserFactory::new()              // Create parser
parser.parse_auto(content)        // Auto-detect format
parser.parse_with_format(content, format)  // Specific format
```

**Adding New Format**:
1. Implement `TranscriptParser` trait
2. Register in `ParserFactory`
3. Add to `TranscriptFormat` enum
4. Update tests

### 2. TTS Batch Synthesizer

**Purpose**: Batch TTS synthesis with multiple segments

**Architecture**:
```
Script → Segmenter → TTS Engine → Audio Segments
         (split)      (voice)       (collect)
```

**Key Configuration**:
```rust
pub struct TtsConfig {
    pub backend: TtsBackend,
    pub language: &'static str,
    pub voice: &'static str,
    pub speed: f32,
}
```

**Supported Backends**:
- `Kokoro`: Local multi-voice TTS (PrimeSpeech)
- `Mock`: Test tone generation (development)

### 3. Audio Mixer

**Purpose**: Combine audio segments with normalization

**Features**:
- Volume normalization (EBU R128)
- Silence insertion between segments
- WAV/MP3 export with metadata

**Usage**:
```rust
let config = MixerConfig {
    output_path: PathBuf::from("./output/podcast"),
    export_format: ExportFormat::Mp3,
    mp3_bitrate: Mp3Bitrate::Kbps192,
    normalize_dB: -14.0,
    silence_duration_secs: 0.5,
    ..Default::default()
};

let mixer = AudioMixer::new();
let result = mixer.mix(request)?;
```

### 4. Dora Integration

**Purpose**: Manage Dora dataflow lifecycle

**Key Functions**:
```rust
// Start dataflow
dora.start_dataflow(dataflow_path)

// Send script segments
dora.send_script_segments(segments)

// Poll for events
let events = dora.poll_events();
for event in events {
    match event {
        DoraEvent::AudioSegment { data } => {
            // Handle audio
        }
        DoraEvent::Progress { current, total } => {
            // Update UI
        }
    }
}
```

**Sequential Processing**:
- Segments sent one-by-one
- Wait for audio before sending next
- Prevents dataflow overload

---

## Adding Features

### Adding a New UI Component

**Example**: Add volume slider

1. **Add to design.rs**:
```rust
volume_slider = <Slider> {
    width: Fill,
    height: 24,
    min: 0.0,
    max: 100.0,
    value: 100.0,
    // ... styling
}
```

2. **Add state field**:
```rust
#[rust]
volume_level: f32,
```

3. **Handle events**:
```rust
if let Some(volume) = self.view.slider(ids!(volume_slider)).value() {
    self.volume_level = volume as f32;
    log::info!("Volume changed to: {}", volume);
}
```

### Adding a New Export Format

**Example**: Add AAC export

1. **Update audio_mixer.rs**:
```rust
pub enum ExportFormat {
    Wav,
    Mp3,
    Aac,  // NEW
}
```

2. **Implement conversion**:
```rust
fn write_aac_file(path: &Path, config: &MixerConfig, audio_data: &[u8]) -> Result<(), MixerError> {
    let temp_wav = path.with_extension("wav");
    Self::write_wav_file(&temp_wav, config, audio_data)?;

    let cmd = std::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(&temp_wav)
        .arg("-codec:a")
        .arg("aac")
        .arg("-b:a")
        .arg("128k")
        .arg(path)
        .output()?;

    Ok(())
}
```

3. **Update UI dropdown**:
```rust
// In handle_event
if let Some(format_id) = self.view.drop_down(ids!(export_format_dropdown)).selected(actions) {
    self.selected_export_format = format_id;  // 0=Wav, 1=Mp3, 2=Aac
}
```

### Adding a New Template

**Example**: Add "Debate" template

1. **Update script_templates.rs**:
```rust
pub enum TemplateType {
    TwoPersonInterview,
    ThreePersonDiscussion,
    Narrative,
    Debate,  // NEW
}

impl TemplateType {
    pub fn display_name(&self) -> &str {
        match self {
            TemplateType::Debate => "4-Person Debate",
            // ...
        }
    }
}
```

2. **Add template content**:
```rust
Template {
    template_type: TemplateType::Debate,
    name: "4-Person Debate",
    description: "Structured debate with moderator and 3 participants",
    content: r#"
## Moderator
Welcome to today's debate on [TOPIC].

## Proponent1
I believe [ARGUMENT 1].

## Opponent1
I disagree because [COUNTER-ARGUMENT 1].

## Proponent2
Adding to that, [ARGUMENT 2].

## Opponent2
However, [COUNTER-ARGUMENT 2].
"#.to_string(),
}
```

3. **Update UI dropdown**:
```rust
// template_dropdown in design.rs
let items = [
    "2-Person Interview",
    "3-Person Discussion",
    "Narrative",
    "4-Person Debate",  // NEW
];
```

---

## Testing

### Running Tests

```bash
# Unit tests
cargo test -p mofa-cast

# Integration tests
cargo test --test integration

# Test with logging
RUST_LOG=info cargo test -p mofa-cast
```

### Test Coverage

**Parser Tests**:
```rust
#[test]
fn test_parse_plain_text() {
    let content = "Speaker1: Hello\nSpeaker2: Hi";
    let parser = PlainTextParser::new();
    let result = parser.parse(content).unwrap();
    assert_eq!(result.messages.len(), 2);
}
```

**Mixer Tests**:
```rust
#[test]
fn test_audio_mixing() {
    let segments = create_test_segments();
    let mixer = AudioMixer::new();
    let result = mixer.mix(request).unwrap();
    assert!(result.output_file.exists());
}
```

### Manual Testing Checklist

**Import Tests**:
- [ ] Plain text import
- [ ] JSON import
- [ ] Markdown import
- [ ] Auto-detect format
- [ ] File dialog opens
- [ ] Parse error handling

**Synthesis Tests**:
- [ ] Single segment synthesis
- [ ] Multi-segment synthesis
- [ ] All voices work
- [ ] Voice routing correct
- [ ] Progress updates
- [ ] Error handling

**Export Tests**:
- [ ] WAV export
- [ ] MP3 export (with FFmpeg)
- [ ] Different bitrates
- [ ] Metadata correct
- [ ] File size reasonable

**Playback Tests**:
- [ ] Play button works
- [ ] Stop button works
- [ ] External player opens
- [ ] Status updates correct

---

## Build and Release

### Building for Production

```bash
# Optimized build
cargo build --release

# Run release binary
./target/release/mofa-studio
```

### Creating Release Package

```bash
# 1. Update version numbers
# Edit Cargo.toml: version = "0.6.4"

# 2. Update CHANGELOG
# Add new section for 0.6.4

# 3. Commit changes
git add .
git commit -m "chore: prepare release 0.6.4"

# 4. Create git tag
git tag v0.6.4
git push --tags

# 5. Build release binaries
cargo build --release

# 6. Package binaries
# macOS
tar -czf mofa-studio-macos-x64.tar.gz -C target/release mofa-studio

# Linux
tar -czf mofa-studio-linux-x64.tar.gz -C target/release mofa-studio
```

### Version Bump Checklist

- [ ] Update `Cargo.toml` version
- [ ] Update `CHANGELOG.md`
- [ ] Update `README.md` version
- [ ] Update `ARCHITECTURE.md` version
- [ ] Run full test suite
- [ ] Create release notes
- [ ] Tag and push

---

## Code Style Guidelines

### Rust Conventions

```rust
// 1. Use Result for error handling
fn parse_transcript(content: &str) -> Result<Transcript, ParseError> {
    // ...
}

// 2. Use Option for nullable values
pub struct Transcript {
    pub metadata: Option<Metadata>,
}

// 3. Prefer &str over String for parameters
fn process_text(text: &str) -> Result<(), Error> {
    // ...
}

// 4. Use builder pattern for complex structs
let config = MixerConfig::default()
    .with_output_path("./output")
    .with_format(ExportFormat::Mp3);

// 5. Log with appropriate levels
log::info!("User action: import script");
log::warn!("Unexpected format, falling back");
log::error!("Failed to parse: {}", error);
```

### Makepad UI Patterns

```rust
// 1. Use live_design! for UI structure
live_design! {
    MyScreen = <View> { ... }
}

// 2. Separate event handlers
fn handle_button_click(&mut self, cx: &mut Cx) {
    // Handle button logic
}

// 3. Update UI with IDs
self.view.label(ids!(status_label))
    .set_text(cx, "Status updated");

// 4. Redraw after changes
self.view.redraw(cx);
```

---

## Performance Considerations

### Memory Management

**Audio Data**:
```rust
// BAD: Keeps all audio in memory
let all_audio: Vec<Vec<i16>> = segments.iter()
    .map(|s| load_audio(s))
    .collect();

// GOOD: Stream to disk incrementally
for segment in segments {
    let audio = load_audio(&segment)?;
    save_to_disk(&audio)?;
}
```

**String Handling**:
```rust
// BAD: Excessive copying
let text = script.clone();
let processed = text.to_uppercase();
let result = processed.clone();

// GOOD: Borrow when possible
fn process(text: &str) -> String {
    text.to_uppercase()
}
```

### Async Operations

Dora integration uses message passing:

```rust
// Main thread
let (cmd_tx, cmd_rx) = bounded(100);
let (event_tx, event_rx) = bounded(100);

// Worker thread
thread::spawn(move || {
    loop {
        match cmd_rx.recv() {
            Ok(DoraCommand::StartDataflow) => {
                // Start dataflow
            }
            // ...
        }
    }
});

// Poll for events
while let Ok(event) = event_rx.try_recv() {
    handle_event(event);
}
```

---

## Debugging Tips

### Logging

```rust
// Structured logging
log::info!(
    "Audio synthesis complete: {} segments, {:.2}s total",
    segments.len(),
    total_duration
);

// Debug logging
log::debug!("Voice mapping: {:#?}", voice_mapping);

// Error logging
log::error!("Failed to export audio: {}", error);
```

### Common Issues

**1. UI not updating**:
```rust
// Make sure to redraw
self.view.redraw(cx);
```

**2. Events not firing**:
```rust
// Check if widget ID is correct
self.view.button(ids!(my_button)).clicked(actions)
```

**3. State not persisting**:
```rust
// Use #[rust] attribute
#[rust]
my_state: String,
```

---

## Contributing

### Pull Request Process

1. Fork repository
2. Create feature branch
3. Make changes
4. Add tests
5. Update documentation
6. Submit PR

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests pass
- [ ] Manual testing completed
- [ ] Documentation updated

## Checklist
- [ ] CHANGELOG.md updated
- [ ] Tests added/updated
- [ ] No new warnings
```

---

## Additional Resources

### Internal Documentation

- [Architecture](../ARCHITECTURE.md) - System design
- [User Guide](USER_GUIDE.md) - End-user documentation
- [Troubleshooting](TROUBLESHOOTING.md) - Issue resolution
- [History](HISTORY.md) - Development history

### External Dependencies

- [Makepad](https://github.com/makepad/makepad) - UI framework
- [Dora](https://github.com/dora-rs/dora) - Dataflow system
- [PrimeSpeech](https://github.com/OpenLanguage-model/CosyVoice2) - TTS engine

### Tools

- `cargo-expand` - Macro expansion
- `cargo-tree` - Dependency tree
- `cargo-outdated` - Check for updates

---

**Last Updated**: 2026-01-17
**Version**: 0.6.3

---

## Quick Reference

```bash
# Development
cargo run -p mofa-studio        # Run app
cargo test -p mofa-cast           # Run tests
cargo clippy -p mofa-cast         # Lint
cargo fmt -p mofa-cast            # Format

# Dora
dora start dataflow/multi-voice-batch-tts.yml
dora ps
dora logs mofa-cast-multi-voice
dora destroy mofa-cast-multi-voice

# Testing
./test_scripts.sh                 # Integration tests
RUST_LOG=debug cargo run         # Verbose logging
```
