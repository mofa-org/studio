//! MoFA Cast Screen - Main UI for script to multi-voice podcast transformation

use makepad_widgets::*;
use crate::transcript_parser::{ParserFactory, Transcript, TranscriptFormat};
use crate::tts_batch::{KokoroBackend, ScriptSegmenter, TtsEngineWrapper, TtsFactory, TtsResult};
use crate::audio_mixer::{AudioMixer, AudioMetadata, ExportFormat, MixerConfig, MixerRequest, Mp3Bitrate};
use crate::dora_integration::{DoraIntegration, DoraEvent};
use std::fs;
use std::path::PathBuf;

#[derive(Live, LiveHook, Widget)]
pub struct CastScreen {
    #[deref]
    view: View,

    #[rust]
    transcript: Option<Transcript>,  // For statistics and speaker list

    #[rust]
    script: Option<String>,  // Current script text (from import or editing)

    #[rust]
    current_script_path: Option<PathBuf>,  // Path to currently imported script file

    #[rust]
    current_script_modified: Option<std::time::SystemTime>,  // Track file modification time

    #[rust]
    file_check_timer: Timer,  // Timer for checking external file changes

    #[rust]
    is_synthesizing: bool,

    #[rust]
    tts_result: Option<TtsResult>,

    #[rust]
    is_exporting: bool,

    #[rust]
    speaker_colors: Vec<(String, Vec3)>,

    // Dora Integration
    #[rust]
    dora_integration: Option<DoraIntegration>,

    #[rust]
    dora_dataflow_path: Option<PathBuf>,

    #[rust]
    dora_timer: Timer,

    // Audio collection from Dora
    #[rust]
    collected_audio_segments: Vec<crate::audio_mixer::AudioSegmentInfo>,

    // Exported audio file path (for playback)
    #[rust]
    exported_audio_path: Option<std::path::PathBuf>,

    // Audio playback process (for stopping playback)
    #[rust]
    audio_playback_process: Option<std::process::Child>,

    // Playing state
    #[rust]
    is_playing: bool,

    #[rust]
    total_segments_expected: usize,

    #[rust]
    segments_received: usize,

    // System Log
    #[rust]
    log_entries: Vec<String>,

    #[rust]
    log_level_filter: u32,  // 0=ALL, 1=INFO, 2=WARN, 3=ERROR

    #[rust]
    log_panel_collapsed: bool,

    // Recent files manager (boxed to avoid Live derive issues)
    #[rust]
    recent_files: Option<Box<crate::recent_files::RecentFilesManager>>,

    // Template selection (saved from dropdown)
    #[rust]
    selected_template_id: usize,

    // File format selection (saved from dropdown)
    #[rust]
    selected_format_id: usize,  // 0=Auto, 1=Plain Text, 2=JSON, 3=Markdown

    // Export format selection (saved from dropdown)
    #[rust]
    selected_export_format: usize,  // 0=WAV, 1=MP3

    // MP3 bitrate selection (saved from dropdown)
    #[rust]
    selected_mp3_bitrate: usize,  // 0=128, 1=192, 2=256, 3=320 kbps

    #[rust]
    log_panel_width: f64,
}

impl CastScreen {
    /// Ensure log panel is initialized (call this before using log panel)
    fn ensure_log_initialized(&mut self, cx: &mut Cx) {
        if self.log_entries.is_empty() {
            self.log_entries = Vec::new();
            self.log_level_filter = 0;  // ALL
            self.log_panel_collapsed = false;
            self.log_panel_width = 320.0;

            // Add welcome message directly (not via add_log to avoid recursion)
            self.log_entries.push("[INFO] 🎙️ MoFA Cast - Multi-Voice Podcast Generator".to_string());
            self.log_entries.push("[INFO] Ready to import transcript and generate audio".to_string());
            self.update_log_display(cx);
        }
    }

    /// Ensure recent files are initialized (lazy initialization)
    fn ensure_recent_files_initialized(&mut self, cx: &mut Cx) {
        if self.recent_files.is_none() {
            ::log::info!("Initializing recent files manager");
            self.recent_files = Some(Box::new(crate::recent_files::RecentFilesManager::load()));
            let file_count = self.recent_files.as_ref().map(|m| m.len()).unwrap_or(0);
            ::log::info!("Loaded {} recent files from disk", file_count);

            // Update UI to display loaded files
            self.update_recent_files_ui(cx);
        }
    }
}

