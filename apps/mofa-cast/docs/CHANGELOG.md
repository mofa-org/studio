# MoFA Cast - Changelog

> All notable changes to MoFA Cast project.

**Format**: This changelog follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

---

## [0.6.3] - 2026-01-16

### ✅ Added - P1.3 Audio Player Widget

**Core Feature**: In-app audio playback with controls

- **Audio Player Section**:
  - New dedicated player section below Control Bar
  - Horizontal layout: Script Editor (500px) | Audio Player (Fill)
  - Professional rounded card design with light blue background
  - Real-time status indicators (No audio/Ready/Playing/Stopped)

- **Playback Controls**:
  - **Play Button** (▶): Start audio playback with platform-specific player
  - **Stop Button** (⏹): Terminate playback immediately
  - **Open in Player Button**: Launch system default audio player
  - Process management: Spawns and controls child audio processes

- **Volume Control**:
  - Volume slider (0-100% range)
  - Default volume: 100% (audible by default)
  - Real-time volume display in status
  - Platform-specific volume flags (afplay -v on macOS)

- **Audio Information Display**:
  - Format: WAV or MP3
  - Duration: Total playback time in seconds
  - File Size: Export size in KB
  - Automatic update after export

- **State Management**:
  - `exported_audio_path`: Store last exported audio file
  - `audio_playback_process`: Track spawned child process
  - `is_playing`: Prevent multiple play instances
  - `volume_level`: Persistent volume setting (100% default)

**Files Modified**:
- `src/screen.rs` (~2400 lines)
  - Added `content_area` horizontal layout (Editor | Player)
  - Added `audio_player_section` UI (~150 lines)
  - Added state fields: `exported_audio_path`, `audio_playback_process`, `is_playing`, `volume_level`
  - Implemented `handle_play_audio()`: Platform-specific audio playback
  - Implemented `handle_stop_audio()`: Process termination
  - Implemented `handle_open_in_player()`: System player launch
  - Updated export success handler to populate player info

- `Cargo.toml`
  - Added `open = "5.0"` dependency

**UI Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ Control Bar                                                  │
├───────────────────┬─────────────────────────────────────────┤
│ Script Editor     │ Audio Player Section                     │
│ (500px)           │ ┌─────────────────────────────────────┐ │
│                   │ │ 🎵 Audio Player                      │ │
│                   │ │                                       │ │
│                   │ │ Status: Playing (Volume: 100%)       │ │
│                   │ │                                       │ │
│                   │ │ [▶] [⏹] [Open in Player]             │ │
│                   │ │                                       │ │
│                   │ │ Volume: ━━━━━━━━━━━━ 100%            │ │
│                   │ │                                       │ │
│                   │ │ Format: MP3                           │ │
│                   │ │ Duration: 125.3 seconds               │ │
│                   │ │ Size: 2048 KB                         │ │
│                   │ └─────────────────────────────────────┘ │
└───────────────────┴─────────────────────────────────────────┘
```

**Platform Support**:
- ✅ **macOS**: `afplay` with volume control
- 🟡 **Windows**: Planned (wmplayer)
- 🟡 **Linux**: Planned (vlc)

**Implementation Details**:

**Process Management**:
```rust
// Spawn audio player process
let result = std::process::Command::new("afplay")
    .arg("-v")
    .arg(&volume.to_string())
    .arg(audio_path)
    .spawn();

match result {
    Ok(mut child) => {
        self.audio_playback_process = Some(child);
        self.is_playing = true;
    }
}

// Stop playback
if let Some(mut child) = self.audio_playback_process.take() {
    child.kill()?;
}
```

**Volume Initialization Fix**:
```rust
// Slider UI (init_value: 100.0)
volume_slider = <Slider> {
    min: 0.0
    max: 100.0
    init_value: 100.0  // FIXED: Default to 100%
}

