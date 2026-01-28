# MoFA Cast - User Guide

**Version**: 0.6.3
**Last Updated**: 2026-01-17

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Importing Scripts](#importing-scripts)
3. [Using Templates](#using-templates)
4. [Synthesizing Audio](#synthesizing-audio)
5. [Exporting Audio](#exporting-audio)
6. [Playing Audio](#playing-audio)
7. [Understanding Log Output](#understanding-log-output)

---

## Quick Start

MoFA Cast transforms your podcast scripts into professional multi-voice audio using local TTS synthesis.

### Basic Workflow

```
1. Import Script (Plain Text/JSON/Markdown)
2. Optional: Use Template or Edit Script
3. Synthesize Audio (Multi-voice TTS)
4. Export Audio (WAV/MP3)
5. Play Audio (In-app or External Player)
```

### System Requirements

- **Operating System**: macOS, Linux, Windows
- **Dependencies**:
  - Dora dataflow system
  - PrimeSpeech TTS models (local)
  - FFmpeg (for MP3 export, optional)

---

## Importing Scripts

MoFA Cast supports multiple transcript formats:

### 1. Plain Text Format

Simple conversational format:

```
Speaker1: Hello everyone!
Speaker2: Hi! Welcome to the show.
Speaker1: Today we'll discuss...
```

**Requirements**:
- Each line: `Speaker: Message`
- Consistent speaker names
- One message per line

### 2. JSON Format

Structured format with metadata:

```json
[
  {
    "speaker": "host",
    "text": "Welcome to the podcast!",
    "timestamp": "00:00:00"
  },
  {
    "speaker": "guest",
    "text": "Thank you for having me.",
    "timestamp": "00:00:05"
  }
]
```

**Benefits**:
- Includes timestamps
- Structured data
- Easy to parse programmatically

### 3. Markdown Format

Readable format with headers:

```markdown
# Podcast Title

## Host
Welcome to today's episode!

## Guest
Great to be here.

## Host
Let's start with...
```

**Requirements**:
- `## SpeakerName` headers
- Content under each header
- Blank lines between sections

### Import Steps

1. Click **"Import Script"** button
2. Select file format (Auto-detect recommended)
3. Choose your transcript file
4. Review parsed content:
   - **File name** and message count
   - **Speakers list** with message counts
   - **Script editor** shows formatted content

**Tips**:
- Use **Auto** format detection for best results
- Check speaker list to verify correct parsing
- Edit script directly in the editor if needed

---

## Using Templates

MoFA Cast includes pre-built templates for common podcast formats.

### Available Templates

#### 1. Two-Person Interview
```
Host + Single Guest
- Professional interview format
- Balanced dialogue
- Ideal for Q&A sessions
```

#### 2. Three-Person Discussion
```
Host + Two Guests
- Panel discussion format
- Multiple perspectives
- Round-table conversations
```

#### 3. Narrative Storytelling
```
Single Narrator
- Storytelling format
- Educational content
- Audiobook style
```

### Using Templates

1. Select template from dropdown
2. Template loads automatically into editor
3. Customize content:
   - Edit speaker names
   - Modify dialogue
   - Adjust pacing
4. Use **"Open in Editor"** to edit externally

**Benefits**:
- Quick start with proven formats
- Consistent structure
- Easy customization

---

## Synthesizing Audio

Convert your script to multi-voice audio using local TTS.

### Voice Mapping

MoFA Cast automatically assigns voices to speakers:

| Speaker Type | Voice Name | Characteristics |
|--------------|------------|-----------------|
| Host | Luo Xiang | Deep, authoritative male voice |
| Guest1 | Ma Yun | Energetic, confident male voice |
| Guest2 | Ma Baoguo | Characteristic, distinctive voice |

**Automatic Detection**:
- `host`, `Host`, `[主持人]` → Luo Xiang
- `guest1`, `Guest1` → Ma Yun
- `guest2`, `Guest2` → Ma Baoguo

### Synthesis Process

1. **Click "Synthesize Audio" button**
2. **Progress indicators**:
   - Header shows: "Synthesizing X/Y (Z%) - Speaker"
   - Log shows: "✅ Saved segment X of Y"
3. **Wait for completion**:
   - All segments received
   - Export button enabled
4. **Typical duration**: 2-5 seconds per segment

### What Happens During Synthesis

```
Script → Voice Router → TTS Nodes → Audio Collection
         (Luo Xiang)    (Ma Yun)     (Segment 1)
                        (Ma Baoguo)   (Segment 2)
                                       ...
                                       (Segment N)
```

**Key Features**:
- **Sequential processing**: Segments sent one-by-one
- **Voice routing**: Each speaker gets assigned voice
- **Audio collection**: All segments saved to `./output/mofa-cast/dora/`

### Troubleshooting

**Issue**: "No audio segments collected"
- **Solution**: Click Synthesize again after dataflow starts

**Issue**: Missing some segments
- **Solution**: Check logs for TTS errors, verify all speakers mapped

**Issue**: Wrong voice for speaker
- **Solution**: Normalize speaker names (use guest1, guest2, host)

---

## Exporting Audio

Combine all audio segments into a single file.

### Export Options

#### Format Selection

| Format | Quality | File Size | Use Case |
|--------|---------|-----------|----------|
| WAV | Lossless | Large (~10MB/min) | Archiving, editing |
| MP3 | High | Small (~2MB/min) | Distribution, streaming |

#### MP3 Bitrate Options

| Bitrate | Quality | File Size | Recommendation |
|---------|---------|-----------|----------------|
| 128 kbps | Good | ~1MB/min | Voice-only content |
| 192 kbps | High | ~1.5MB/min | **Recommended** |
| 256 kbps | Very High | ~2MB/min | Music + voice |
| 320 kbps | Maximum | ~2.5MB/min | Audiophile quality |

### Audio Processing

During export, MoFA Cast applies:

1. **Volume Normalization**: EBU R128 standard (-14 dB)
   - Consistent volume across segments
   - Professional broadcast quality

2. **Silence Insertion**: 0.5 seconds between segments
   - Natural pauses
   - Clear speaker transitions

3. **Metadata Tagging** (MP3 only):
   - Title: Script filename
   - Artist: MoFA Cast
   - Album: Generated by MoFA Cast
   - Year: Current year
   - Comment: Segment count info

### Export Steps

1. Select **Export Format** (WAV or MP3)
2. If MP3, select **Bitrate** (192 kbps recommended)
3. Click **"Export Audio"** button
4. **Wait for mixing**:
   - Progress shows: "Mixing and exporting audio..."
   - Status updates with duration and file size
5. **Output location**:
   - WAV: `./output/mofa-cast/podcast.wav`
   - MP3: `./output/mofa-cast/podcast.mp3`

### Export Results

After successful export:

```
✅ Exported! 125.3s audio • 2048KB
```

**Audio Player Section** updates:
- **Status**: "Ready to play"
- **Format**: WAV or MP3
- **Duration**: Total playback time
- **File Size**: Exported file size

---

## Playing Audio

Listen to your exported audio directly in MoFA Cast.

### Playback Controls

#### Play Button (▶)
- Starts audio playback
- Uses platform-specific player:
  - **macOS**: `afplay`
  - **Windows**: `wmplayer`
  - **Linux**: `vlc`
- Status shows: "Playing"

#### Stop Button (⏹)
- Stops playback immediately
- Terminates audio player process
- Status shows: "Stopped"

#### Open in Player Button
- Launches system default audio player
- Applications:
  - **macOS**: QuickTime Player, iTunes
  - **Windows**: Windows Media Player, Groove
  - **Linux**: VLC, Audacious
- **Benefits**: Full player controls, seek, volume

### Volume Control

**Note**: Volume control removed in v0.6.3 - use system volume or external player volume.

### Audio Information

The player section displays:
- **Format**: WAV or MP3
- **Duration**: Total playback time (e.g., "125.3 seconds")
- **File Size**: Exported file size (e.g., "2048 KB")

### Playback Status

| Status | Meaning |
|--------|---------|
| No audio | No audio exported yet |
| Ready to play | Audio exported, ready for playback |
| Playing | Audio currently playing |
| Stopped | Playback stopped |

---

## Understanding Log Output

MoFA Cast includes a real-time log viewer for monitoring progress and troubleshooting.

### Log Levels

Filter logs by severity:

| Level | Description | Example |
|-------|-------------|---------|
| ALL | All log messages | Development debugging |
| INFO | Normal operations | "✅ Saved segment 1 of 10" |
| WARN | Warnings | "⚠️ File dialog cancelled" |
| ERROR | Errors only | "❌ Failed to parse transcript" |

### Log Panel Controls

- **Toggle Button** (< or >): Expand/collapse log panel
- **Level Filter**: Select minimum log level to display
- **Clear Log**: Remove all log entries

### Common Log Messages

#### Successful Operations

```
[INFO] 📂 Opening file dialog...
[INFO] ✅ File selected: sample_markdown.md
[INFO] ✅ Parsed: 3 speakers, 10 messages
[INFO] 🎙️ Starting Dora dataflow...
[INFO] ✅ Saved segment 1 of 10: segment_000_host.wav (7.68s)
[INFO] ✅ All 10 segments received! Ready to export.
[INFO] 📥 Mixing and exporting audio...
[INFO] ✅ Exported! 125.3s audio • 2048KB
```

#### Warning Messages

```
[WARN] ⚠️ No script file loaded
[WARN] ⚠️ Audio file not found. Please export first
[WARN] Failed to create temp file for template
```

#### Error Messages

```
[ERROR] ❌ Failed to parse transcript: Invalid JSON
[ERROR] ❌ Dataflow configuration not found
[ERROR] ❌ Failed to start TTS dataflow
[ERROR] ❌ Export failed: No segments to mix
```

### Log Color Coding (Terminal)

- **White**: Normal text
- **Green**: Success messages (✅)
- **Yellow**: Warnings (⚠️)
- **Red**: Errors (❌)
- **Blue**: Informational headers

### Using Logs for Troubleshooting

1. **Check first error**:
   - Scroll up to first ERROR message
   - Error context usually above it

2. **Look for patterns**:
   - Repeated errors = systemic issue
   - Single error = transient issue

3. **Export logs**:
   - Select all text (Cmd/Ctrl+A)
   - Copy and paste to text file
   - Share with support team

---

## Tips and Best Practices

### Script Preparation

1. **Use consistent speaker names**:
   - Good: `host`, `guest1`, `guest2`
   - Bad: `Host`, `HOST`, `[Host]`, `主持人`

2. **Keep segments focused**:
   - One topic per segment
   - 1-3 sentences per message
   - Natural speaker transitions

3. **Test with short scripts first**:
   - 3-5 messages
   - Verify voice mapping
   - Check audio quality

### Audio Quality

1. **Use MP3 192 kbps** for best balance:
   - Good quality
   - Reasonable file size
   - Wide compatibility

2. **Normalize input scripts**:
   - Remove excessive whitespace
   - Fix typos before synthesis
   - Use consistent punctuation

3. **Export to WAV for editing**:
   - Lossless quality
   - Edit in Audacity or similar
   - Convert to MP3 later

### Performance

1. **Close other applications** during synthesis:
   - Frees up CPU for TTS
   - Prevents audio glitches

2. **Use SSD for output directory**:
   - Faster file writes
   - Better audio quality

3. **Monitor system resources**:
   - Check CPU usage
   - Available memory
   - Disk space

---

## FAQ

### Q: Can I use custom voices?

**A**: Currently, MoFA Cast supports 3 pre-configured voices (Luo Xiang, Ma Yun, Ma Baoguo). Custom voice support is planned for future releases.

### Q: Why are some segments missing?

**A**: Check:
1. All speakers are mapped correctly
2. No TTS errors in logs
3. Dataflow is running (check Dora processes)

### Q: Can I edit audio after export?

**A**: Yes! Export to WAV (lossless), edit in Audacity or similar, then convert to MP3 if needed.

### Q: How do I change the voice for a speaker?

**A**: Normalize speaker names in your script:
- `host`, `Host`, `[主持人]` → Luo Xiang
- `guest1`, `Guest1` → Ma Yun
- `guest2`, `Guest2` → Ma Baoguo

### Q: Can I adjust the speed of speech?

**A**: Voice speed is fixed at 1.0x (normal) in current version. Speed adjustment is planned for future releases.

### Q: What's the maximum script length?

**A**: No hard limit, but practical limit is ~100 segments (15-20 minutes) for optimal performance.

---

## Getting Help

### Documentation

- **Architecture**: See [ARCHITECTURE.md](../ARCHITECTURE.md)
- **Development**: See [docs/DEVELOPMENT.md](DEVELOPMENT.md)
- **Changelog**: See [CHANGELOG.md](../CHANGELOG.md)

### Troubleshooting

- **Common Issues**: See [docs/TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- **Known Issues**: Check [CHANGELOG.md](../CHANGELOG.md) for version-specific issues

### Support

For bugs, feature requests, or questions:
1. Check existing issues
2. Search troubleshooting guide
3. Create new issue with:
   - MoFA Cast version
   - Operating system
   - Error logs
   - Steps to reproduce

---

**Last Updated**: 2026-01-17
**Version**: 0.6.3
