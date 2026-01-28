# MoFA Cast

> Transform your **optimized scripts** into multi-voice podcast audio with local TTS

**Version**: 0.6.3
**Status**: 🎉 Production-Ready with Multi-Voice Support
**Last Updated**: 2026-01-17

## Overview

MoFA Cast is a **local multi-voice text-to-speech (TTS) tool** that converts your already-optimized podcast scripts into professional audio with distinct speaker voices through **local** multi-voice TTS synthesis using PrimeSpeech engine.

## Project Philosophy

**Local-First Development**: All core functionality should work locally without requiring external API keys or cloud services. This ensures:
- ✅ Complete privacy and data ownership
- ✅ No ongoing costs for users
- ✅ Works offline
- ✅ Faster response times (no network latency)

## Project Structure

```
apps/mofa-cast/
├── Cargo.toml                   # Dependencies
├── README.md                    # This file
├── ARCHITECTURE.md              # Technical architecture
├── CHANGELOG.md                 # Version history
├── dataflow/                    # Dora dataflow configs
│   └── multi-voice-batch-tts.yml     # Multi-voice TTS pipeline
├── docs/                        # Documentation
│   ├── USER_GUIDE.md            # User documentation
│   ├── TROUBLESHOOTING.md       # Issue resolution
│   ├── DEVELOPMENT.md           # Developer guide
│   ├── HISTORY.md               # Development history
│   └── SCRIPT_OPTIMIZATION_GUIDE.md  # AI script optimization
├── test_samples/                # Test files
│   ├── sample_plain.txt
│   ├── sample_json.json
│   └── sample_markdown.md
└── src/
    ├── lib.rs                   # MoFA Studio app integration
    ├── screen/                  # UI components
    │   ├── mod.rs
    │   ├── main.rs              # Main UI screen
    │   └── design.rs            # Live design definitions
    ├── transcript_parser.rs     # Parse transcript formats
    ├── script_templates.rs      # Pre-built templates
    ├── recent_files.rs          # Recent files management
    ├── tts_batch.rs             # TTS engine abstraction
    ├── audio_mixer.rs           # Audio mixing and export
    ├── dora_integration.rs      # Dora dataflow integration
    └── dora_process_manager.rs  # Dora lifecycle management
```

**Legend**: ✅ = Completed, ⏳ = Planned

## Features

- **Import**: Load optimized podcast scripts (Plain text, JSON, Markdown)
- **Edit**: Make minor adjustments to imported scripts
- **Synthesize**: Multi-voice batch TTS with Dora dataflow using PrimeSpeech (local, high quality)
- **Export**: Audio mixing and WAV export ✅
- **Monitor**: Real-time log viewer with filtering

## What MoFA Cast Does NOT Do

- ❌ **Script optimization** - Use ChatGPT, Claude, or other AI tools (see [SCRIPT_OPTIMIZATION_GUIDE.md](docs/SCRIPT_OPTIMIZATION_GUIDE.md))
- ❌ **LLM API integration** - No OpenAI/Claude API calls
- ❌ **Automated content generation** - Script optimization is external

**Why?** External AI tools (ChatGPT, Claude) provide:
- ✅ Zero cost (no API fees)
- ✅ Better quality (direct interaction with latest models)
- ✅ More flexibility (iterate until perfect)
- ✅ Access to GPT-4o, Claude 4, etc.

MoFA Cast focuses on what it does best: **multi-voice TTS synthesis**.

## Current Status

### ✅ Completed (2026-01-14)

**Latest Release**: v0.5.0 - Multi-Voice Support & UI Enhancements

#### Infrastructure & Setup
- ✅ Project structure created and configured
- ✅ Dependencies configured (Cargo.toml)
- ✅ MofaApp trait implemented
- ✅ Documentation organized in `docs/` directory