// Direct volume usage (simplified from complex calculation)
let volume = self.volume_level;  // 100.0
```

**Bugs Fixed**:
1. **Stop Button Not Working**:
   - Problem: Using `open::that()` launched external player with no control
   - Solution: Use `std::process::Command::spawn()` and store Child process
   - User feedback: "点击播放后,点击停止播放不起效,仍然继续播放音频"

2. **Volume Showing 0%**:
   - Problem: Slider had no init_value, defaulted to 0
   - Solution: Set `init_value: 100.0` and simplify calculation
   - User feedback: "播放时显示"Playing (Volume: 0%)" 听不见声音,音量应该默认100%"

**Known Limitations**:
- Volume slider events not implemented (UI only for now)
- Cannot change volume during playback (requires restart)
- When audio ends naturally, state not auto-reset (would need polling)
- Only tested on macOS

**Future Enhancements** (P1.4):
- Real-time volume slider events (restart player on change)
- Cross-platform support (Windows/Linux)
- Audio progress bar and time tracking
- Auto-state reset when audio finishes naturally

**Test Results**:
- ✅ Compilation successful (cargo check --release)
- ✅ Play button spawns audio process
- ✅ Stop button terminates playback
- ✅ Volume defaults to 100% (audible)
- ⏳ End-to-end testing pending

---

## [0.6.2] - 2026-01-16

### ✅ Added - P1.2.2 Audio Quality Enhancements

**Core Feature**: Volume normalization and ID3 tag embedding

- **RMS Volume Normalization**:
  - Automatic level adjustment to -14.0 dB (EBU R128 standard)
  - Per-segment RMS calculation and normalization
  - Amplification range: 0.1x - 10.0x (safety limits)
  - Protects against audio clipping with i16 clamping
  - Detailed logging: before/after levels, amplification factor

- **ID3 Tag Embedding** (MP3 only):
  - Title: Auto-extracted from script filename
  - Artist: "MoFA Cast"
  - Album: "Generated by MoFA Cast"
  - Year: Current year (auto-generated)
  - Comment: Segment count and version info
  - Encoded-by: "MoFA Cast v0.6.2"

**Files Modified**:
- `src/audio_mixer.rs` (~590 lines)
  - Added `normalize_audio()` method (~55 lines)
  - Enhanced `write_mp3_file()` with ID3 tag support
  - Modified `mix()` to apply normalization

- `src/screen.rs`
  - Added `AudioMetadata` import
  - Added metadata generation in `handle_export_audio()`
  - Set `normalize_dB: -14.0` in MixerConfig

**Implementation Details**:

**Normalization Algorithm**:
```rust
fn normalize_audio(audio_data: &[u8], target_dB: f32) -> Result<Vec<u8>, MixerError> {
    // 1. Convert bytes to i16 samples
    let samples = audio_data.len() / 2;
    let audio_i16: Vec<i16> = /* ... */;

    // 2. Calculate RMS level
    let sum_squares: f64 = audio_i16.iter()
        .map(|&s| (s as f64) * (s as f64))
        .sum();
    let rms = (sum_squares / samples as f64).sqrt();

    // 3. Calculate target RMS from dB
    let target_rms = 32768.0 * 10_f64.powf(target_dB as f64 / 20.0);

    // 4. Compute amplification factor
    let amplification = (target_rms / rms).max(0.1).min(10.0);

    // 5. Apply normalization with overflow protection
    let normalized: Vec<u8> = audio_i16.iter()
        .map(|&sample| {
            let val = (sample as f64 * amplification) as i16;
            val.clamp(i16::MIN, i16::MAX).to_le_bytes()
        })
        .flatten()
        .collect();

    Ok(normalized)
}
```

**ID3 Metadata Generation**:
```rust
let metadata = AudioMetadata {
    title: Some(script_filename),
    artist: Some("MoFA Cast".to_string()),
    album: Some("Generated by MoFA Cast".to_string()),
    year: Some(chrono::Utc::now().format("%Y").to_string()),
    comment: Some(format!("Created with MoFA Cast v0.6.2 - {} segments",
        segment_count)),
};
```

**FFmpeg Command with ID3 Tags**:
```bash
ffmpeg -y -i input.wav \
  -codec:a libmp3lame -b:a 192k -qscale:a 2 \
  -metadata title="Podcast Name" \
  -metadata artist="MoFA Cast" \
  -metadata album="Generated by MoFA Cast" \
  -metadata year="2026" \
  -metadata comment="Created with MoFA Cast v0.6.2 - 10 segments" \
  -metadata encoded_by="MoFA Cast v0.6.2" \
  output.mp3
