# Script Optimization Guide

> How to optimize your podcast scripts with external AI tools

**Last Updated**: 2026-01-15
**For MoFA Cast Version**: 0.6.0+

## Why External Optimization?

MoFA Cast focuses on what it does best: **multi-voice TTS synthesis**.

For script optimization, we recommend using dedicated AI tools:
- ✅ **Zero cost** - No API fees
- ✅ **Better quality** - Direct LLM interaction
- ✅ **Latest models** - GPT-4o, Claude 4, etc.
- ✅ **Flexibility** - Iterate until perfect

## Recommended Tools

### ChatGPT (https://chatgpt.com)
**Best for**: General podcast scripting

- ✅ Free tier available
- ✅ GPT-4o access (Plus users)
- ✅ Excellent conversational abilities
- ✅ Easy to use

### Claude (https://claude.ai)
**Best for**: Long-form content, nuanced writing

- ✅ Superior long-context understanding
- ✅ Natural, human-like writing
- ✅ Great for educational content
- ✅ Free tier available

## Recommended Prompt Templates

### Template 1: Basic Podcast Conversion

```
Transform this chat transcript into an engaging podcast script:

Requirements:
- Add a brief introduction
- Smooth transitions between speakers
- Maintain conversational tone
- Remove filler words (um, uh, like)
- Format as: Speaker: Dialogue

Original transcript:
[paste your chat log here]
```

### Template 2: Multi-Speaker Format (for MoFA Cast)

```
Convert this chat into a podcast script with these speakers:

- host: Main narrator/interviewer
- guest1: First guest (expert opinion)
- guest2: Second guest (counterpoint)

Requirements:
- Use exact speaker names: "host:", "guest1:", "guest2:"
- Add engaging intro and outro
- Keep responses conversational but concise
- Format for text-to-speech (clear punctuation)

Chat transcript:
[paste your transcript]
```

### Template 3: Educational Podcast

```
Transform this conversation into an educational podcast:

Style: Informative but friendly
Audience: General listeners (non-technical)
Format:
  - host: Explains concepts
  - guest: Provides expertise

Requirements:
- Simplify technical terms
- Add brief summaries after key points
- Include real-world examples
- Keep under 30 minutes when spoken

Original:
[paste content]
```

### Template 4: Interview Style

```
Convert this interview into a podcast script:

Format:
- host: Interviewer
- guest: Interviewee

Enhancements:
- Add engaging questions if missing
- Smooth transitions between topics
- Remove awkward pauses or repetitions
- Maintain authentic voice of guest

Style: Conversational interview (like NPR, Joe Rogan)

Original interview transcript:
[paste transcript]
```

## MoFA Cast Speaker Format

MoFA Cast automatically recognizes these speaker names and assigns voices:

| Speaker Name | Voice Model | Characteristics |
|--------------|-------------|-----------------|
| `host`, `[主持人]` | Luo Xiang | Deep male, authoritative |
| `guest1`, `Guest 1` | Ma Yun | Energetic male |
| `guest2`, `Guest 2` | Ma Baoguo | Characteristic |

### Custom Speakers

If you need more than 3 speakers, use consistent names:
```
speaker1: (will use Luo Xiang)
speaker2: (will use Ma Yun)
speaker3: (will use Ma Baoguo)
```

**Important**: Use consistent speaker names throughout your script!

## Step-by-Step Workflow

### 1. Prepare Your Source Material

**Chat logs** (Discord, Slack, WhatsApp):
- Export as plain text
- Clean up timestamps if needed

**Transcripts** (YouTube, meetings):
- Export as plain text or SRT
- Remove timecodes

**Raw notes**:
- Outline structure
- Key talking points

### 2. Optimize with AI

1. **Open ChatGPT or Claude**
2. **Copy one of the prompt templates above**
3. **Paste your source material**
4. **Review the output**
5. **Iterate if needed** (ask for adjustments)
6. **Copy final script**

### 3. Save for MoFA Cast

**Recommended formats**:

**Plain Text** (`script.txt`):
```
host: Welcome to today's episode...

guest1: Thanks for having me...

guest2: I'm excited to be here...
```

**Markdown** (`script.md`):
```markdown
# Podcast Episode 1

host: Welcome to today's episode...

guest1: Thanks for having me...
```

**JSON** (`script.json`):
```json
{
  "title": "Podcast Episode 1",
  "segments": [
    {"speaker": "host", "text": "Welcome..."},
    {"speaker": "guest1", "text": "Thanks..."}
  ]
}
```

### 4. Import to MoFA Cast

1. Launch MoFA Cast
2. Select format (Plain Text/JSON/Markdown)
3. Click "Import Script"
4. Select your file
5. Review and make minor edits
6. Click "Synthesize Audio"

## Pro Tips

### Quality Checks

Before importing to MoFA Cast, verify:
- ✅ Consistent speaker names (case-sensitive)
- ✅ Clear punctuation for TTS
- ✅ No excessive abbreviations
- ✅ Appropriate length (under 5000 words)

### Common Issues

**Issue**: Speaker not recognized
**Fix**: Use exact names: `host`, `guest1`, `guest2`

**Issue**: Too long pauses
**Fix**: Remove blank lines between segments

**Issue**: Unnatural pronunciation
**Fix**: Add phonetic spelling or rephrase

### Iteration Workflow

1. **First pass**: Basic conversion
2. **Review**: Check for flow issues
3. **Second pass**: Ask AI to:
   - "Make it more engaging"
   - "Add transitions"
   - "Shorten this section"
4. **Final pass**: Format for TTS

## Examples

### Before (Raw Chat Log)
```
[10:00 AM] John: so i was thinking about the project
[10:01 AM] Jane: yeah me too
[10:02 AM] John: we should probably start soon
```

### After (Optimized Script)
```
host: So, I was thinking about our new project.
What are your thoughts?

guest1: Yeah, I've been considering it too.
I think we have a great opportunity here.

host: Absolutely. We should probably start planning soon.
```

## Resources

- **ChatGPT**: https://chatgpt.com
- **Claude**: https://claude.ai
- **MoFA Cast Documentation**: See README.md
- **PrimeSpeech TTS**: https://github.com/FisherWY/PrimeSpeech

---

**Need help?** Check MoFA Cast README for TTS-specific guidance.