impl Widget for CastScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize recent files on first event (lazy initialization)
        self.ensure_recent_files_initialized(cx);

        // Handle timer events for Dora polling (must check before Actions match)
        if self.dora_timer.is_event(event).is_some() {
            ::log::trace!("Dora timer triggered - polling events");
            self.poll_dora_events(cx);
        }

        // Handle timer events for file change detection
        if self.file_check_timer.is_event(event).is_some() {
            self.check_file_changes(cx);
        }

        // Handle button clicks
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => return,
        };

        // Import button
        if self.view.button(ids!(main_content.left_panel.import_section.import_button)).clicked(actions) {
            self.handle_file_import(cx);
        }

        // Format dropdown changed
        if let Some(format_id) = self.view.drop_down(ids!(main_content.left_panel.import_section.format_dropdown)).selected(actions) {
            ::log::info!("Format dropdown changed to: {}", format_id);
            self.selected_format_id = format_id;
        }

        // Template dropdown changed - auto-load template
        if let Some(template_id) = self.view.drop_down(ids!(main_content.right_panel.templates_section.template_dropdown)).selected(actions) {
            ::log::info!("Template dropdown changed to: {}", template_id);
            self.selected_template_id = template_id;
            // Auto-load template when dropdown changes
            self.load_template(cx, template_id);
        }

        // Use template button (optional - can still use button after selecting from dropdown)
        if self.view.button(ids!(main_content.right_panel.templates_section.use_template_button)).clicked(actions) {
            ::log::info!("Use Template button clicked! Using saved template ID: {}", self.selected_template_id);
            self.load_template(cx, self.selected_template_id);
        }

        // Refine button
        

        // Synthesize button
        if self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button)).clicked(actions) {
            self.handle_synthesize_audio(cx);
        }

        // Open in Editor button
        if self.view.button(ids!(main_content.right_panel.control_bar.open_editor_button)).clicked(actions) {
            self.handle_open_in_editor(cx);
        }

        // Export format dropdown changed
        if let Some(format_id) = self.view.drop_down(ids!(main_content.right_panel.control_bar.export_format_dropdown)).selected(actions) {
            ::log::info!("Export format changed to: {}", if format_id == 0 { "WAV" } else { "MP3" });
            self.selected_export_format = format_id;
        }

        // MP3 bitrate dropdown changed
        if let Some(bitrate_id) = self.view.drop_down(ids!(main_content.right_panel.control_bar.mp3_bitrate_dropdown)).selected(actions) {
            let bitrate_names = ["128", "192", "256", "320"];
            ::log::info!("MP3 bitrate changed to: {} kbps", bitrate_names[bitrate_id]);
            self.selected_mp3_bitrate = bitrate_id;
        }

        // Export button
        if self.view.button(ids!(main_content.right_panel.control_bar.export_button)).clicked(actions) {
            self.handle_export_audio(cx);
        }

        // Audio player buttons
        if self.view.button(ids!(main_content.right_panel.content_area.audio_player_section.playback_controls.play_button)).clicked(actions) {
            self.handle_play_audio(cx);
        }

        if self.view.button(ids!(main_content.right_panel.content_area.audio_player_section.playback_controls.stop_button)).clicked(actions) {
            self.handle_stop_audio(cx);
        }

        if self.view.button(ids!(main_content.right_panel.content_area.audio_player_section.playback_controls.open_in_player_button)).clicked(actions) {
            self.handle_open_in_player(cx);
        }

        // Toggle log panel button
        if self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).clicked(actions) {
            self.toggle_log_panel(cx);
        }

        // Log level filter dropdown
        if let Some(selected) = self.view.drop_down(ids!(log_section.log_content_column.log_header.log_filter_row.level_filter)).selected(actions) {
            self.log_level_filter = selected as u32;
            self.update_log_display(cx);
        }

        // Clear log button
        if self.view.button(ids!(log_section.log_content_column.log_header.log_filter_row.clear_log_btn)).clicked(actions) {
            self.clear_logs(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

// Helper methods for CastScreen
impl CastScreen {
    /// Handle file import button click
    fn handle_file_import(&mut self, cx: &mut Cx) {
        // Use rfd to open file dialog
        // Note: On macOS, file dialogs must run on main thread
        // We need to handle this carefully to avoid blocking UI

        ::log::info!("Opening file dialog...");
        self.add_log(cx, "[INFO] 📂 Opening file dialog...");

        // Try to open file dialog
        let file_handle = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt", "json", "md"])
            .add_filter("All Files", &["*"])
            .set_title("Select Transcript File")
            .pick_file();

        match file_handle {
            Some(file_path) => {
                ::log::info!("File selected: {:?}", file_path);
                self.add_log(cx, &format!("[INFO] ✅ File selected: {}", file_path.display()));

                // Read file content
                match fs::read_to_string(&file_path) {
                    Ok(content) => {
                        ::log::info!("File read successfully: {} bytes", content.len());
                        self.add_log(cx, &format!("[INFO] ✅ File read successfully: {} bytes", content.len()));

                        // Create parser factory and parse the transcript
                        let parser_factory = ParserFactory::new();

                        // Parse transcript based on selected format
                        let parse_result = if self.selected_format_id == 0 {
                            // Auto detect
                            parser_factory.parse_auto(&content)
                        } else {
                            // Use specific format parser
                            let format = match self.selected_format_id {
                                1 => TranscriptFormat::PlainText,
                                2 => TranscriptFormat::Json,
                                3 => TranscriptFormat::Markdown,
                                _ => TranscriptFormat::PlainText,
                            };

                            ::log::info!("Using format: {:?}", format);
                            parser_factory.parse_with_format(&content, format)
                        };

                        match parse_result {
                            Ok(transcript) => {
                                ::log::info!("Transcript parsed successfully: {} speakers, {} messages",
                                    transcript.metadata.participants.len(),
                                    transcript.message_count());
                                self.add_log(cx, &format!("[INFO] ✅ Parsed: {} speakers, {} messages",
                                    transcript.metadata.participants.len(),
                                    transcript.message_count()));

                                // Store the transcript
                                self.transcript = Some(transcript.clone());

                                // Add to recent files
                                if let Some(ref mut manager) = self.recent_files {
                                    let recent_file = crate::recent_files::RecentFile::new(file_path.clone(), &transcript);
                                    manager.add(recent_file);

                                    // Update recent files UI
                                    self.update_recent_files_ui(cx);
                                }

                                // Update UI with parsed content
                                let path_str = file_path.to_string_lossy().to_string();
                                self.update_ui_with_transcript(cx, &transcript, &path_str);

                                // Clear previous TTS result
                                self.tts_result = None;

                                // Clear progress label
                                self.view.label(ids!(header.header_description))
                                    .set_text(cx, "");
                            }
                            Err(e) => {
                                ::log::error!("Failed to parse transcript: {}", e);
                                self.add_log(cx, &format!("[ERROR] ❌ Failed to parse transcript: {}", e));
                                // Show error message in UI
                                let error_msg = format!("Parse error: {}", e);
                                self.view.label(ids!(main_content.left_panel.import_section.file_info))
                                    .set_text(cx, &error_msg);

                                eprintln!("Failed to parse transcript: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        ::log::error!("Failed to read file: {}", e);
                        self.add_log(cx, &format!("[ERROR] ❌ Failed to read file: {}", e));
                        // Show error message
                        let error_msg = format!("Error reading file: {}", e);
                        self.view.label(ids!(main_content.left_panel.import_section.file_info))
                            .set_text(cx, &error_msg);

                        eprintln!("Failed to read file: {}", e);
                    }
                }
            }
            None => {
                ::log::info!("File dialog cancelled by user");
                self.add_log(cx, "[INFO] File dialog cancelled");
                // User cancelled the dialog
                self.view.label(ids!(main_content.left_panel.import_section.file_info))
                    .set_text(cx, "No file selected");
            }
        }

        self.view.redraw(cx);
    }

    /// Update UI elements with parsed transcript data
    fn update_ui_with_transcript(&mut self, cx: &mut Cx, transcript: &Transcript, file_path: &str) {
        // Update file info label
        let file_name = std::path::Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown file");

        let info_text = format!(
            "{}\n{} messages • {} speakers",
            file_name,
            transcript.message_count(),
            transcript.metadata.participants.len()
        );

        self.view.label(ids!(main_content.left_panel.import_section.file_info))
            .set_text(cx, &info_text);

        // Generate colors for speakers
        self.generate_speaker_colors(transcript);

        // Format transcript for display and store
        let script_text = self.format_transcript_for_display(transcript);
        self.script = Some(script_text.clone());

        // Store file path and modification time for external editor support
        self.current_script_path = Some(PathBuf::from(file_path));
        if let Ok(metadata) = fs::metadata(file_path) {
            if let Ok(modified) = metadata.modified() {
                self.current_script_modified = Some(modified);
            }
        }

        // Restart file change detection timer
        cx.stop_timer(self.file_check_timer);
        self.file_check_timer = cx.start_interval(2.0);

        self.view.text_input(ids!(main_content.right_panel.editor_container.script_editor))
            .set_text(cx, &script_text);

        // Update speakers list
        self.update_speakers_list(cx, transcript);

        // Redraw the UI
        self.view.redraw(cx);
    }

    /// Generate colors for each speaker
    fn generate_speaker_colors(&mut self, transcript: &Transcript) {
        self.speaker_colors.clear();

        let speakers = transcript.get_speakers();
        let predefined_colors = vec![
            Vec3 { x: 0.24, y: 0.51, z: 0.96 },  // Blue #3b82f6
            Vec3 { x: 0.06, y: 0.73, z: 0.50 },  // Green #10b981
            Vec3 { x: 0.98, y: 0.38, z: 0.26 },  // Red #f97316
            Vec3 { x: 0.55, y: 0.36, z: 0.96 },  // Purple #8b5cf6
            Vec3 { x: 0.93, y: 0.27, z: 0.51 },  // Pink #ef4444
            Vec3 { x: 0.96, y: 0.62, z: 0.07 },  // Orange #f59e0b
            Vec3 { x: 0.14, y: 0.73, z: 0.73 },  // Teal #0d9488
            Vec3 { x: 0.72, y: 0.11, z: 0.65 },  // Magenta #b81d7d
        ];

        for (i, speaker) in speakers.iter().enumerate() {
            let color = predefined_colors.get(i % predefined_colors.len()).copied().unwrap();
            self.speaker_colors.push((speaker.name.clone(), color));
        }
    }

    /// Format transcript as human-readable text
    fn format_transcript_for_display(&self, transcript: &Transcript) -> String {
        let mut text = String::new();

        for msg in &transcript.messages {
            text.push_str(&format!("{}: {}\n", msg.speaker, msg.text));
        }

        text
    }

    /// Update the speakers list in the left panel
    fn update_speakers_list(&mut self, cx: &mut Cx, transcript: &Transcript) {
        let speakers = transcript.get_speakers();

        if speakers.is_empty() {
            // Show placeholder
            self.view.label(ids!(main_content.left_panel.speakers_section.speakers_list.placeholder))
                .set_text(cx, "No speakers found");
        } else {
            // For now, just update the placeholder with summary
            // TODO: Implement dynamic speaker list widget
            let summary = speakers.iter()
                .map(|s| format!("{} ({} msgs)", s.name, s.message_count))
                .collect::<Vec<_>>()
                .join("\n");

            self.view.label(ids!(main_content.left_panel.speakers_section.speakers_list.placeholder))
                .set_text(cx, &summary);
        }
    }

    /// Handle script refinement


    /// Initialize default values for rust fields
    fn ensure_defaults(&mut self) {
        // Initialize TTS engine configuration
        // Set environment variable to use Kokoro: MOFA_CAST_TTS=kokoro
        // Otherwise defaults to Mock for testing
    }

    /// Write audio samples to WAV file
    fn write_wav_file(&self, path: &std::path::Path, samples: &[i16], sample_rate: u32, channels: u16) -> Result<(), String> {
        use std::io::Write;

        // Create file
        let mut file = std::fs::File::create(path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        // WAV header parameters
        let byte_rate = sample_rate * channels as u32 * 2;
        let block_align = channels * 2;
        let data_size = samples.len() * 2;
        let file_size = 36 + data_size as u32;

        // Write RIFF header
        file.write_all(b"RIFF").map_err(|e| format!("Failed to write RIFF: {}", e))?;
        file.write_all(&file_size.to_le_bytes()).map_err(|e| format!("Failed to write file size: {}", e))?;
        file.write_all(b"WAVE").map_err(|e| format!("Failed to write WAVE: {}", e))?;

        // Write fmt chunk
        file.write_all(b"fmt ").map_err(|e| format!("Failed to write fmt: {}", e))?;
        file.write_all(&16u32.to_le_bytes()).map_err(|e| format!("Failed to write fmt size: {}", e))?; // PCM chunk size
        file.write_all(&1u16.to_le_bytes()).map_err(|e| format!("Failed to write audio format: {}", e))?; // Audio format (PCM)
        file.write_all(&channels.to_le_bytes()).map_err(|e| format!("Failed to write channels: {}", e))?;
        file.write_all(&sample_rate.to_le_bytes()).map_err(|e| format!("Failed to write sample rate: {}", e))?;
        file.write_all(&byte_rate.to_le_bytes()).map_err(|e| format!("Failed to write byte rate: {}", e))?;
        file.write_all(&block_align.to_le_bytes()).map_err(|e| format!("Failed to write block align: {}", e))?;
        file.write_all(&16u16.to_le_bytes()).map_err(|e| format!("Failed to write bits per sample: {}", e))?; // Bits per sample

        // Write data chunk
        file.write_all(b"data").map_err(|e| format!("Failed to write data: {}", e))?;
        file.write_all(&(data_size as u32).to_le_bytes()).map_err(|e| format!("Failed to write data size: {}", e))?;

        // Write sample data
        for sample in samples {
            file.write_all(&sample.to_le_bytes()).map_err(|e| format!("Failed to write sample: {}", e))?;
        }

        Ok(())
    }

    /// Create TTS engine based on current configuration
    fn create_tts_engine(&self) -> Result<TtsEngineWrapper, String> {
        // Check environment variable for engine selection
        let tts_engine = std::env::var("MOFA_CAST_TTS").unwrap_or_default();

        match tts_engine.as_str() {
            "kokoro" => {
                ::log::info!("Using Dora Kokoro TTS engine (local, real TTS)");
                let engine = TtsFactory::create_dora_kokoro_engine()
                    .with_backend(KokoroBackend::Auto)
                    .with_language("en")
                    .with_voice("af_heart")
                    .with_speed(1.0);
                Ok(TtsEngineWrapper::Kokoro(engine))
            }
            "mock" | "" | _ => {
                ::log::info!("Using Mock TTS engine (test tones only)");
                ::log::info!("To use real TTS, set environment variable: MOFA_CAST_TTS=kokoro");
                Ok(TtsEngineWrapper::Mock(TtsFactory::create_mock_engine()))
            }
        }
    }

    /// Handle TTS synthesis using Dora dataflow
    fn handle_synthesize_audio(&mut self, cx: &mut Cx) {
        // Get script text from stored state
        let script_text = match self.script {
            Some(ref script) => script.clone(),
            None => {
                // Show error message
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "⚠️ Please import a script first");
                self.view.redraw(cx);
                return;
            }
        };

        // Show loading state
        self.is_synthesizing = true;
        self.view.label(ids!(header.header_description))
            .set_text(cx, "🎙️ Starting Dora dataflow...");
        self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
            .set_enabled(cx, false);
        self.view.button(ids!(main_content.right_panel.control_bar.export_button))
            .set_enabled(cx, false);
        self.view.redraw(cx);

        // Initialize Dora integration
        self.init_dora(cx);

        // Create segmenter and parse script
        let segmenter = match ScriptSegmenter::new() {
            Ok(s) => s,
            Err(e) => {
                let error_msg = format!("❌ Segmenter error: {}", e);
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Script segmentation error: {:?}", e);
                self.is_synthesizing = false;
                self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                    .set_enabled(cx, true);
                self.view.redraw(cx);
                return;
            }
        };

        let segments = match segmenter.segment_script(&script_text) {
            Ok(segments) => segments,
            Err(e) => {
                let error_msg = format!("❌ Segmentation error: {}", e);
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Script segmentation error: {:?}", e);
                self.is_synthesizing = false;
                self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                    .set_enabled(cx, true);
                self.view.redraw(cx);
                return;
            }
        };

        // Initialize audio collection state
        self.total_segments_expected = segments.len();
        self.segments_received = 0;
        self.collected_audio_segments.clear();

        ::log::info!("Expecting {} audio segments from Dora dataflow", self.total_segments_expected);

        // Get speakers list for voice mapping
        let speakers: Vec<String> = segments.iter().map(|s| s.speaker.clone()).collect();
        let voice_mapping = crate::dora_integration::VoiceMapping::from_speakers(&speakers);

        // Log the voice mapping
        ::log::info!("Voice mapping for {} speakers:", voice_mapping.voices.len());
        for voice_config in &voice_mapping.voices {
            ::log::info!("  '{}' → '{}' (speed: {:.1})",
                       voice_config.speaker, voice_config.voice_name, voice_config.speed);
        }

        // Convert to Dora script segments with voice information
        let default_voice = crate::dora_integration::VoiceConfig::new("unknown", "Luo Xiang", 1.0);
        let dora_segments: Vec<crate::dora_integration::ScriptSegment> = segments
            .iter()
            .enumerate()
            .map(|(idx, seg)| {
                // Normalize speaker names (merge [主持人] and host into one)
                let normalized_speaker = self.normalize_speaker_name(&seg.speaker);

                // Get voice config for normalized speaker
                let voice_config = voice_mapping.get_voice_for_speaker(&normalized_speaker)
                    .unwrap_or(&default_voice);

                crate::dora_integration::ScriptSegment {
                    speaker: normalized_speaker,
                    text: seg.text.clone(),
                    segment_index: idx,
                    voice_name: voice_config.voice_name.clone(),
                    speed: voice_config.speed,
                }
            })
            .collect();

        // Save segment count before moving dora_segments
        let segment_count = dora_segments.len();

        // Get dataflow path
        let dataflow_path = match &self.dora_dataflow_path {
            Some(path) => path.clone(),
            None => {
                let error_msg = "❌ Dataflow configuration not found".to_string();
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Dataflow path not set");
                self.is_synthesizing = false;
                self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                    .set_enabled(cx, true);
                self.view.redraw(cx);
                return;
            }
        };

        // Start Dora dataflow with configuration
        if let Some(ref dora) = self.dora_integration {
            // Set voice mapping before starting dataflow
            if !dora.set_voice_mapping(voice_mapping) {
                ::log::warn!("Failed to set voice mapping (non-critical)");
            }

            if !dora.start_dataflow(dataflow_path.clone()) {
                let error_msg = "❌ Failed to start dataflow".to_string();
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Failed to start dataflow");
                self.is_synthesizing = false;
                self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                    .set_enabled(cx, true);
                self.view.redraw(cx);
                return;
            }

            // Send script segments for synthesis
            if !dora.send_script_segments(dora_segments) {
                let error_msg = "❌ Failed to send script segments".to_string();
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Failed to send script segments");
                self.is_synthesizing = false;
                self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                    .set_enabled(cx, true);
                self.view.redraw(cx);
                return;
            }

            ::log::info!("Started Dora TTS synthesis with {} segments", segment_count);
            self.view.label(ids!(header.header_description))
                .set_text(cx, &format!("🎙️ Synthesizing {} segments via Dora...", segment_count));
            self.view.redraw(cx);
        } else {
            let error_msg = "❌ Dora integration not available".to_string();
            self.view.label(ids!(header.header_description))
                .set_text(cx, &error_msg);
            ::log::error!("Dora integration is None");
            self.is_synthesizing = false;
            self.view.button(ids!(main_content.right_panel.control_bar.synthesize_button))
                .set_enabled(cx, true);
            self.view.redraw(cx);
        }
    }

    /// Load a template by ID
    fn load_template(&mut self, cx: &mut Cx, template_id: usize) {
        use crate::script_templates::TemplateType;

        let template_type = match template_id {
            0 => TemplateType::TwoPersonInterview,
            1 => TemplateType::ThreePersonDiscussion,
            2 => TemplateType::Narrative,
            _ => {
                self.add_log(cx, "[WARN] Invalid template ID, defaulting to 2-Person Interview");
                TemplateType::TwoPersonInterview
            }
        };

        let template_type_clone = template_type.clone();
        let template_name = template_type.display_name().to_string();
        let template = crate::script_templates::ScriptTemplate::get_template(template_type_clone);

        ::log::info!("Loading template: {}, content length: {}", template_name, template.content.len());
        ::log::info!("Template content preview: {}", &template.content.chars().take(100).collect::<String>());

        // Display template in script editor
        self.view.text_input(ids!(main_content.right_panel.editor_container.script_editor))
            .set_text(cx, &template.content);

        self.script = Some(template.content.clone());

        let msg = format!("✅ Loaded template: {}", template_name);
        ::log::info!("{}", msg);
        self.add_log(cx, &format!("[INFO] {}", msg));
        self.view.label(ids!(header.header_description))
            .set_text(cx, &msg);

        // Create a temporary file for external editor support
        let temp_dir = std::env::temp_dir();
        let temp_filename = format!("mofa_cast_template_{}.txt", template_name.to_lowercase().replace(' ', "_"));
        let temp_path = temp_dir.join(&temp_filename);

        match fs::write(&temp_path, &template.content) {
            Ok(_) => {
                self.current_script_path = Some(temp_path.clone());
                if let Ok(metadata) = fs::metadata(&temp_path) {
                    if let Ok(modified) = metadata.modified() {
                        self.current_script_modified = Some(modified);
                    }
                }
                ::log::info!("Created temp file for template: {}", temp_path.display());

                // Update file info to show temp file location
                let file_info_msg = format!("Template (temp file: {})", temp_filename);
                self.view.label(ids!(main_content.left_panel.import_section.file_info))
                    .set_text(cx, &file_info_msg);
            }
            Err(e) => {
                ::log::warn!("Failed to create temp file for template: {}", e);
                // Still clear file info since this is a template
                self.view.label(ids!(main_content.left_panel.import_section.file_info))
                    .set_text(cx, "Template loaded - External editor unavailable");
            }
        }

        self.view.redraw(cx);
    }

    /// Handle "Open in Editor" button click
    fn handle_open_in_editor(&mut self, cx: &mut Cx) {
        // Check if we have a current script file
        let file_path = match &self.current_script_path {
            Some(path) => path.clone(),
            None => {
                self.add_log(cx, "[WARN] ⚠️ No script file loaded. Please import a script first.");
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "⚠️ No script file loaded");
                self.view.redraw(cx);
                return;
            }
        };

        // Open file with default editor
        #[cfg(target_os = "macos")]
        let command = "open";
        #[cfg(target_os = "linux")]
        let command = "xdg-open";
        #[cfg(target_os = "windows")]
        let command = "start";

        let result = std::process::Command::new(command)
            .arg(&file_path)
            .spawn();

        match result {
            Ok(_) => {
                let msg = format!("✅ Opened in external editor: {}", file_path.display());
                ::log::info!("{}", msg);
                self.add_log(cx, &format!("[INFO] {}", msg));
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "✅ Opened in external editor");
            }
            Err(e) => {
                let msg = format!("❌ Failed to open editor: {}", e);
                ::log::error!("{}", msg);
                self.add_log(cx, &format!("[ERROR] {}", msg));
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "❌ Failed to open editor");
            }
        }

        self.view.redraw(cx);
    }

    /// Update recent files UI
    fn update_recent_files_ui(&mut self, cx: &mut Cx) {
        let recent_files = match &self.recent_files {
            Some(manager) => manager.get_all().to_vec(),
            None => {
                // Show placeholder
                self.view.label(ids!(main_content.left_panel.recent_files_section.recent_files_placeholder))
                    .set_text(cx, "No recent files");
                return;
            }
        };

        if recent_files.is_empty() {
            self.view.label(ids!(main_content.left_panel.recent_files_section.recent_files_placeholder))
                .set_text(cx, "No recent files");
            return;
        }

        // Hide placeholder and show list as text
        self.view.label(ids!(main_content.left_panel.recent_files_section.recent_files_placeholder))
            .set_text(cx, "");

        // Build a formatted list of recent files as text
        // Note: Makepad doesn't support dynamic widget creation, so we display as formatted text
        let mut display_text = String::new();
        for (i, file) in recent_files.iter().enumerate() {
            if i > 0 {
                display_text.push('\n');
            }
            display_text.push_str(&format!("{}. {}", i + 1, file.name));
        }

        // Log the files for debugging
        ::log::info!("Recent files ({} total):", recent_files.len());
        for (i, file) in recent_files.iter().enumerate() {
            ::log::info!("  {}: {} ({} msgs, {} speakers)",
                i + 1, file.name, file.message_count, file.speaker_count);
        }

        // Update the label with the formatted list
        // For now, we'll show count + first file name to keep it simple
        let summary = if recent_files.len() == 1 {
            format!("1 recent file:\n{}", recent_files[0].name)
        } else {
            format!("{} recent files:\n{}", recent_files.len(), recent_files[0].name)
        };

        self.view.label(ids!(main_content.left_panel.recent_files_section.recent_files_placeholder))
            .set_text(cx, &summary);
    }

    /// Check for external file changes and reload if modified
    fn check_file_changes(&mut self, cx: &mut Cx) {
        // Only check if we have a file path
        let file_path = match &self.current_script_path {
            Some(path) => path.clone(),
            None => return,
        };

        // Get current modification time
        let current_modified = match fs::metadata(&file_path)
            .and_then(|m| m.modified()) {
            Ok(time) => time,
            Err(e) => {
                ::log::warn!("Failed to check file modification time: {}", e);
                return;
            }
        };

        // Check if file was modified
        let should_reload = match self.current_script_modified {
            Some(previous) => current_modified != previous,
            None => false,
        };

        if !should_reload {
            return;
        }

        // File was modified, reload it
        ::log::info!("File modified externally, reloading: {}", file_path.display());
        self.add_log(cx, "[INFO] 📝 File changed externally, reloading...");

        match fs::read_to_string(&file_path) {
            Ok(content) => {
                // Try to parse and update
                let parser_factory = ParserFactory::new();
                match parser_factory.parse_auto(&content) {
                    Ok(transcript) => {
                        // Update stored modification time
                        self.current_script_modified = Some(current_modified);

                        // Update UI with new content
                        let path_str = file_path.to_string_lossy().to_string();
                        self.update_ui_with_transcript(cx, &transcript, &path_str);

                        let msg = format!("✅ Reloaded: {} messages", transcript.message_count());
                        ::log::info!("{}", msg);
                        self.add_log(cx, &format!("[INFO] {}", msg));
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, "✅ Script reloaded from file");
                    }
                    Err(e) => {
                        let msg = format!("Failed to parse modified file: {}", e);
                        ::log::warn!("{}", msg);
                        self.add_log(cx, &format!("[WARN] {}", msg));
                    }
                }
            }
            Err(e) => {
                let msg = format!("Failed to read modified file: {}", e);
                ::log::error!("{}", msg);
                self.add_log(cx, &format!("[ERROR] {}", msg));
            }
        }

        self.view.redraw(cx);
    }

    /// Handle audio export
    fn handle_export_audio(&mut self, cx: &mut Cx) {
        // Check if we have collected audio segments from Dora
        if self.collected_audio_segments.is_empty() {
            // Show error message
            self.view.label(ids!(header.header_description))
                .set_text(cx, "⚠️ No audio segments collected. Please synthesize audio first");
            self.view.redraw(cx);
            return;
        }

        // Show loading state
        self.is_exporting = true;
        self.view.label(ids!(header.header_description))
            .set_text(cx, "📥 Mixing and exporting audio...");
        self.view.button(ids!(main_content.right_panel.control_bar.export_button))
            .set_enabled(cx, false);
        self.view.redraw(cx);

        ::log::info!("Exporting {} audio segments", self.collected_audio_segments.len());

        // Get sample rate from first segment (PrimeSpeech uses 32000Hz)
        let first_sample_rate = self.collected_audio_segments.first()
            .map(|s| s.sample_rate)
            .unwrap_or(32000);  // Default to PrimeSpeech rate

        let first_channels = self.collected_audio_segments.first()
            .map(|s| s.channels)
            .unwrap_or(1);  // Default to mono

        ::log::info!("Using sample rate: {}Hz, channels: {}", first_sample_rate, first_channels);

        // Create mixer config with detected sample rate and user-selected format
        let output_dir = std::path::PathBuf::from("./output/mofa-cast");

        // Determine export format
        let export_format = match self.selected_export_format {
            0 => ExportFormat::Wav,
            1 => ExportFormat::Mp3,
            _ => ExportFormat::Wav,
        };

        // Determine MP3 bitrate (only used if export_format is MP3)
        let mp3_bitrate = match self.selected_mp3_bitrate {
            0 => Mp3Bitrate::Kbps128,
            1 => Mp3Bitrate::Kbps192,
            2 => Mp3Bitrate::Kbps256,
            3 => Mp3Bitrate::Kbps320,
            _ => Mp3Bitrate::Kbps192,
        };

        ::log::info!("Export format: {:?}, MP3 bitrate: {:?}", export_format, mp3_bitrate);

        // Generate metadata from current script
        let metadata_title = self.current_script_path.as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Podcast".to_string());

        let metadata = AudioMetadata {
            title: Some(metadata_title.clone()),
            artist: Some("MoFA Cast".to_string()),
            album: Some("Generated by MoFA Cast".to_string()),
            year: Some(chrono::Utc::now().format("%Y").to_string()),
            comment: Some(format!("Created with MoFA Cast v0.6.2 - {} segments",
                self.collected_audio_segments.len())),
        };

        ::log::info!("Using metadata: title={}, artist={}, album={}",
            metadata_title, "MoFA Cast", "Generated by MoFA Cast");

        let config = MixerConfig {
            output_path: output_dir.join("podcast"),
            export_format,
            mp3_bitrate,
            normalize_dB: -14.0,  // EBU R128 standard
            silence_duration_secs: 0.5,
            sample_rate: first_sample_rate,  // Use detected sample rate
            channels: first_channels,         // Use detected channels
            bits_per_sample: 16,             // PrimeSpeech uses 16-bit
            metadata,
        };

        let request = MixerRequest {
            segments: self.collected_audio_segments.clone(),
            config,
        };

        // Create mixer and mix
        let mixer = AudioMixer::new();
        match mixer.mix(request) {
            Ok(result) => {
                // Update progress label with success message
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &format!(
                        "✅ Exported! {:.1}s audio • {}KB",
                        result.total_duration_secs,
                        result.file_size_bytes / 1024
                    ));

                // Update audio player UI
                let format_name = match export_format {
                    ExportFormat::Wav => "WAV",
                    ExportFormat::Mp3 => "MP3",
                };

                self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.player_status))
                    .set_text(cx, "Ready to play");

                self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.audio_info.format_label))
                    .set_text(cx, &format!("Format: {}", format_name));

                self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.audio_info.duration_label))
                    .set_text(cx, &format!("Duration: {:.1} seconds", result.total_duration_secs));

                self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.audio_info.file_size_label))
                    .set_text(cx, &format!("Size: {} KB", result.file_size_bytes / 1024));

                // Store exported audio path for playback
                self.exported_audio_path = Some(result.output_file.clone());

                // Log success message with file location
                ::log::info!("Audio exported successfully: {:.1}s, {}KB",
                    result.total_duration_secs, result.file_size_bytes / 1024);

                ::log::info!("Export file: {}", result.output_file.display());
            }
            Err(e) => {
                // Show error
                let error_msg = format!("❌ Export failed: {}", e);
                self.view.label(ids!(header.header_description))
                    .set_text(cx, &error_msg);
                ::log::error!("Audio export error: {:?}", e);
            }
        }

        self.is_exporting = false;
        self.view.button(ids!(main_content.right_panel.control_bar.export_button))
            .set_enabled(cx, true);
        self.view.redraw(cx);
    }

    /// Handle play audio button
    fn handle_play_audio(&mut self, cx: &mut Cx) {
        if let Some(ref audio_path) = self.exported_audio_path {
            if audio_path.exists() {
                // If already playing, don't start another
                if self.is_playing {
                    self.view.label(ids!(header.header_description))
                        .set_text(cx, "⚠️ Already playing. Stop first.");
                    self.view.redraw(cx);
                    return;
                }

                // Determine platform-specific player
                #[cfg(target_os = "macos")]
                let player = "afplay";  // macOS default audio player
                #[cfg(target_os = "windows")]
                let player = "wmplayer";  // Windows Media Player
                #[cfg(target_os = "linux")]
                let player = "vlc";  // VLC (common on Linux)

                ::log::info!("Playing audio file");

                let result = std::process::Command::new(player)
                    .arg(audio_path)
                    .spawn();

                match result {
                    Ok(mut child) => {
                        // Store the child process
                        self.audio_playback_process = Some(child);
                        self.is_playing = true;

                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("▶️ Playing: {}",
                                audio_path.file_name().unwrap_or_default().to_string_lossy()));

                        self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.player_status))
                            .set_text(cx, "Playing");

                        ::log::info!("Started playback: {}", audio_path.display());
                    }
                    Err(e) => {
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("❌ Failed to play: {}", e));
                        ::log::error!("Failed to start audio player: {}", e);
                    }
                }

                self.view.redraw(cx);
            } else {
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "⚠️ Audio file not found. Please export first.");
                self.view.redraw(cx);
            }
        } else {
            self.view.label(ids!(header.header_description))
                .set_text(cx, "⚠️ No audio exported yet. Please export first.");
            self.view.redraw(cx);
        }
    }

    /// Handle stop audio button
    fn handle_stop_audio(&mut self, cx: &mut Cx) {
        if !self.is_playing {
            self.view.label(ids!(header.header_description))
                .set_text(cx, "⚠️ No audio playing");
            self.view.redraw(cx);
            return;
        }

        // Kill the playback process
        if let Some(mut child) = self.audio_playback_process.take() {
            match child.kill() {
                Ok(_) => {
                    self.is_playing = false;

                    self.view.label(ids!(header.header_description))
                        .set_text(cx, "⏹ Playback stopped");

                    self.view.label(ids!(main_content.right_panel.content_area.audio_player_section.player_status))
                        .set_text(cx, "Stopped");

                    ::log::info!("Playback stopped");
                }
                Err(e) => {
                    self.view.label(ids!(header.header_description))
                        .set_text(cx, &format!("❌ Failed to stop: {}", e));
                    ::log::error!("Failed to kill audio player: {}", e);
                }
            }
        }

        self.view.redraw(cx);
    }

    /// Handle "Open in Player" button
    fn handle_open_in_player(&mut self, cx: &mut Cx) {
        if let Some(ref audio_path) = self.exported_audio_path {
            if audio_path.exists() {
                // Use `open` crate to open with system default
                let result = open::that(audio_path);

                match result {
                    Ok(_) => {
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("🔓 Opened in external player: {}",
                                audio_path.file_name().unwrap_or_default().to_string_lossy()));
                        ::log::info!("Opened audio file: {}", audio_path.display());
                    }
                    Err(e) => {
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("❌ Failed to open: {}", e));
                        ::log::error!("Failed to open audio file: {}", e);
                    }
                }

                self.view.redraw(cx);
            } else {
                self.view.label(ids!(header.header_description))
                    .set_text(cx, "⚠️ Audio file not found. Please export first.");
                self.view.redraw(cx);
            }
        } else {
            self.view.label(ids!(header.header_description))
                .set_text(cx, "⚠️ No audio exported yet. Please export first.");
            self.view.redraw(cx);
        }
    }

    /// Normalize speaker names to merge duplicates
    /// Maps various speaker name variants to unified names
    /// Examples: "[主持人]" -> "host", "Host" -> "host", etc.
    fn normalize_speaker_name(&self, speaker: &str) -> String {
        match speaker {
            // Chinese主持人 variants -> host
            s if s.contains("主持人") => "host".to_string(),
            s if s.contains("主持") => "host".to_string(),

            // Case-insensitive host mapping
            "Host" | "HOST" => "host".to_string(),

            // Case-insensitive guest mappings
            "Guest1" | "GUEST1" => "guest1".to_string(),
            "Guest2" | "GUEST2" => "guest2".to_string(),
            "Guest" | "GUEST" => "guest1".to_string(), // Default to guest1

            // Remove brackets and special characters
            s if s.starts_with('[') && s.ends_with(']') => {
                s[1..s.len()-1].to_string()
            }

            // Return as-is if no mapping found
            _ => speaker.to_string(),
        }
    }

    // =====================================================
    // LOG PANEL METHODS
    // =====================================================

    /// Toggle log panel visibility
    fn toggle_log_panel(&mut self, cx: &mut Cx) {
        self.log_panel_collapsed = !self.log_panel_collapsed;

        if self.log_panel_width == 0.0 {
            self.log_panel_width = 320.0;
        }

        if self.log_panel_collapsed {
            // Collapse: hide log content, show only toggle button
            self.view.view(ids!(log_section)).apply_over(cx, live!{ width: Fit });
            self.view.view(ids!(log_section.log_content_column)).set_visible(cx, false);
            self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).set_text(cx, ">");
        } else {
            // Expand: show log content at saved width
            let width = self.log_panel_width;
            self.view.view(ids!(log_section)).apply_over(cx, live!{ width: (width) });
            self.view.view(ids!(log_section.log_content_column)).set_visible(cx, true);
            self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).set_text(cx, "<");
        }

        self.view.redraw(cx);
    }

    /// Update log display based on current filter
    fn update_log_display(&mut self, cx: &mut Cx) {
        let level_filter = self.log_level_filter;

        // Filter log entries
        let filtered_logs: Vec<&String> = self.log_entries.iter().filter(|entry| {
            // Level filter: 0=ALL, 1=INFO, 2=WARN, 3=ERROR
            match level_filter {
                0 => true, // ALL
                1 => entry.contains("[INFO]") || entry.contains("[INFO]"),
                2 => entry.contains("[WARN]") || entry.contains("[WARNING]"),
                3 => entry.contains("[ERROR]"),
                _ => true,
            }
        }).collect();

        // Build display text (use double newlines for Markdown paragraph breaks)
        let log_text = if filtered_logs.is_empty() {
            "*No log entries*".to_string()
        } else {
            filtered_logs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n\n")
        };

        // Update markdown display
        self.view.markdown(ids!(log_section.log_content_column.log_scroll.log_content_wrapper.log_content)).set_text(cx, &log_text);
        self.view.redraw(cx);
    }

    /// Add a log entry
    fn add_log(&mut self, cx: &mut Cx, entry: &str) {
        self.ensure_log_initialized(cx);
        self.log_entries.push(entry.to_string());
        self.update_log_display(cx);
    }

    /// Clear all logs
    fn clear_logs(&mut self, cx: &mut Cx) {
        self.log_entries.clear();
        self.update_log_display(cx);
    }

    /// Poll Rust log messages and add them to the system log
    fn poll_rust_logs(&mut self, cx: &mut Cx) {
        // For now, this is a placeholder
        // In the future, we can integrate with the env logger
        // For now, we'll just capture logs that are explicitly added via add_log
    }
}

