# Audio pipeline

## 1. AudioManager
- Enumerates input/output devices with `get_input_devices()` and `get_output_devices()`.
- Starts mic monitoring on the selected input device.

## 2. UI integration
- Populate dropdown labels with default markers.
- Update mic level meters from `AudioManager::get_mic_level()`.

## 3. Buffer status
- Use `AudioPlayer::buffer_fill_percentage()`.
- Send to Dora via `DoraCommand::UpdateBufferStatus`.