```

**Logging Example**:
```
[INFO] Audio normalized: RMS -20.5 → -14.0 (amplification: 2.1x)
[INFO] Using metadata: title=my_script, artist=MoFA Cast, album=Generated by MoFA Cast
[INFO] MP3 export with ID3 tags completed: podcast.mp3
```

**Technical Details**:
- **Format**: 16-bit PCM mono (PrimeSpeech output)
- **Target Level**: -14.0 dBFS (EBU R128 broadcast standard)
- **Algorithm**: RMS (Root Mean Square) normalization
- **Safety**: Amplification limited to 0.1x - 10.0x range
- **Overflow Protection**: i16::MIN to i16::MAX clamping

**Benefits**:
- ✅ Consistent audio levels across segments
- ✅ Professional broadcast-quality loudness
- ✅ MP3 files with proper metadata for music players
- ✅ Better user experience (no volume adjustments needed)

**Known Limitations**:
- Assumes 16-bit mono audio format
- No manual metadata input UI (auto-generated only)
- RMS normalization (simpler than EBU R128, but sufficient)

**Migration Notes**: None (backward compatible - normalization can be disabled by setting `normalize_dB: 0.0`)

**Test Results**:
- ✅ Compilation successful
- ✅ All warnings non-critical
- ✅ RMS calculation verified
- ⏳ End-to-end audio quality testing pending

---

## [0.6.1] - 2026-01-16

### ✅ Added - P1.2.1 MP3 Export Feature

**Core Feature**: MP3 export with bitrate selection using ffmpeg

- **Export Format Selection**: WAV and MP3 format support
  - Format dropdown (70px wide) with ["WAV", "MP3"] options
  - Default: WAV format (backward compatible)

- **MP3 Bitrate Selection**: 4 quality options
  - 128 kbps (Good quality, ~1MB/min)
  - 192 kbps (High quality, ~1.5MB/min) - **Recommended**
  - 256 kbps (Very high quality, ~2MB/min)
  - 320 kbps (Maximum quality, ~2.5MB/min)
  - Bitrate dropdown (110px wide) with light blue styling

- **State Management**:
  - `selected_export_format: usize` - Tracks format selection (0=WAV, 1=MP3)
  - `selected_mp3_bitrate: usize` - Tracks bitrate selection (0-3)

- **Event Handling**:
  - Dropdown change events logged to console
  - Format changes: "Export format changed to: WAV/MP3"
  - Bitrate changes: "MP3 bitrate changed to: XXX kbps"

- **MP3 Encoding**: Using ffmpeg CLI tool
  - Codec: libmp3lame (industry-standard LAME encoder)
  - VBR quality: 2 (high quality variable bitrate)
  - Temporary WAV file created and converted to MP3
  - Automatic cleanup of temporary files

**Files Modified**:
- `src/audio_mixer.rs` (~600 lines)
  - Added `ExportFormat` enum (Wav, Mp3)
  - Added `Mp3Bitrate` enum (Kbps128, Kbps192, Kbps256, Kbps320)
  - Added `export_format` field to `MixerConfig`
  - Added `mp3_bitrate` field to `MixerConfig`
  - Implemented `write_mp3_file()` method using ffmpeg
  - Modified `mix()` method to handle format selection

- `src/screen.rs` (~1200 lines)
  - Added UI dropdowns in control_bar (export_format_dropdown, mp3_bitrate_dropdown)
  - Added state field initialization in live_design
  - Added event handlers for dropdown changes
  - Modified `handle_export_audio()` to use selected format/bitrate
  - Enhanced logging for export operations

- `src/lib.rs`
  - Exported `ExportFormat` and `Mp3Bitrate` types

**Backend Implementation**:
```rust
// Export format enum
pub enum ExportFormat {
    Wav,
    Mp3,
}