// =====================================================
// DORA INTEGRATION METHODS (for CastScreen)
// =====================================================

impl CastScreen {
    /// Initialize Dora integration (lazy initialization)
    fn init_dora(&mut self, cx: &mut Cx) {
        if self.dora_integration.is_some() {
            return;
        }

        ::log::info!("Initializing Dora integration for mofa-cast");
        let integration = DoraIntegration::new();
        self.dora_integration = Some(integration);

        // Note: recent_files are now loaded by ensure_recent_files_initialized()
        // which is called earlier in handle_event()

        // Start timer to poll for Dora events (100ms interval)
        self.dora_timer = cx.start_interval(0.1);

        // Start timer for file change detection (2 second interval, will be started when file is loaded)
        self.file_check_timer = cx.start_interval(2.0);
        // Stop it initially - will start when a file is imported
        cx.stop_timer(self.file_check_timer);

        // Find default dataflow
        let cwd = std::env::current_dir().ok();
        let dataflow_path = cwd.and_then(|p| {
            // First try: Multi-voice batch TTS (P1.1 feature - 3 voices)
            let multi_voice_path = p.join("apps").join("mofa-cast").join("dataflow").join("multi-voice-batch-tts.yml");
            if multi_voice_path.exists() {
                ::log::info!("Using multi-voice-batch-tts.yml configuration (3 voices: Luo Xiang, Yang Mi, Ma Yun)");
                Some(multi_voice_path)
            } else {
                // Fallback: Single voice config
                let simple_path = p.join("apps").join("mofa-cast").join("dataflow").join("test-primespeech-simple.yml");
                if simple_path.exists() {
                    ::log::info!("Using test-primespeech-simple.yml configuration (single voice: Luo Xiang)");
                    Some(simple_path)
                } else {
                    // Fallback to test-direct config
                    let test_path = p.join("apps").join("mofa-cast").join("dataflow").join("test-direct.yml");
                    if test_path.exists() {
                        ::log::info!("Using test-direct.yml configuration (with text-segmenter)");
                        Some(test_path)
                    } else {
                        // Fallback to batch-tts config
                        let batch_path = p.join("apps").join("mofa-cast").join("dataflow").join("batch-tts.yml");
                        if batch_path.exists() {
                            ::log::info!("Using batch-tts.yml configuration (with text-segmenter)");
                            Some(batch_path)
                        } else {
                            None
                        }
                    }
                }
            }
        });

        self.dora_dataflow_path = dataflow_path;

        ::log::info!("Dora integration initialized, dataflow: {:?}", self.dora_dataflow_path);
    }

