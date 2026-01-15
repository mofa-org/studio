# Audio edge cases

- No default output device: show error and stop playback.
- Unsupported sample format: handle I16 and F32; error otherwise.
- Buffer full: circular buffer overwrites; expect drop in oldest samples.
- Missing question_id: smart reset cannot filter; audio may mix between rounds.
- Input naming mismatch: participant becomes `unknown` and UI mapping breaks.
- Sample rate mismatch: waveform levels may appear wrong; ensure 32kHz chain.
