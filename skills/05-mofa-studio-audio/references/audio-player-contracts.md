# AudioPlayer contracts

## 1. Key methods
- `write_audio_with_question(samples, participant_id, question_id)`
- `buffer_fill_percentage()`
- `current_participant_idx()`
- `get_waveform_data()`
- `reset()` / `smart_reset(question_id)`

## 2. Participant tracking
- Participant ID is inferred from input name `audio_<participant>`.
- Question ID drives smart reset and active speaker tracking.

## 3. Waveform data
- `get_waveform_data()` returns latest output waveform for visualization.