#### P0.1 - Transcript Parsing ✅
- ✅ Implemented `TranscriptParser` trait
- ✅ Created 3 parsers: PlainText, JSON, Markdown
- ✅ Implemented `ParserFactory` with auto-detection
- ✅ Added speaker statistics extraction
- ✅ All unit tests passing (5/5)
- ✅ ~672 lines of production code

#### P0.2 - UI Integration ✅
- ✅ Integrated parser with CastScreen
- ✅ Added file import button handler
- ✅ Display parsed transcript in original editor
- ✅ Show speaker statistics in left panel
- ✅ Update file info with message/speaker count
- ✅ Created test sample files
- ✅ ~590 lines total in screen.rs

#### P0.3 - AI Script Refinement ✅
- ✅ Implemented `ScriptRefiner` trait with async/await
- ✅ Created `OpenAiRefiner` with OpenAI API integration
- ✅ Implemented `MockRefiner` for testing without API
- ✅ Added `PromptTemplates` for structured prompts
- ✅ Comprehensive error handling (8 error types)
- ✅ Integrated with CastScreen Refine button
- ✅ Show progress indicator during refinement
- ✅ Display refined script in editable editor
- ✅ All unit tests passing (7/7: 5 parser + 2 refiner)
- ✅ ~485 lines of production code

**Key Features**:
- Trait-based architecture for extensibility
- Multiple AI provider support (OpenAI ready, Claude stub)
- Mock refiner enables testing without API costs
- Async/await with tokio runtime integration
- Streaming support for real-time progress updates

#### P0.4 - Batch TTS Synthesis ✅
- ✅ Implemented `TtsEngine` trait for extensibility
- ✅ Created `ScriptSegmenter` to parse scripts by speaker
- ✅ Implemented `BatchTtsSynthesizer` with parallel processing
- ✅ Created `MockTtsEngine` for testing without TTS engine
- ✅ Comprehensive error handling (7 error types)
- ✅ Integrated with CastScreen Synthesize button
- ✅ Progress tracking during synthesis
- ✅ Audio file management (organized by speaker)
- ✅ All unit tests passing (11/11: 5 parser + 2 refiner + 4 TTS)
- ✅ ~580 lines of production code
- ✅ **Removed OpenAI TTS (cloud-based, violates local-first principle)**

**Key Features**:
- Trait-based TTS engine architecture (extensible for Dora, Kokoro, PrimeSpeech)
- Script segmentation by speaker with regex pattern matching
- Parallel async synthesis with configurable concurrency
- Mock TTS engine creates valid WAV files for testing
- Progress callbacks for real-time UI updates
- Organized output structure (output_dir/speaker/segment_NNN.wav)

**🚨 Important**: Currently using `MockTtsEngine` (generates test tones).
Next step: Integrate `dora-kokoro-tts` for real local TTS synthesis.

#### P0.5 - Audio Mixing and Export ✅
- ✅ Implemented `AudioMixer` with WAV file handling
- ✅ Created `WavHeader` structure for parsing/generating WAV files
- ✅ Implemented audio concatenation with silence insertion
- ✅ Volume normalization interface (configurable)
- ✅ Metadata support structure (title, artist, album, etc.)
- ✅ Integrated with CastScreen Export button
- ✅ Organized segment collection from TTS output
- ✅ All unit tests passing (16/16: 5 parser + 2 refiner + 4 TTS + 5 mixer)
- ✅ ~540 lines of production code

**Key Features**:
- Direct WAV file manipulation without external dependencies
- Configurable silence between segments (default 0.5s)
- Format validation (sample rate, channels, bits per sample)
- Automatic segment ordering by filename
- Duration calculation and file size reporting