    /// Start TTS dataflow
    fn start_tts_dataflow(&mut self, cx: &mut Cx) {
        self.init_dora(cx);

        if let Some(dataflow_path) = &self.dora_dataflow_path {
            if let Some(ref dora) = self.dora_integration {
                ::log::info!("Starting TTS dataflow: {:?}", dataflow_path);
                if dora.start_dataflow(dataflow_path.clone()) {
                    ::log::info!("TTS dataflow started successfully");
                } else {
                    ::log::error!("Failed to start TTS dataflow");
                    // Show error in UI
                    self.view.label(ids!(header.header_description))
                        .set_text(cx, "❌ Failed to start TTS dataflow");
                }
            }
        } else {
            ::log::error!("No dataflow path found");
            self.view.label(ids!(header.header_description))
                .set_text(cx, "❌ Dataflow configuration not found");
        }
    }

    /// Stop TTS dataflow
    fn stop_tts_dataflow(&mut self, cx: &mut Cx) {
        if let Some(ref dora) = self.dora_integration {
            ::log::info!("Stopping TTS dataflow");
            dora.stop_dataflow();
        }
    }

    /// Poll for Dora events
    fn poll_dora_events(&mut self, cx: &mut Cx) {
        if let Some(ref dora) = self.dora_integration {
            let events = dora.poll_events();

            ::log::debug!("Poll dora events: got {} events", events.len());

            for event in events {
                match event {
                    DoraEvent::DataflowStarted { dataflow_id } => {
                        ::log::info!("Dora dataflow started: {}", dataflow_id);
                        self.add_log(cx, &format!("[INFO] ✅ Dataflow started: {}", dataflow_id));
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("✅ Dataflow started: {}", dataflow_id));
                    }
                    DoraEvent::DataflowStopped => {
                        ::log::info!("Dora dataflow stopped");
                        self.add_log(cx, "[INFO] Dataflow stopped");
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, "Dataflow stopped");
                    }
                    DoraEvent::AudioSegment { data } => {
                        ::log::info!("Received audio segment in UI: {} samples, rate: {}, channels: {}",
                            data.samples.len(), data.sample_rate, data.channels);

                        // Create output directory if it doesn't exist
                        let output_dir = std::path::PathBuf::from("./output/mofa-cast/dora");
                        if let Err(e) = std::fs::create_dir_all(&output_dir) {
                            ::log::error!("Failed to create output directory: {}", e);
                            let _ = self.view.label(ids!(header.header_description))
                                .set_text(cx, &format!("❌ Failed to create output directory: {}", e));
                            continue;
                        }

                        // Generate filename for this segment
                        let speaker = data.participant_id.as_deref().unwrap_or("unknown");
                        let filename = format!("segment_{:03}_{}.wav", self.segments_received, speaker);
                        let file_path = output_dir.join(&filename);

                        // Convert f32 samples to i16 for WAV format
                        let samples_i16: Vec<i16> = data.samples.iter()
                            .map(|s| (s.clamp(-1.0, 1.0) * 32767.0) as i16)
                            .collect();

                        // Create WAV file
                        if let Err(e) = self.write_wav_file(&file_path, &samples_i16, data.sample_rate, data.channels) {
                            ::log::error!("Failed to write WAV file: {}", e);
                            let _ = self.view.label(ids!(header.header_description))
                                .set_text(cx, &format!("❌ Failed to write audio: {}", e));
                            continue;
                        }

                        // Calculate duration
                        let duration_secs = data.samples.len() as f64 / data.sample_rate as f64;

                        // Create segment info
                        let segment_info = crate::audio_mixer::AudioSegmentInfo {
                            path: file_path,
                            speaker: speaker.to_string(),
                            duration_secs,
                            sample_rate: data.sample_rate,
                            channels: data.channels,
                        };

                        self.collected_audio_segments.push(segment_info);
                        self.segments_received += 1;

                        ::log::info!("✅ Saved segment {} of {}: {} ({:.2}s)",
                            self.segments_received, self.total_segments_expected, filename, duration_secs);
                        self.add_log(cx, &format!("[INFO] ✅ Saved segment {} of {}: {} ({:.2}s)",
                            self.segments_received, self.total_segments_expected, filename, duration_secs));

                        // Update progress
                        let pct = if self.total_segments_expected > 0 {
                            (self.segments_received as f32 / self.total_segments_expected as f32 * 100.0) as u32
                        } else {
                            0
                        };

                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("🎙️ Received {}/{} segments ({}%) - {}",
                                self.segments_received, self.total_segments_expected, pct, speaker));

                        // Check if all segments received
                        if self.segments_received >= self.total_segments_expected && self.total_segments_expected > 0 {
                            ::log::info!("All {} segments received, enabling export", self.segments_received);
                            self.add_log(cx, &format!("[INFO] ✅ All {} segments received! Ready to export.", self.segments_received));
                            self.view.label(ids!(header.header_description))
                                .set_text(cx, &format!("✅ All {} segments received! Ready to export.", self.segments_received));
                            self.view.button(ids!(main_content.right_panel.control_bar.export_button))
                                .set_enabled(cx, true);
                            self.is_synthesizing = false;
                            self.view.redraw(cx);
                        }
                    }
                    DoraEvent::Progress { current, total, speaker } => {
                        let pct = (current as f32 / total as f32 * 100.0) as u32;
                        ::log::info!("Progress: {}/{} ({}%) - {}", current, total, pct, speaker);
                        self.add_log(cx, &format!("[INFO] 🎙️ Synthesizing... {}/{} ({}%) - {}", current, total, pct, speaker));
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("🎙️ Synthesizing... {}/{} ({}%) - {}", current, total, pct, speaker));
                    }
                    DoraEvent::Error { message } => {
                        ::log::error!("Dora error: {}", message);
                        self.add_log(cx, &format!("[ERROR] ❌ Error: {}", message));
                        self.view.label(ids!(header.header_description))
                            .set_text(cx, &format!("❌ Error: {}", message));
                    }
                    DoraEvent::Log { message } => {
                        ::log::info!("[Dora] {}", message);
                        self.add_log(cx, &format!("[INFO] [Dora] {}", message));
                    }
                }
            }
        }
    }
}

// Export WidgetRefExt trait for timer control
impl CastScreenRef {
    /// Update dark mode for this screen
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            // Update main background
            inner.view.apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Update headers
            inner.view.label(ids!(header.title_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            inner.view.label(ids!(header.description)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Update panels
            inner.view.view(ids!(main_content.left_panel.import_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            inner.view.view(ids!(main_content.left_panel.speakers_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            inner.view.view(ids!(main_content.right_panel.control_bar)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply theme to script panel
            inner.view.view(ids!(main_content.right_panel.editor_container.script_panel)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // NOTE: TextInput apply_over causes "target class not found" errors (from mofa-fm)
            // So we DON'T apply dark_mode to script_editor here
            // The TextInput will handle dark_mode through its internal instance

            // Update all labels
            inner.view.apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