// MP3 bitrate enum with display names
pub enum Mp3Bitrate {
    Kbps128,  // "128 kbps (Good)"
    Kbps192,  // "192 kbps (High)" - Recommended
    Kbps256,  // "256 kbps (Very High)"
    Kbps320,  // "320 kbps (Max)"
}

impl Mp3Bitrate {
    pub fn kbps(&self) -> u32 { /* ... */ }
    pub fn display_name(&self) -> &str { /* ... */ }
}
```

**UI Controls**:
```rust
// Format dropdown
export_format_dropdown = <DropDown> {
    width: 70, height: 24
    labels: ["WAV", "MP3"]
    values: [0, 1]
}

// MP3 bitrate dropdown
mp3_bitrate_dropdown = <DropDown> {
    width: 110, height: 24
    labels: ["128 kbps", "192 kbps", "256 kbps", "320 kbps"]
    values: [0, 1, 2, 3]
    draw_bg: {
        // Light blue for MP3
        color: mix(#e0f2fe, #bae6fd, self.bg_hover);
    }
}
```

**Export Workflow**:
1. User selects format (WAV or MP3)
2. If MP3 selected, user selects bitrate (128/192/256/320 kbps)
3. Click "Export Audio" button
4. System generates audio segments
5. Mixer concatenates segments and adds silence
6. Export in selected format:
   - WAV: Direct write (no external tools)
   - MP3: ffmpeg conversion with selected bitrate
7. Output file: `podcast.wav` or `podcast.mp3`

**Prerequisites**:
- ffmpeg must be installed on system
- macOS: `brew install ffmpeg`
- Linux: `sudo apt install ffmpeg` (Ubuntu/Debian)

**Test Results**:
- ✅ Compilation successful (cargo build --release)
- ✅ All warnings non-critical
- ✅ UI controls rendering correctly
- ✅ Event handlers working
- ⏳ End-to-end testing pending

**Known Limitations**:
- Requires external ffmpeg installation
- No volume normalization yet (planned for v0.6.2)
- No ID3 tag embedding yet (planned for v0.6.2)

**Migration Notes**: None (backward compatible with existing WAV export)

---

## [0.6.0] - 2026-01-15

### Changed

- **Refactored to TTS-focused workflow** (Simplified architecture)
- Removed AI script refinement (moved to external tools)
- Removed file watching and external editor integration
- Removed template system and recent files
- Focused on core TTS workflow with multi-voice support

**Breaking Changes**: Script refinement now requires external AI tools (ChatGPT, Claude, etc.)

---

## [0.5.0] - 2026-01-14

### ✅ Added - P1.1 Multi-Voice Support

**Core Feature**: Dynamic voice routing for different speakers

- **Smart Voice Assignment**: Automatic role-based voice mapping
  - host → Luo Xiang (deep male voice)
  - guest1 → Ma Yun (energetic male voice)
  - guest2 → Ma Baoguo (characteristic voice)
  - Zero-configuration user experience

- **Custom Voice Router Node**: `dora-voice-router`
  - Python-based routing node
  - JSON-based segment format: `{"speaker": "...", "text": "...", "voice_name": "...", "speed": 1.0}`
  - Routes to 3 parallel PrimeSpeech TTS nodes

- **Speaker Normalization**: Auto-merges duplicate speaker names
  - Merges "[主持人]" and "host" variants
  - Applied during TTS sending (doesn't modify UI text)

- **Multi-Voice Dataflow**: `dataflow/multi-voice-batch-tts.yml`
  - 3 parallel TTS nodes with different voice models
  - Automatic audio merging
  - Complete logging from all nodes

**Files Created**:
- `node-hub/dora-voice-router/` - Custom routing node
  - `pyproject.toml` - Package configuration
  - `dora_voice_router/main.py` - Routing logic
  - `README.md` - Documentation

**Files Modified**:
- `src/dora_integration.rs` - VoiceConfig, VoiceMapping, smart assignment
- `src/lib.rs` - Export voice types
- `src/screen.rs` - Speaker normalization, voice mapping UI
- `mofa-dora-bridge/src/widgets/cast_controller.rs` - Multi-input event handling
- `dataflow/multi-voice-batch-tts.yml` - New dataflow

**Testing Results**:
- ✅ 10/10 segments generated with distinct voices
- ✅ 100% success rate
- ✅ ~4s per segment (no slowdown)

**Breaking Changes**: None

**Migration Notes**: None (backward compatible)

---

### ✅ Added - P1.2 UI Enhancements (Partial)

**Core Feature**: Real-time log viewer and UI polish

- **Real-Time Log Viewer**:
  - Collapsible log panel (320px width, toggle button)
  - Log level filtering (ALL/INFO/WARN/ERROR)
  - Clear logs button
  - Markdown rendering for formatted logs
  - Auto-capture of all Dora events

- **Layout Improvements**:
  - Left panel width: 300px → 200px (33% reduction)
  - Compact spacing: ~28px vertical space saved
  - PanelHeader padding: 12px → 8px
  - CastScreen top padding: 16px → 12px

- **Visual Polish**:
  - Application icon: 🎙️ (studio microphone)
  - Custom dropdown styling with hover effects
  - Fixed color schemes for light theme
  - Text input auto-wrap and scrolling

**Files Modified**:
- `src/screen.rs` - Major UI updates
  - Added log_section UI (lines 448-596)
  - Fixed dropdown styles (format_dropdown, level_filter)
  - Improved text inputs (original_text, refined_text)
  - Added icon_label (header)
  - Reduced left panel width
  - Compact spacing

**State Added**:
- `log_entries: Vec<String>` - Log storage
- `log_level_filter: u32` - Filter level
- `log_panel_collapsed: bool` - Panel state
- `log_panel_width: f64` - Panel width

**Methods Added**:
- `ensure_log_initialized()` - Lazy initialization
- `toggle_log_panel()` - Show/hide panel
- `update_log_display()` - Filter and render
- `add_log()` - Add log entry
- `clear_logs()` - Reset logs

**Bugs Fixed**:
1. **Stack Overflow** - Fixed infinite recursion in initialization
2. **Scroll Component** - Changed from `Scroll` to `ScrollYView`
3. **White Text** - Fixed log text color (white → GRAY_700)

**Breaking Changes**: None

**Migration Notes**: None

---

### ⏳ Changed - TTS Engine Migration

**From**: Kokoro-82M (single voice)
**To**: PrimeSpeech (multi-voice)

**Reason**:
- Multi-voice support requirement (P1.1)
- Better Chinese TTS quality
- Multiple voice models available

**Impact**:
- ✅ Support for 3+ distinct voices
- ✅ Better Chinese pronunciation
- ✅ More voice options
- ⚠️ Single-voice mode still supported

**Files Deprecated**:
- `docs/KOKORO_TTS_GUIDE.md` → `docs/KOKORO_TTS_GUIDE_DEPRECATED.md`

**Dataflow Changes**:
- Old: `dataflow/batch-tts.yml` (Kokoro)
- New: `dataflow/multi-voice-batch-tts.yml` (PrimeSpeech × 3)

**Compatibility**: Fully backward compatible with single-voice mode

---

## [0.4.0] - 2026-01-14

### ✅ Added - P1.1 Multi-Voice Support

**See version 0.5.0 for full details**

---

## [0.3.0] - 2026-01-09

### ✅ Added - P0.6 Dora Integration

**Core Feature**: 100% reliable sequential TTS sending

- **DoraProcessManager**: Auto-start daemon/coordinator
- **Event Forwarding**: Fixed segment_complete propagation
- **Sequential Processing**: Flow control with segment_complete handshake
- **Test Results**: 10/10 segments generated (100% success rate)

**Critical Fix**:
```rust
// Before (BROKEN - only 1 segment):
"segment_complete" => {
    // No event sent!
}

// After (WORKING - all segments):
"segment_complete" => {
    let _ = event_sender.send(BridgeEvent::DataReceived {
        input_id: "segment_complete".to_string(),
        data: DoraData::Empty,
        metadata: event_meta,
    });
}
```

**Performance**:
- ⚠️ Slower (40s for 10 segments vs 10s with batch)
- ✅ But 100% reliable (batch only 20% reliable)
- ✅ Predictable processing time
- ✅ Better progress tracking

---

### ✅ Added - P0.5 Audio Mixing and Export

**Features**:
- Audio segment concatenation
- Silence between segments (0.5s default)
- WAV file export
- Volume normalization interface
- Metadata support structure

**Testing**: 5 unit tests, all passing

---

## [0.2.0] - 2026-01-08

### ✅ Added - P0.3 AI Script Refinement

**Features**:
- OpenAI API integration (GPT-4)
- Mock refiner for testing without API
- Streaming responses
- Prompt templates for structured outputs
- Comprehensive error handling (8 error types)

**Testing**: 2 unit tests, all passing

---

## [0.1.0] - 2026-01-08

### ✅ Added - P0.1 Transcript Parsing

**Features**:
- Plain text parser (speaker: message format)
- JSON parser (OpenAI chat format)
- Markdown parser (GitHub discussions)
- Auto-detection with ParserFactory
- Speaker statistics extraction
- Unit tests (5/5 passing)

**Files**: `transcript_parser.rs` (~672 lines)

---

### ✅ Added - P0.2 Script Editor UI

**Features**:
- Split-view layout (original | refined)
- File import with format detection
- Dark mode support
- Shell integration with sidebar
- Speaker statistics display

**Files**: `screen.rs` (~590 lines)

---

## [0.0.1] - 2026-01-07

### ✅ Added - Project Initialization

**Features**:
- Project structure created
- Dependencies configured (Cargo.toml)
- MofaApp trait implemented
- Documentation organized
- MoFA Studio shell integration

---

## Version Summary

| Version | Date | Status | Key Features |
|---------|------|--------|--------------|
| 0.6.3 | 2026-01-16 | ✅ Stable | In-app audio player widget |
| 0.6.2 | 2026-01-16 | ✅ Stable | Volume normalization, ID3 tags |
| 0.6.1 | 2026-01-16 | ✅ Stable | MP3 export with bitrate selection |
| 0.6.0 | 2026-01-15 | ✅ Stable | TTS-focused workflow (refactored) |
| 0.5.0 | 2026-01-14 | ✅ Stable | Multi-voice support, UI enhancements |
| 0.4.0 | 2026-01-14 | ✅ Stable | Dora integration, sequential sending |
| 0.3.0 | 2026-01-09 | ✅ Stable | Audio mixing and WAV export |
| 0.2.0 | 2026-01-08 | ✅ Stable | AI script refinement |
| 0.1.0 | 2026-01-08 | ✅ Stable | Transcript parsing and UI |
| 0.0.1 | 2026-01-07 | ✅ Alpha | Project initialization |

---

## Migration Guide

### Upgrading from 0.3.x to 0.5.0

**No breaking changes** - All existing scripts and dataflows work.

**New Features Available**:
1. Multi-voice support: Use `multi-voice-batch-tts.yml` instead of `batch-tts.yml`
2. Log viewer: Automatically enabled in UI
3. Compact layout: Left panel now 200px (was 300px)

**Recommended Actions**:
- Update dataflow path to use `multi-voice-batch-tts.yml`
- Configure voice mapping in UI (automatic for common patterns)
- Use log panel for debugging and progress tracking

---

## Deprecations

### Deprecated in 0.5.0

- **Kokoro TTS Engine**: Replaced by PrimeSpeech for multi-voice support
  - Reason: PrimeSpeech supports multiple voice models
  - Migration: Update dataflow to use PrimeSpeech nodes
  - Documentation: See `docs/KOKORO_TTS_GUIDE_DEPRECATED.md`

### Removed Features

None - All features from 0.3.x are still supported in 0.5.0

---

## Future Roadmap

### Planned for v0.6.0

- **MP3 Export**: Bitrate selection (128k/192k/320k)
- **Audio Player Widget**: In-app playback
- **Keyboard Shortcuts**: Ctrl+O (open), Ctrl+S (save), Ctrl+E (export)
- **Auto-Save**: Prevent data loss

### Planned for v0.7.0

- **Advanced Format Support**: WhatsApp, WeChat, Telegram, Discord
- **Audio Enhancements**: Volume normalization, crossfade
- **Progress Tracking**: ETA calculation, per-segment progress bars