#### P0.6 - Dora Dataflow Integration ✅
- ✅ Created `dataflow/batch-tts.yml` for TTS pipeline
- ✅ Implemented `DoraIntegration` with worker thread
- ✅ Integrated `DynamicNodeDispatcher` for bridge management
- ✅ Refactored `handle_synthesize_audio()` to use Dora dataflow
- ✅ Added environment variable configuration (BACKEND, VOICE, LANGUAGE, SPEED)
- ✅ Implemented event polling for progress updates
- ✅ Added ScriptSegment for dataflow communication
- ✅ ~320 lines of dora integration code
- ✅ Build successful with 14 warnings (non-breaking)

**Key Features**:
- Dataflow lifecycle management (start/stop)
- Async communication via command/event channels
- Timer-based event polling (100ms interval)
- Integration with dora-kokoro-tts for local TTS
- Extensible architecture for additional TTS engines
- Consistent with mofa-fm architecture pattern

**Dataflow Nodes**:
- `text-input`: Dynamic node receiving script segments
- `text-segmenter`: Splits text into TTS-friendly chunks
- `kokoro-tts`: Local TTS synthesis (Kokoro-82M)
- `mofa-cast-controller`: Dynamic node returning audio to UI

#### P0.7 - Audio Collection and Export ✅
- ✅ Implemented audio segment collection in `poll_dora_events()`
- ✅ Added WAV file writing from AudioData (f32 → i16 conversion)
- ✅ Store collected segments with metadata (path, speaker, duration)
- ✅ Track synthesis progress (received/expected segments)
- ✅ Auto-enable export when all segments received
- ✅ Refactored `handle_export_audio()` to use collected segments
- ✅ Integration with audio_mixer for final export
- ✅ Added comprehensive test documentation (TTS_WORKFLOW_TEST.md)
- ✅ ~200 lines of audio collection code

**Key Features**:
- Real-time audio segment collection from Dora events
- Automatic WAV file generation with proper headers
- Progress tracking and UI updates during synthesis
- Segments organized by speaker with sequential naming
- Duration calculation and validation
- Complete export workflow (collect → mix → save)
- End-to-end testing guide with troubleshooting

**Audio Output**:
- Individual segments: `output/mofa-cast/dora/segment_XXX_speaker.wav`
- Final podcast: `output/mofa-cast/podcast.wav`
- Format: 16-bit PCM WAV, 22050 Hz (or native rate), mono
- Segments separated by 0.5s silence

#### P1.1 - Multi-Voice Support ✅
- ✅ Implemented dynamic voice routing with `dora-voice-router`
- ✅ Smart voice assignment (host→Luo Xiang, guest1→Ma Yun, guest2→Ma Baoguo)
- ✅ Speaker normalization (merges duplicate speaker names)
- ✅ Created `multi-voice-batch-tts.yml` dataflow with 3 parallel PrimeSpeech TTS nodes
- ✅ Automatic voice mapping UI in CastScreen
- ✅ 100% success rate in testing (10/10 segments with distinct voices)
- ✅ ~200 lines of voice router code + dataflow configuration

