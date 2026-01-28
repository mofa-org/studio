# MoFA Cast - Development History

> Archived documentation of deprecated features, abandoned approaches, and historical decisions

**Last Updated**: 2026-01-17  
**Purpose**: Preserve historical context while keeping active documentation clean

---

## Documentation Cleanup (v0.6.3)

**Archived Documents** (19 → 1):

The following documents were consolidated into this history file:

1. KOKORO_TTS_GUIDE_DEPRECATED.md
2. FILE_DIALOG_GUIDE.md
3. FILE_DIALOG_TROUBLESHOOTING.md
4. MP3_EXPORT_RESEARCH.md
5. TEST_REPORT_TEMPLATE.md
6. NEXT_STEPS.md
7. NEXT_STeps_v0.6.0.md
8. roadmap-claude.md
9. v0.6.0_RELEASE_NOTES.md
10. v0.6.0_TESTING_GUIDE.md
11. IMPROVEMENTS.md
12. ARCHITECTURE_cn.md
13. DROPDOWN_STATE_MANAGEMENT.md
14. UI_LAYOUT.md
15. TTS_INTEGRATION.md
16. TTS_WORKFLOW_TEST.md
17. CHECKLIST.md
18. APP_DEVELOPMENT_GUIDE.md
19. KNOWN_ISSUES.md

**New Documentation Structure**:

```
apps/mofa-cast/
├── README.md                # Project overview
├── ARCHITECTURE.md          # Technical architecture
├── CHANGELOG.md             # Version history
└── docs/
    ├── USER_GUIDE.md        # User documentation (NEW)
    ├── TROUBLESHOOTING.md   # Issue resolution (NEW)
    ├── DEVELOPMENT.md       # Developer guide (NEW)
    └── HISTORY.md           # This file (NEW)
```

---

## Deprecated TTS Approaches

### Kokoro TTS (Deprecated v0.5.x)

**Status**: ❌ Abandoned  
**Reason**: Integration complexity, model loading issues  
**Replacement**: PrimeSpeech multi-voice TTS

**What Was It**:
Kokoro was an open-source TTS engine we attempted to integrate for local multi-voice synthesis.

**Why It Failed**:
1. Complex model loading process
2. Inconsistent voice quality
3. Performance issues on macOS
4. Limited voice options
5. Difficult dependency management

**Lessons Learned**:
- Prefer production-ready TTS engines
- Model loading should be transparent
- Voice quality consistency is critical
- Integration complexity matters

---

## Abandoned Features

### Volume Control UI (Removed v0.6.3)

**Status**: ❌ Removed  
**Reason**: Simplified UI, use system volume

**Original Design**:
- Volume slider in audio player section
- Range: 0-100%
- Real-time volume adjustment
- Platform-specific volume flags

**Why Removed**:
- Added UI complexity
- System volume sufficient
- Inconsistent across platforms
- User confusion about which volume to adjust

### Script Refinement with LLM (Removed v0.6.0)

**Status**: ❌ Removed  
**Reason**: External tools are better

**Original Concept**:
Integrate ChatGPT/Claude API directly into MoFA Cast for script optimization.

**Why Removed**:
- Users prefer external AI tools
- Direct interaction with GPT-4o/Claude 4 is better
- No API key management in app
- No ongoing costs for users
- More flexibility in prompt engineering

---

## Architecture Evolution

### TTS Integration Evolution

**Phase 1**: Mock TTS (v0.1.0)
**Phase 2**: Kokoro Integration (v0.3.x - v0.4.x) - Deprecated
**Phase 3**: PrimeSpeech Multi-Voice (v0.5.0+)

### Dataflow Architecture

**Initial Design**: Single TTS node
**Current Design**: Multi-voice with router (3 parallel PrimeSpeech nodes)

### Sequential Processing (v0.6.0)

**Problem**: All segments sent at once → race conditions
**Solution**: Sequential sending based on audio return
**Result**: Guaranteed order, no missing segments

---

## Version Summary

### v0.6.3 (2026-01-16)
- Added audio player widget
- Removed volume control

### v0.6.2 (2026-01-15)
- Added template system
- Recent files management

### v0.6.1 (2026-01-14)
- MP3 export support
- FFmpeg integration

### v0.6.0 (2026-01-13)
- PrimeSpeech multi-voice TTS
- Dora dataflow integration
- Voice routing (3 voices)

### v0.5.x (2025-12-20)
- File dialog implementation
- Transcript parser
- Audio mixer

---

## Research Notes

### MP3 Export Research

**Options Evaluated**:
1. mp3lame-encoder (rejected - limited features)
2. ffmpeg CLI (chosen - reliable, fast)
3. lame bindings (rejected - complex FFI)

**Final Choice**: FFmpeg via command line

---

## Decision Log

### Why Makepad?
- Native performance
- GPU-accelerated
- Rust-native
- Declarative UI

### Why Dora?
- Modular architecture
- Multi-language support
- Battle-tested

### Why PrimeSpeech?
- Production quality
- Multiple voices
- Fast inference
- Good documentation

---

**Note**: This file is for historical reference only. See active documentation in:
- README.md
- ARCHITECTURE.md  
- CHANGELOG.md
- docs/USER_GUIDE.md
- docs/TROUBLESHOOTING.md
- docs/DEVELOPMENT.md

