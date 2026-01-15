# Signal contracts

## 1. session_start
- Sent once per `question_id` when the first audio chunk arrives.
- Payload data: `["audio_started"]`.
- Metadata: `question_id`, `participant`, `source`.

## 2. audio_complete
- Sent for each audio chunk received.
- Metadata: `participant`, `question_id`, `session_status`.

## 3. buffer_status
- Percent 0-100 based on actual circular buffer fill.
- Used by controller and text segmenter for backpressure.

## 4. Metadata notes
- `question_id` may be Integer in Dora metadata; convert to string.
- `session_status` can be `started`, `streaming`, `ended` or `complete`.

## 5. Prompt payload
- Prompt input sends JSON `{ "prompt": "..." }` via `control` output.
