# MoFA Cast - Troubleshooting Guide

**Version**: 0.6.3
**Last Updated**: 2026-01-17

---

## Table of Contents

1. [Quick Diagnostics](#quick-diagnostics)
2. [Common Issues](#common-issues)
3. [TTS-Specific Issues](#tts-specific-issues)
4. [Audio Export Issues](#audio-export-issues)
5. [File Dialog Issues](#file-dialog-issues)
6. [Performance Issues](#performance-issues)
7. [Debug Mode](#debug-mode)

---

## Quick Diagnostics

### System Health Check

Before diving into specific issues, verify your system is ready:

```bash
# 1. Check Dora installation
dora --version

# 2. Check PrimeSpeech models
ls ~/.dora/models/primespeech/

# 3. Check FFmpeg (for MP3 export)
ffmpeg -version

# 4. Check output directory permissions
ls -la ./output/mofa-cast/
```

**Expected Results**:
- Dora: Version 0.3.0 or higher
- PrimeSpeech: Model files present (several GB)
- FFmpeg: Version 4.0 or higher (optional)
- Output: Directory writable

### Log Analysis

Check the log panel for error patterns:

**Green Flags** (Healthy):
```
[INFO] ✓ Dora processes started successfully
[INFO] ✓ Dataflow started successfully
[INFO] ✅ Saved segment X of Y
[INFO] ✅ All segments received
```

**Red Flags** (Issues):
```
[ERROR] ❌ Failed to start dataflow
[ERROR] ❌ Failed to parse transcript
[WARN] ⚠️ No audio segments collected
```

---

## Common Issues

### Issue: "No audio segments collected. Please synthesize audio first."

**Symptoms**:
- Click "Export Audio" button
- Error message appears
- No audio files in output directory

**Causes**:
1. TTS synthesis not completed
2. Dataflow failed to start
3. All segments failed to generate

**Solutions**:

1. **Verify synthesis completed**:
   - Check log for "✅ All X segments received"
   - Header should show synthesis progress 100%

2. **Check dataflow status**:
   ```bash
   dora ps
   ```
   - Should see `mofa-cast-multi-voice` running

3. **Re-synthesize audio**:
   - Click "Synthesize Audio" again
   - Wait for all segments to complete

4. **Check output directory**:
   ```bash
   ls ./output/mofa-cast/dora/
   ```
   - Should see segment_000_*.wav files

**Prevention**:
- Always wait for "All segments received" message
- Check logs for TTS errors during synthesis

---

### Issue: "Failed to parse transcript"

**Symptoms**:
- Import script fails
- Error message shown
- No content loaded

**Causes**:
1. Invalid file format
2. Malformed JSON
3. Inconsistent plain text format

**Solutions**:

1. **Verify file format**:
   - Use correct file extension: `.txt`, `.json`, `.md`
   - Check format matches content

2. **Validate JSON**:
   ```bash
   # Use jq to validate JSON
   cat script.json | jq .
   ```
   - Fix syntax errors
   - Ensure proper escaping

3. **Check plain text format**:
   - Each line: `Speaker: Message`
   - Consistent speaker names
   - No empty lines in middle

4. **Try Auto-detect format**:
   - Set format dropdown to "Auto"
   - Let parser detect format

**Prevention**:
- Validate JSON before import
- Use consistent speaker names
- Test with short script first

---

### Issue: Some segments missing from export

**Symptoms**:
- Synthesis says "X segments received"
- Export succeeds but fewer segments than expected
- Audio playback incomplete

**Causes**:
1. TTS processing errors
2. Dataflow stopped early
3. Audio corruption

**Solutions**:

1. **Check synthesis logs**:
   - Look for ERROR or WARN messages
   - Note which segments failed
   - Check specific speaker/voice issues

2. **Verify all segments saved**:
   ```bash
   ls ./output/mofa-cast/dora/ | wc -l
   ```
   - Should equal expected segment count

3. **Re-synthesize failed segments**:
   - Note which speakers/segments failed
   - Simplify problematic text
   - Try synthesis again

4. **Check dataflow stability**:
   ```bash
   dora logs mofa-cast-multi-voice
   ```
   - Look for node crashes
   - Check memory issues

**Prevention**:
- Keep segments under 200 characters
- Use simple punctuation
- Avoid special characters

---

## TTS-Specific Issues

### Issue: Wrong voice assigned to speaker

**Symptoms**:
- Speaker sounds like wrong voice
- Inconsistent voices across segments

**Causes**:
1. Speaker name not recognized
2. Inconsistent naming in script
3. Voice mapping mismatch

**Solutions**:

1. **Normalize speaker names**:
   ```
   Good: host, guest1, guest2
   Bad:  Host, HOST, [主持人], Moderator
   ```

2. **Check voice mapping in logs**:
   ```
   [INFO] Voice mapping for X speakers:
   [INFO]   'host' → 'Luo Xiang' (speed: 1.0)
   [INFO]   'guest1' → 'Ma Yun' (speed: 1.0)
   ```

3. **Update script with consistent names**:
   - Use lowercase: `host`, `guest1`, `guest2`
   - Avoid special characters
   - Be consistent throughout

**Voice Mapping Reference**:
| Speaker Pattern | Voice Assigned |
|-----------------|----------------|
| `host`, `Host`, `主持人` | Luo Xiang |
| `guest1`, `Guest1` | Ma Yun |
| `guest2`, `Guest2` | Ma Baoguo |

---

### Issue: TTS synthesis very slow

**Symptoms**:
- Each segment takes 10+ seconds
- Overall synthesis takes several minutes
- CPU usage low

**Causes**:
1. CPU not fully utilized
2. Model not loaded in memory
3. System resource contention

**Solutions**:

1. **Check CPU usage**:
   ```bash
   # macOS
   top -pid $(pgrep -f primespeech)

   # Linux
   htop
   ```
   - Should see 50-100% CPU per TTS node
   - Multiple nodes = 150-300% total

2. **Verify model loaded**:
   - First synthesis is slower (model loading)
   - Subsequent syntheses should be faster

3. **Close other applications**:
   - Free up CPU resources
   - Reduce background tasks

4. **Check NUM_THREADS setting**:
   - In dataflow YAML files
   - Default: 4 threads per node
   - Increase if CPU has more cores

**Performance Expectations**:
- First segment: 5-10 seconds (model loading)
- Subsequent segments: 2-4 seconds each
- 10 segments ~30-40 seconds total

---

### Issue: TTS node crashes during synthesis

**Symptoms**:
- Synthesis stops midway
- Dataflow shows node failures
- Logs show segmentation fault

**Causes**:
1. Out of memory
2. Corrupted model files
3. Invalid input text

**Solutions**:

1. **Check available memory**:
   ```bash
   # macOS
   vm_stat

   # Linux
   free -h
   ```
   - Need at least 4GB free per TTS node
   - 3 nodes = 12GB minimum

2. **Verify model files**:
   ```bash
   ls -lh ~/.dora/models/primespeech/
   ```
   - Check file sizes (should be several GB)
   - Re-download models if corrupted

3. **Simplify problematic text**:
   - Remove emojis
   - Avoid special characters
   - Shorten very long segments

4. **Check crash logs**:
   ```bash
   dora logs mofa-cast-multi-voice --tail 100
   ```
   - Look for segmentation fault
   - Check Python traceback

**Prevention**:
- Ensure sufficient RAM
- Validate input text
- Keep segments under 300 characters

---

## Audio Export Issues

### Issue: MP3 export fails

**Symptoms**:
- "Export failed" error
- No MP3 file created
- WAV export works fine

**Causes**:
1. FFmpeg not installed
2. FFmpeg version too old
3. Invalid MP3 settings

**Solutions**:

1. **Verify FFmpeg installed**:
   ```bash
   ffmpeg -version
   ```
   - Should be version 4.0 or higher
   - If not found:
     ```bash
     # macOS
     brew install ffmpeg

     # Linux
     sudo apt install ffmpeg
     ```

2. **Test FFmpeg manually**:
   ```bash
   ffmpeg -i input.wav -codec:a libmp3lame -b:a 192k output.mp3
   ```
   - Should convert successfully
   - Check error messages if fails

3. **Try different bitrate**:
   - Lower bitrate (128 kbps)
   - Or export to WAV instead

**Prevention**:
- Install FFmpeg before first use
- Test conversion with sample file

---

### Issue: Exported audio too quiet or too loud

**Symptoms**:
- Volume inconsistent
- Need to adjust volume manually

**Causes**:
1. Normalization applied incorrectly
2. Input audio already normalized
3. Different segments have different levels

**Solutions**:

1. **Check normalization setting**:
   - Currently fixed at -14 dB (EBU R128)
   - This is broadcast standard
   - Should be appropriate for most content

2. **Export to WAV and edit**:
   - Use Audacity or similar
   - Apply custom normalization
   - Re-export as MP3

3. **Check source audio**:
   - Verify TTS output levels
   - Check for inconsistencies

**Volume Reference**:
- -14 dB LUFS = Broadcast standard
- -16 dB LUFS = Podcast standard
- -12 dB LUFS = Louder (more dynamic)

---

### Issue: Export creates very large file

**Symptoms**:
- File size much larger than expected
- Disk space issues

**Causes**:
1. Exporting to uncompressed WAV
2. Very long script
3. High sample rate

**Solutions**:

1. **Use MP3 format**:
   - 192 kbps recommended
   - Reduces file size by ~80%
   - Minimal quality loss

2. **Check audio duration**:
   - Duration × bitrate = expected size
   - Example: 125 seconds × 192 kbps ≈ 3 MB

3. **Verify sample rate**:
   - Default: 32000 Hz (PrimeSpeech)
   - Higher rate = larger file
   - Can't be changed currently

**File Size Calculator**:
```
WAV: Duration (s) × Sample Rate × 2 bytes × Channels
MP3: Duration (s) × Bitrate / 8

Example (125 seconds):
WAV: 125 × 32000 × 2 × 1 = 8,000,000 bytes ≈ 7.6 MB
MP3 (192 kbps): 125 × 192000 / 8 = 3,000,000 bytes ≈ 2.9 MB
```

---

## File Dialog Issues

### Issue: File dialog doesn't open

**Symptoms**:
- Click "Import Script"
- Nothing happens
- No error message

**Causes**:
1. macOS permission issue
2. File dialog blocked
3. UI freeze

**Solutions**:

1. **Check macOS permissions**:
   ```bash
   # Check if file access is allowed
   # System Settings → Privacy & Security → Files and Folders
   ```
   - Grant Full Disk Access to MoFA Studio
   - Or grant access to specific folders

2. **Restart application**:
   - Quit MoFA Studio completely
   - Reopen and try again

3. **Use alternative method**:
   - Place script in test_samples/
   - Type path directly (if supported)

**Prevention**:
- Grant necessary permissions on first launch
- Keep app updated

---

## Performance Issues

### Issue: Application freezes during synthesis

**Symptoms**:
- UI becomes unresponsive
- Progress doesn't update
- Force quit required

**Causes**:
1. Main thread blocked
2. Memory exhaustion
3. Dataflow deadlock

**Solutions**:

1. **Check system resources**:
   ```bash
   # macOS Activity Monitor
   # Check CPU and Memory usage
   ```
   - Look for memory leaks
   - Check CPU saturation

2. **Monitor from terminal**:
   ```bash
   # Run from command line to see logs
   cargo run -p mofa-studio
   ```
   - Check for error messages
   - Look for panic/stack traces

3. **Kill and restart**:
   ```bash
   # Stop Dora processes
   dora destroy mofa-cast-multi-voice

   # Restart application
   ```

**Prevention**:
- Don't run other heavy tasks
- Ensure sufficient RAM
- Keep scripts reasonable length

---

### Issue: Memory usage keeps growing

**Symptoms**:
- Memory usage increases with each synthesis
- Eventually crashes or slows down
- Memory not released after export

**Causes**:
1. Audio data not freed
2. Log entries accumulate
3. TTS models not unloaded

**Solutions**:

1. **Restart application periodically**:
   - After 5-10 syntheses
   - Clears cached data
   - Frees memory

2. **Clear log panel**:
   - Click "Clear Log" button
   - Reduces memory footprint

3. **Monitor memory usage**:
   ```bash
   # macOS
   top -pid $(pgrep mofa-studio)

   # Check RSS (Resident Set Size)
   ```

**Expected Memory Usage**:
- Base: 200-400 MB
- With TTS: +2-4 GB (models in memory)
- Per synthesis: +50-100 MB (audio data)
- Total: 3-5 GB typical

---

## Debug Mode

### Enable Verbose Logging

For detailed debugging information:

1. **Run from terminal**:
   ```bash
   cd /path/to/mofa-studio
   RUST_LOG=debug cargo run -p mofa-studio
   ```

2. **Check Dora logs**:
   ```bash
   # Real-time logs
   dora logs mofa-cast-multi-voice --follow

   # Last 100 lines
   dora logs mofa-cast-multi-voice --tail 100
   ```

3. **Save logs to file**:
   ```bash
   RUST_LOG=debug cargo run -p mofa-studio 2>&1 | tee mofa-cast-debug.log
   ```

### Common Debug Patterns

**Pattern 1: Dataflow won't start**
```
[ERROR] Failed to start dataflow: ...
```
→ Check YAML syntax
→ Verify node paths
→ Check environment variables

**Pattern 2: Audio not generated**
```
[INFO] 🎙️ Synthesizing... X/Y
[WARN] No audio received after 30s
```
→ Check TTS node status
→ Verify text routing
→ Test with simple script

**Pattern 3: Export fails**
```
[ERROR] ❌ Export failed: ...
```
→ Check audio files exist
→ Verify mixer config
→ Test with fewer segments

### Report an Issue

When reporting bugs, include:

1. **System Information**:
   ```bash
   uname -a              # OS and kernel
   rustc --version       # Rust version
   dora --version        # Dora version
   ```

2. **Error Messages**:
   - Full error text
   - Log output
   - Screenshots if applicable

3. **Steps to Reproduce**:
   - What you did
   - What you expected
   - What actually happened

4. **Test Case**:
   - Minimal script that shows issue
   - File attachments if relevant

---

## Additional Resources

### Documentation

- [User Guide](USER_GUIDE.md) - How to use MoFA Cast
- [Architecture](../ARCHITECTURE.md) - Technical details
- [Development](DEVELOPMENT.md) - For contributors

### External Tools

- **Audacity** - Audio editing: https://www.audacityteam.org/
- **FFmpeg** - Audio conversion: https://ffmpeg.org/
- **Dora** - Dataflow system: https://github.com/dora-rs/dora

### Community

- **GitHub Issues**: Bug reports and feature requests
- **Discussions**: Questions and community support

---

**Last Updated**: 2026-01-17
**Version**: 0.6.3

---

## Quick Reference Card

```
PROBLEM                      | SOLUTION
-----------------------------|----------------------------------
No audio collected           | Wait for synthesis, check dataflow
Parse failed                 | Validate file format, check JSON
Wrong voice                  | Normalize speaker names
Slow synthesis               | Close other apps, check CPU
Node crashes                 | Check memory, verify models
MP3 export fails             | Install FFmpeg, try WAV
File dialog stuck            | Check macOS permissions
App freezes                  | Check resources, restart app
Memory growing               | Restart app periodically
```

---

## Archived TTS Troubleshooting

*The following content was migrated from the old TTS_TROUBLESHOOTING.md file*

### Legacy TTS Issues (Kokoro - Deprecated)

**Note**: Kokoro TTS has been deprecated in favor of PrimeSpeech. The following issues are kept for historical reference only.

#### Issue: Kokoro model not loading

**Symptoms**:
- Error: "Model not found"
- Synthesis fails immediately

**Solution** (If still using Kokoro):
1. Verify model path in environment variables
2. Check model files exist
3. Consider migrating to PrimeSpeech

**Migration Path**: See [docs/HISTORY.md](HISTORY.md) for migration guide

#### Issue: Kokoro Python environment issues

**Symptoms**:
- ImportError in Python
- Wrong Python version

**Solution** (If still using Kokoro):
1. Verify conda environment: `conda activate mofa-studio`
2. Reinstall dependencies: `pip install -r requirements.txt`
3. Check Python version: `python --version` (should be 3.8+)

**Current Recommendation**: Use PrimeSpeech (no Python dependency issues)

---

**End of Archived Content**