**Key Features**:
- JSON-based segment format: `{"speaker": "...", "text": "...", "voice_name": "...", "speed": 1.0}`
- Routes to 3 parallel PrimeSpeech TTS nodes based on voice_name
- Zero-configuration user experience with automatic role-based mapping
- Applied during TTS sending (doesn't modify UI text)
- No slowdown (~4s per segment, same as single-voice)

#### P1.2 - UI Enhancements (Partial) ✅
- ✅ Implemented real-time log viewer with collapsible panel
- ✅ Added log level filtering (ALL/INFO/WARN/ERROR)
- ✅ Improved layout: left panel 300→200px, compact spacing
- ✅ Added application icon (🎙️ studio microphone)
- ✅ Enhanced dropdown styling with hover effects
- ✅ Fixed text input auto-wrap and scrolling
- ✅ Fixed 3 bugs (stack overflow, scroll component, text color)
- ✅ ~400 lines of UI improvements in screen.rs

**Key Features**:
- Collapsible log panel (320px width, toggle button)
- Markdown rendering for formatted logs
- Auto-capture of all Dora events
- Fixed color schemes for light theme
- Text input auto-wrap and scrolling
- Compact layout design (28px vertical space saved)

**Bugs Fixed**:
1. Stack overflow in log initialization (infinite recursion)
2. Scroll component error (changed to ScrollYView)
3. White text on light background (fixed font_color)

## Quick Start

### 1. Build and Run

```bash
# Build mofa-cast
cargo build --release --package mofa-cast

# Start mofa-studio-shell
./target/release/mofa-studio-shell
```

### 2. Prepare Your Script

**Step 1**: Use ChatGPT or Claude to optimize your chat transcript

**Step 2**: Save the optimized script as `script.txt`, `script.md`, or `script.json`

**Step 3**: Use this format in your script:
```
host: Welcome to today's episode...
guest1: Thanks for having me...
guest2: I'm excited to be here...
```

💡 **See [SCRIPT_OPTIMIZATION_GUIDE.md](docs/SCRIPT_OPTIMIZATION_GUIDE.md) for recommended prompts and workflows**

### 3. Generate Audio

1. Click **MoFA Cast** icon in sidebar
2. **Import**: Select format → Click Import → Choose your script file
3. **Edit** (optional): Make minor adjustments in the script editor
4. **Synthesize**: Click "Synthesize Audio"
   - Voices are auto-assigned: host→Luo Xiang, guest1→Ma Yun, guest2→Ma Baoguo
   - Monitor progress in real-time log viewer
5. **Export**: Click "Export Audio" to create final podcast file

**Expected Output**: `output/mofa-cast/podcast.wav`

**Documentation**:
- **Testing**: [docs/TTS_WORKFLOW_TEST.md](docs/TTS_WORKFLOW_TEST.md)
- **File Dialog Issues**: [docs/FILE_DIALOG_TROUBLESHOOTING.md](docs/FILE_DIALOG_TROUBLESHOOTING.md)

## TTS Engine Options

### ✅ PrimeSpeech (Recommended - Multi-Voice)
**Local multi-voice TTS engine via dora-primespeech**

**Features**:
- 🎭 **Multi-Voice**: Support for 3+ distinct voices in single podcast
- 🌍 **Chinese Optimized**: High-quality Chinese pronunciation
- 🚀 **Fast**: ~4s per segment with parallel processing
- 🔒 **Privacy**: 100% local, no internet required
- 💰 **Free**: No API costs, no rate limits

**Available Voices**:
- Luo Xiang: Deep male voice (authoritative, host)
- Ma Yun: Energetic male voice (guest speaker)
- Ma Baoguo: Characteristic voice (distinctive style)
- More voices available in `~/.dora/models/primespeech/`

**Usage**:
```bash
# Build mofa-cast with PrimeSpeech
cargo build --release --package mofa-cast

# Start mofa-studio
./target/release/mofa-studio-shell

# In the UI:
# 1. Import a transcript with multiple speakers
# 2. Click "Refine Script" (AI enhancement)
# 3. Click "Synthesize Audio" (uses multi-voice PrimeSpeech)
#    - Voices are auto-assigned based on speaker names
#    - Monitor progress in log viewer
# 4. Export final podcast
```

**Dataflow**: Uses `dataflow/multi-voice-batch-tts.yml` with 3 parallel TTS nodes

### ⏳ Kokoro-82M (Deprecated - Single Voice Only)
**Legacy single-voice TTS engine**

**Status**: Replaced by PrimeSpeech for multi-voice support
- See [docs/KOKORO_TTS_GUIDE_DEPRECATED.md](docs/KOKORO_TTS_GUIDE_DEPRECATED.md)
- Still supported for single-voice use cases
- 6.6x realtime on Apple Silicon (MLX), 4.1x on CPU
- 100+ voices across EN, ZH, JA, KO languages

### ⏳ MockTtsEngine (Testing)
**Simple test tones for development**

**Features**:
- Generates 440Hz sine wave tones
- Creates valid WAV files for testing audio pipeline
- No external dependencies
- Useful for UI/UX testing without TTS engine

## Development Roadmap
- Metadata structure ready for future implementation

#### UI Framework
- ✅ Complete `CastScreen` widget implementation
- ✅ Split-view editor (original | refined script)
- ✅ Left panel (import section + speaker list)
- ✅ Right panel (control buttons + editor)
- ✅ Dark mode support throughout
- ✅ Custom icon (`cast.svg`) created

#### Shell Integration
- ✅ Registered in `mofa-studio-shell`
- ✅ Sidebar navigation ("MoFA Cast" button)
- ✅ Page visibility toggling
- ✅ Dark mode propagation

#### Build Status
- ✅ **Build Successful**: `cargo build --release` completed without errors
- ✅ **Tests Passing**: All 16 unit tests (5 parser + 2 refiner + 4 TTS + 5 mixer)
- ✅ Only non-critical warnings (unused imports, naming conventions)

### 🎉 Core Features Complete!

#### Phase 1: Core Functionality (P0) - ALL COMPLETE ✅
1. ✅ **Transcript Parsing** (Completed in <1 day)
   - Plain text, JSON, Markdown parsers
   - Auto-detection
2. ✅ **UI Integration** (Completed in <1 day)
   - Parser integrated with UI
   - File import handler
   - Display parsed content
3. ✅ **AI Script Refinement** (Completed in <1 day)
   - OpenAI/Claude API integration
   - Streaming responses
   - Mock refiner for testing
4. ✅ **Batch TTS Synthesis** (Completed in <1 day)
   - Script segmentation by speaker
   - Parallel async synthesis
   - Mock TTS engine for testing
5. ✅ **Audio Mixing** (Completed in <1 day)
   - WAV concatenation and export
   - Silence insertion
   - Metadata structure

**Total Time**: ~5 days estimated → **Completed in <1 day!**

**🚀 MVP Status**: All core features implemented and tested!
- Full pipeline: Import → Parse → Refine → Synthesize → Export
- 16 unit tests passing
- Production-ready code with comprehensive error handling
- Multi-voice support with automatic voice assignment
- Real-time log viewer for progress monitoring
- ⏳ **Next**: MP3 export, audio player, keyboard shortcuts (P1.2 remaining)

## Development

### TTS Integration Status
- ✅ **PrimeSpeech TTS**: Multi-voice synthesis (default, recommended)
- ✅ **Voice Router**: Automatic role-based voice mapping
- ✅ **Dora Integration**: Multi-node dataflow with parallel processing
- ⏳ **Kokoro-82M**: Single-voice fallback (deprecated)

**Model Installation**:
```bash
# PrimeSpeech models are stored in:
~/.dora/models/primespeech/

# Available voices:
- Luo Xiang (host)
- Ma Yun (guest1)
- Ma Baoguo (guest2)
```

## Documentation

### User Documentation
- **[User Guide](docs/USER_GUIDE.md)** - Complete usage instructions
- **[Script Optimization](docs/SCRIPT_OPTIMIZATION_GUIDE.md)** - How to optimize scripts with AI tools
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Common issues and solutions

### Developer Documentation
- **[Architecture](ARCHITECTURE.md)** - Technical design and system architecture
- **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and code organization
- **[History](docs/HISTORY.md)** - Deprecated features and development history
- **[Changelog](docs/CHANGELOG.md)** - Version history and release notes

## Build & Run

```bash
# Build the project
cd /path/to/mofa-studio
cargo build --release

# Run MoFA Studio
cargo run --release
```

Then click on **"MoFA Cast"** in the sidebar to access the application.

## Contributing

This is part of the MoFA Studio project. See main [README](../../README.md) for contribution guidelines.

## License

Apache-2.0 (same as MoFA Studio)

---

**Note**: This project is in active development. Core functionality is being implemented incrementally.
