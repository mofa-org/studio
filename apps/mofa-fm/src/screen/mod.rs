//! MoFA FM Screen - Main screen for AI-powered audio streaming
//!
//! This module is split into sub-modules for better organization:
//! - `design.rs` - UI layout and styling (live_design! DSL)
//! - `audio_controls.rs` - Audio device selection, mic monitoring
//! - `chat_panel.rs` - Chat display, prompt input
//! - `log_panel.rs` - Log display, filtering
//! - `dora_handlers.rs` - Dora event handling, dataflow control

mod audio_controls;
mod chat_panel;
pub mod design;  // Public for Makepad live_design path resolution
mod dora_handlers;
mod log_panel;
mod role_config;

use role_config::{RoleConfig, get_role_config_path, VOICE_OPTIONS};

use makepad_widgets::*;
use crate::mofa_hero::{MofaHeroWidgetExt, MofaHeroAction};
use crate::log_bridge;
use crate::dora_integration::{DoraIntegration, DoraCommand};
use mofa_widgets::participant_panel::ParticipantPanelWidgetExt;
use mofa_widgets::{StateChangeListener, TimerControl};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Data preloaded in background thread
#[derive(Default)]
struct PreloadedData {
    context_content: Option<String>,
    student1_config: Option<RoleConfig>,
    student2_config: Option<RoleConfig>,
    tutor_config: Option<RoleConfig>,
    loading_complete: bool,
}

/// Register live design for this module
pub fn live_design(cx: &mut Cx) {
    design::live_design(cx);
}

/// Chat message entry for display
#[derive(Clone, Debug)]
pub struct ChatMessageEntry {
    pub sender: String,
    pub content: String,
    pub timestamp: u64,
    pub is_streaming: bool,
    pub session_id: Option<String>,
}

impl ChatMessageEntry {
    pub fn new(sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            sender: sender.into(),
            content: content.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            is_streaming: false,
            session_id: None,
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct MoFaFMScreen {
    #[deref]
    view: View,
    #[rust]
    log_panel_collapsed: bool,
    #[rust]
    log_panel_width: f64,
    #[rust]
    splitter_dragging: bool,
    #[rust]
    audio_manager: Option<crate::audio::AudioManager>,
    #[rust]
    audio_timer: Timer,
    #[rust]
    audio_initialized: bool,
    #[rust]
    input_devices: Vec<String>,
    #[rust]
    output_devices: Vec<String>,
    #[rust]
    input_labels: Vec<String>,  // Labels with "(Default)" suffix for dropdown
    #[rust]
    output_labels: Vec<String>, // Labels with "(Default)" suffix for dropdown
    #[rust]
    input_device_count: usize,  // Number of input devices (for index calculation)
    #[rust]
    selected_input_idx: usize,  // Currently selected input device index (1-based in combined list)
    #[rust]
    selected_output_idx: usize, // Currently selected output device index
    #[rust]
    log_level_filter: usize,  // 0=ALL, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR
    #[rust]
    log_node_filter: usize,   // 0=ALL, 1=ASR, 2=TTS, 3=LLM, 4=Bridge, 5=Monitor, 6=App
    #[rust]
    log_entries: Vec<String>,  // Raw log entries for filtering

    // Dropdown width caching for popup menu sync
    #[rust]
    cached_device_dropdown_width: f64,

    // AEC toggle state
    #[rust]
    aec_enabled: bool,
    // Note: AEC blink animation is now shader-driven (self.time), no timer needed

    // Mic mute state
    #[rust]
    mic_muted: bool,

    // Dora integration
    #[rust]
    dora_integration: Option<DoraIntegration>,
    #[rust]
    dataflow_path: Option<PathBuf>,
    #[rust]
    dora_timer: Timer,
    // NextFrame-based animation for copy buttons (smooth fade instead of timer reset)
    #[rust]
    copy_chat_flash_active: bool,
    #[rust]
    copy_chat_flash_start: f64,  // Absolute start time
    #[rust]
    copy_log_flash_active: bool,
    #[rust]
    copy_log_flash_start: f64,   // Absolute start time
    #[rust]
    chat_messages: Vec<ChatMessageEntry>,
    #[rust]
    last_chat_count: usize,

    // Audio playback
    #[rust]
    audio_player: Option<std::sync::Arc<crate::audio_player::AudioPlayer>>,
    // Participant audio levels for decay animation (matches conference-dashboard)
    #[rust]
    participant_levels: [f64; 3],  // 0=student1, 1=student2, 2=tutor

    // SharedDoraState tracking (for detecting changes)
    #[rust]
    connected_bridges: Vec<String>,
    #[rust]
    processed_dora_log_count: usize,

    // Tab state: 0 = Running, 1 = Settings
    #[rust]
    active_tab: usize,

    // Context content loaded from study-context.md
    #[rust]
    context_content: String,

    // Role configurations
    #[rust]
    student1_config: RoleConfig,
    #[rust]
    student2_config: RoleConfig,
    #[rust]
    tutor_config: RoleConfig,
    // Background preloading - configs loaded into memory at startup
    #[rust]
    configs_preloaded: bool,
    // Async preloaded data from background thread
    #[rust]
    async_preload: Option<Arc<Mutex<PreloadedData>>>,
    // Lazy UI population flags - track which TextInputs have been populated
    #[rust]
    context_ui_populated: bool,
    #[rust]
    student1_ui_populated: bool,
    #[rust]
    student2_ui_populated: bool,
    #[rust]
    tutor_ui_populated: bool,

    // Editor maximize state: None = normal, Some(id) = maximized editor
    #[rust]
    maximized_editor: Option<String>,
    // Shader pre-compilation: hide Settings tab after first draw
    #[rust]
    shader_precompile_frame: usize,
}

impl Widget for MoFaFMScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize audio and log bridge on first event
        if !self.audio_initialized {
            log_bridge::init();
            self.init_audio(cx);
            self.audio_initialized = true;
            // Start async preloading in background thread
            self.start_async_preload();
        }

        // Check if async preload completed - store data and trigger UI population
        if !self.configs_preloaded {
            let mut preload_ready = false;
            if let Some(ref preload) = self.async_preload {
                if let Ok(mut data) = preload.try_lock() {
                    if data.loading_complete {
                        if let Some(content) = data.context_content.take() {
                            self.context_content = content;
                        }
                        if let Some(config) = data.student1_config.take() {
                            self.student1_config = config;
                        }
                        if let Some(config) = data.student2_config.take() {
                            self.student2_config = config;
                        }
                        if let Some(config) = data.tutor_config.take() {
                            self.tutor_config = config;
                        }
                        preload_ready = true;
                    }
                }
            }
            if preload_ready {
                self.configs_preloaded = true;
                // Trigger UI population on next frame
                self.shader_precompile_frame = 1;
                cx.new_next_frame();
                ::log::info!("Async preload complete - triggering UI population");
            }
        }

        // Handle audio timer for mic level updates, log polling, and buffer status
        if self.audio_timer.is_event(event).is_some() {
            self.update_mic_level(cx);
            // Poll Rust logs (50ms interval is fine for log updates)
            self.poll_rust_logs(cx);
            // Send actual buffer fill percentage to dora for backpressure control
            // This replaces the bridge's estimation with the real value from AudioPlayer
            if let Some(ref player) = self.audio_player {
                let fill_percentage = player.buffer_fill_percentage();
                if let Some(ref dora) = self.dora_integration {
                    dora.send_command(DoraCommand::UpdateBufferStatus { fill_percentage });
                }
            }
        }

        // Handle dora timer for polling dora events
        if self.dora_timer.is_event(event).is_some() {
            self.poll_dora_events(cx);
        }

        // Handle NextFrame for smooth copy button fade animation
        if let Event::NextFrame(nf) = event {
            let mut needs_redraw = false;
            let current_time = nf.time;

            // Copy chat button fade animation
            if self.copy_chat_flash_active {
                // Capture start time on first frame
                if self.copy_chat_flash_start == 0.0 {
                    self.copy_chat_flash_start = current_time;
                }
                let elapsed = current_time - self.copy_chat_flash_start;
                // Hold at full brightness for 0.3s, then fade out over 0.5s
                let fade_start = 0.3;
                let fade_duration = 0.5;
                let total_duration = fade_start + fade_duration;

                if elapsed >= total_duration {
                    // Animation complete
                    self.copy_chat_flash_active = false;
                    self.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.copy_chat_btn))
                        .apply_over(cx, live!{ draw_bg: { copied: 0.0 } });
                } else if elapsed >= fade_start {
                    // Fade out phase - smoothstep interpolation
                    let t = (elapsed - fade_start) / fade_duration;
                    // Smoothstep: 3t² - 2t³ for smooth ease-out
                    let smooth_t = t * t * (3.0 - 2.0 * t);
                    let copied = 1.0 - smooth_t;
                    self.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.copy_chat_btn))
                        .apply_over(cx, live!{ draw_bg: { copied: (copied) } });
                }
                needs_redraw = true;
                if self.copy_chat_flash_active {
                    cx.new_next_frame();
                }
            }

            // Copy log button fade animation
            if self.copy_log_flash_active {
                // Capture start time on first frame
                if self.copy_log_flash_start == 0.0 {
                    self.copy_log_flash_start = current_time;
                }
                let elapsed = current_time - self.copy_log_flash_start;
                // Hold at full brightness for 0.3s, then fade out over 0.5s
                let fade_start = 0.3;
                let fade_duration = 0.5;
                let total_duration = fade_start + fade_duration;

                if elapsed >= total_duration {
                    // Animation complete
                    self.copy_log_flash_active = false;
                    self.view.view(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn))
                        .apply_over(cx, live!{ draw_bg: { copied: 0.0 } });
                } else if elapsed >= fade_start {
                    // Fade out phase - smoothstep interpolation
                    let t = (elapsed - fade_start) / fade_duration;
                    // Smoothstep: 3t² - 2t³ for smooth ease-out
                    let smooth_t = t * t * (3.0 - 2.0 * t);
                    let copied = 1.0 - smooth_t;
                    self.view.view(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn))
                        .apply_over(cx, live!{ draw_bg: { copied: (copied) } });
                }
                needs_redraw = true;
                if self.copy_log_flash_active {
                    cx.new_next_frame();
                }
            }

            // Populate TextInputs and force render at startup
            if self.shader_precompile_frame > 0 && self.configs_preloaded {
                match self.shader_precompile_frame {
                    1 => {
                        // Step 1: Populate content and show Settings tab
                        self.lazy_populate_editor(cx, "context");
                        self.lazy_populate_editor(cx, "student1");
                        self.lazy_populate_editor(cx, "student2");
                        self.lazy_populate_editor(cx, "tutor");
                        self.view.view(ids!(left_column.settings_tab_content)).set_visible(cx, true);
                        ::log::info!("Startup: Content loaded, Settings visible for pre-render");
                        self.shader_precompile_frame = 2;
                        cx.new_next_frame();
                        needs_redraw = true;
                    }
                    5 => {
                        // Step 2: After a few frames, hide Settings and show Running
                        self.view.view(ids!(left_column.settings_tab_content)).set_visible(cx, false);
                        self.view.view(ids!(left_column.running_tab_content)).set_visible(cx, true);
                        self.shader_precompile_frame = 0;
                        ::log::info!("Startup: Pre-render complete, Settings hidden");
                        needs_redraw = true;
                    }
                    _ => {
                        // Keep rendering for a few frames
                        self.shader_precompile_frame += 1;
                        cx.new_next_frame();
                    }
                }
            }

            if needs_redraw {
                self.view.redraw(cx);
            }
        }

        // Handle mic mute button click
        let mic_btn = self.view.view(ids!(running_tab_content.audio_container.mic_container.mic_group.mic_mute_btn));
        match event.hits(cx, mic_btn.area()) {
            Hit::FingerUp(_) => {
                self.mic_muted = !self.mic_muted;
                self.view.view(ids!(running_tab_content.audio_container.mic_container.mic_group.mic_mute_btn.mic_icon_on))
                    .set_visible(cx, !self.mic_muted);
                self.view.view(ids!(running_tab_content.audio_container.mic_container.mic_group.mic_mute_btn.mic_icon_off))
                    .set_visible(cx, self.mic_muted);
                self.view.redraw(cx);
            }
            _ => {}
        }

        // Handle AEC toggle button click
        // Note: AEC blink animation is now shader-driven, no timer needed
        let aec_btn = self.view.view(ids!(running_tab_content.audio_container.aec_container.aec_group.aec_toggle_btn));
        match event.hits(cx, aec_btn.area()) {
            Hit::FingerUp(_) => {
                self.aec_enabled = !self.aec_enabled;
                let enabled_val = if self.aec_enabled { 1.0 } else { 0.0 };
                self.view.view(ids!(running_tab_content.audio_container.aec_container.aec_group.aec_toggle_btn))
                    .apply_over(cx, live!{ draw_bg: { enabled: (enabled_val) } });
                self.view.redraw(cx);
            }
            _ => {}
        }

        // Handle tab clicks
        let running_tab = self.view.view(ids!(left_column.tab_bar.running_tab));
        let settings_tab = self.view.view(ids!(left_column.tab_bar.settings_tab));

        // Running tab hover
        match event.hits(cx, running_tab.area()) {
            Hit::FingerHoverIn(_) => {
                if self.active_tab != 0 {
                    self.view.view(ids!(left_column.tab_bar.running_tab))
                        .apply_over(cx, live!{ draw_bg: { hover: 1.0 } });
                    self.view.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) => {
                self.view.view(ids!(left_column.tab_bar.running_tab))
                    .apply_over(cx, live!{ draw_bg: { hover: 0.0 } });
                self.view.redraw(cx);
            }
            Hit::FingerUp(_) => {
                if self.active_tab != 0 {
                    self.switch_tab(cx, 0);
                }
            }
            _ => {}
        }

        // Settings tab hover
        match event.hits(cx, settings_tab.area()) {
            Hit::FingerHoverIn(_) => {
                if self.active_tab != 1 {
                    self.view.view(ids!(left_column.tab_bar.settings_tab))
                        .apply_over(cx, live!{ draw_bg: { hover: 1.0 } });
                    self.view.redraw(cx);
                }
            }
            Hit::FingerHoverOut(_) => {
                self.view.view(ids!(left_column.tab_bar.settings_tab))
                    .apply_over(cx, live!{ draw_bg: { hover: 0.0 } });
                self.view.redraw(cx);
            }
            Hit::FingerUp(_) => {
                if self.active_tab != 1 {
                    self.switch_tab(cx, 1);
                }
            }
            _ => {}
        }

        // Handle splitter drag
        let splitter = self.view.view(ids!(splitter));
        match event.hits(cx, splitter.area()) {
            Hit::FingerDown(_) => {
                self.splitter_dragging = true;
            }
            Hit::FingerMove(fm) => {
                if self.splitter_dragging {
                    self.resize_log_panel(cx, fm.abs.x);
                }
            }
            Hit::FingerUp(_) => {
                self.splitter_dragging = false;
            }
            _ => {}
        }

        // Handle actions
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        // Handle MofaHero start/stop actions
        for action in actions {
            match action.as_widget_action().cast() {
                MofaHeroAction::StartClicked => {
                    ::log::info!("Screen received StartClicked action");
                    self.handle_mofa_start(cx);
                }
                MofaHeroAction::StopClicked => {
                    ::log::info!("Screen received StopClicked action");
                    self.handle_mofa_stop(cx);
                }
                MofaHeroAction::None => {}
            }
        }

        // Handle toggle log panel button
        if self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).clicked(actions) {
            self.toggle_log_panel(cx);
        }

        // Handle unified audio device dropdown selection
        // Layout: [0: mic divider, 1..N: mics, N+1: speaker divider, N+2..M: speakers, M+1: ☰ trigger]
        // Trigger at END so popup extends UPWARD with OnSelected positioning
        if let Some(item) = self.view.drop_down(ids!(running_tab_content.audio_container.device_container.device_selectors.audio_device_dropdown)).selected(actions) {
            let mic_divider_idx = 0;
            let speaker_divider_idx = self.input_device_count + 1;
            let trigger_idx = self.input_device_count + self.output_devices.len() + 2;

            // Skip trigger and divider items
            if item == trigger_idx || item == mic_divider_idx || item == speaker_divider_idx {
                // Non-selectable item - restore trigger display
                self.rebuild_dropdown_labels(cx);
            } else if item >= 1 && item <= self.input_device_count {
                // Input device selected (indices 1 to input_device_count)
                let device_idx = item - 1; // Convert to 0-based index
                if device_idx < self.input_devices.len() {
                    let device_name = self.input_devices[device_idx].clone();
                    self.selected_input_idx = item;
                    self.select_input_device(cx, &device_name);
                    self.rebuild_dropdown_labels(cx);
                    self.update_device_dropdown_label(cx);
                }
            } else if item > speaker_divider_idx && item < trigger_idx {
                // Output device selected (indices between speaker divider and trigger)
                let device_idx = item - speaker_divider_idx - 1; // Convert to 0-based index
                if device_idx < self.output_devices.len() {
                    let device_name = self.output_devices[device_idx].clone();
                    self.selected_output_idx = item;
                    self.select_output_device(&device_name);
                    self.rebuild_dropdown_labels(cx);
                    self.update_device_dropdown_label(cx);
                }
            }
        }

        // Handle log level filter dropdown
        if let Some(selected) = self.view.drop_down(ids!(log_section.log_content_column.log_header.log_filter_row.level_filter)).selected(actions) {
            self.log_level_filter = selected;
            self.update_log_display(cx);
        }

        // Handle log node filter dropdown
        if let Some(selected) = self.view.drop_down(ids!(log_section.log_content_column.log_header.log_filter_row.node_filter)).selected(actions) {
            self.log_node_filter = selected;
            self.update_log_display(cx);
        }

        // Handle copy log button (manual click detection since it's a View)
        let copy_log_btn = self.view.view(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn));
        match event.hits(cx, copy_log_btn.area()) {
            Hit::FingerUp(_) => {
                self.copy_logs_to_clipboard(cx);
                // Trigger copied feedback animation with NextFrame-based smooth fade
                self.view.view(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn))
                    .apply_over(cx, live!{ draw_bg: { copied: 1.0 } });
                self.copy_log_flash_active = true;
                self.copy_log_flash_start = 0.0;  // Sentinel: capture actual time on first NextFrame
                cx.new_next_frame();
                self.view.redraw(cx);
            }
            _ => {}
        }

        // Handle copy chat button (manual click detection since it's a View)
        let copy_chat_btn = self.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.copy_chat_btn));
        match event.hits(cx, copy_chat_btn.area()) {
            Hit::FingerUp(_) => {
                self.copy_chat_to_clipboard(cx);
                // Trigger copied feedback animation with NextFrame-based smooth fade
                self.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.copy_chat_btn))
                    .apply_over(cx, live!{ draw_bg: { copied: 1.0 } });
                self.copy_chat_flash_active = true;
                self.copy_chat_flash_start = 0.0;  // Sentinel: capture actual time on first NextFrame
                cx.new_next_frame();
                self.view.redraw(cx);
            }
            _ => {}
        }

        // Handle log search text change
        if self.view.text_input(ids!(log_section.log_content_column.log_header.log_filter_row.log_search)).changed(actions).is_some() {
            self.update_log_display(cx);
        }

        // Handle Send button click
        if self.view.button(ids!(left_column.prompt_container.prompt_section.prompt_row.button_group.send_prompt_btn)).clicked(actions) {
            self.send_prompt(cx);
        }

        // Handle Reset button click
        if self.view.button(ids!(left_column.prompt_container.prompt_section.prompt_row.button_group.reset_btn)).clicked(actions) {
            self.reset_conversation(cx);
        }

        // Handle Context Save button click
        if self.view.button(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_save_row.context_save_btn)).clicked(actions) {
            self.save_context(cx);
        }

        // Handle role save button clicks
        if self.view.button(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_save_row.student1_save_btn)).clicked(actions) {
            self.save_role_config(cx, "student1");
        }
        if self.view.button(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_save_row.student2_save_btn)).clicked(actions) {
            self.save_role_config(cx, "student2");
        }
        if self.view.button(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_save_row.tutor_save_btn)).clicked(actions) {
            self.save_role_config(cx, "tutor");
        }

        // Handle maximize button clicks
        let context_max_btn = self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_maximize_btn));
        match event.hits(cx, context_max_btn.area()) {
            Hit::FingerUp(_) => { self.toggle_maximize(cx, "context"); }
            _ => {}
        }

        let student1_max_btn = self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_header.student1_maximize_btn));
        match event.hits(cx, student1_max_btn.area()) {
            Hit::FingerUp(_) => { self.toggle_maximize(cx, "student1"); }
            _ => {}
        }

        let student2_max_btn = self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_header.student2_maximize_btn));
        match event.hits(cx, student2_max_btn.area()) {
            Hit::FingerUp(_) => { self.toggle_maximize(cx, "student2"); }
            _ => {}
        }

        let tutor_max_btn = self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_header.tutor_maximize_btn));
        match event.hits(cx, tutor_max_btn.area()) {
            Hit::FingerUp(_) => { self.toggle_maximize(cx, "tutor"); }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update popup menu width to match dropdown width
        // This handles first-frame zero width and caches values for performance
        let device_dropdown = self.view.drop_down(ids!(running_tab_content.audio_container.device_container.device_selectors.audio_device_dropdown));
        let dropdown_width = device_dropdown.area().rect(cx).size.x;

        // Only update if width changed significantly (> 1px) to avoid unnecessary apply_over calls
        if dropdown_width > 0.0 && (dropdown_width - self.cached_device_dropdown_width).abs() > 1.0 {
            self.cached_device_dropdown_width = dropdown_width;
            device_dropdown.apply_over(cx, live! {
                popup_menu: { width: (dropdown_width) }
            });
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

// Tab switching
impl MoFaFMScreen {
    fn switch_tab(&mut self, cx: &mut Cx, tab: usize) {
        self.active_tab = tab;

        // Update tab button styles
        let running_selected = if tab == 0 { 1.0 } else { 0.0 };
        let settings_selected = if tab == 1 { 1.0 } else { 0.0 };

        self.view.view(ids!(left_column.tab_bar.running_tab))
            .apply_over(cx, live!{ draw_bg: { selected: (running_selected), hover: 0.0 } });
        self.view.label(ids!(left_column.tab_bar.running_tab.tab_label))
            .apply_over(cx, live!{ draw_text: { selected: (running_selected) } });

        self.view.view(ids!(left_column.tab_bar.settings_tab))
            .apply_over(cx, live!{ draw_bg: { selected: (settings_selected), hover: 0.0 } });
        self.view.label(ids!(left_column.tab_bar.settings_tab.tab_label))
            .apply_over(cx, live!{ draw_text: { selected: (settings_selected) } });

        // Toggle visibility of tab content
        self.view.view(ids!(left_column.running_tab_content)).set_visible(cx, tab == 0);
        self.view.view(ids!(left_column.settings_tab_content)).set_visible(cx, tab == 1);

        // Content already populated at startup - just show/hide
        self.view.redraw(cx);
    }

    /// Populate a role's model dropdown with models and select the default
    fn populate_role_dropdown(&mut self, cx: &mut Cx, role: &str, models: &[String], selected: &str) {
        let dropdown_id = match role {
            "student1" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row.student1_model_dropdown),
            "student2" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row.student2_model_dropdown),
            "tutor" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row.tutor_model_dropdown),
            _ => return,
        };

        let dropdown = self.view.drop_down(dropdown_id);

        // Find selected index
        let selected_idx = models.iter()
            .position(|m| m == selected)
            .unwrap_or(0);

        dropdown.set_labels(cx, models.to_vec());
        dropdown.set_selected_item(cx, selected_idx);
    }

    /// Populate a role's voice dropdown and select the current voice
    fn populate_voice_dropdown(&mut self, cx: &mut Cx, role: &str, selected_voice: &str) {
        let dropdown_id = match role {
            "student1" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row.student1_voice_dropdown),
            "student2" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row.student2_voice_dropdown),
            "tutor" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row.tutor_voice_dropdown),
            _ => return,
        };

        let dropdown = self.view.drop_down(dropdown_id);

        // Find selected index
        let selected_idx = VOICE_OPTIONS.iter()
            .position(|&v| v == selected_voice)
            .unwrap_or(0);

        dropdown.set_selected_item(cx, selected_idx);
    }

    /// Save a role's configuration
    fn save_role_config(&mut self, cx: &mut Cx, role: &str) {
        let (config, prompt_input_id) = match role {
            "student1" => (
                &mut self.student1_config,
                ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container.student1_prompt_scroll.student1_prompt_wrapper.student1_prompt_input)
            ),
            "student2" => (
                &mut self.student2_config,
                ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container.student2_prompt_scroll.student2_prompt_wrapper.student2_prompt_input)
            ),
            "tutor" => (
                &mut self.tutor_config,
                ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container.tutor_prompt_scroll.tutor_prompt_wrapper.tutor_prompt_input)
            ),
            _ => return,
        };

        // Get current system prompt from text input
        let system_prompt = self.view.text_input(prompt_input_id).text();
        config.system_prompt = system_prompt;

        // Get selected model from dropdown
        let dropdown_id = match role {
            "student1" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row.student1_model_dropdown),
            "student2" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row.student2_model_dropdown),
            "tutor" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row.tutor_model_dropdown),
            _ => return,
        };

        let dropdown = self.view.drop_down(dropdown_id);
        let selected_idx = dropdown.selected_item();
        if selected_idx < config.models.len() {
            config.default_model = config.models[selected_idx].clone();
        }

        // Get selected voice from dropdown
        let voice_dropdown_id = match role {
            "student1" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row.student1_voice_dropdown),
            "student2" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row.student2_voice_dropdown),
            "tutor" => ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row.tutor_voice_dropdown),
            _ => return,
        };

        let voice_dropdown = self.view.drop_down(voice_dropdown_id);
        let voice_idx = voice_dropdown.selected_item();
        if voice_idx < VOICE_OPTIONS.len() {
            config.voice = VOICE_OPTIONS[voice_idx].to_string();
        }

        // Save to file
        match config.save() {
            Ok(_) => ::log::info!("Saved {} config", role),
            Err(e) => ::log::error!("Failed to save {} config: {}", role, e),
        }

        self.view.redraw(cx);
    }

    /// Lazy populate a TextInput only when user first interacts with it
    fn lazy_populate_editor(&mut self, cx: &mut Cx, editor: &str) {
        match editor {
            "context" if !self.context_ui_populated && !self.context_content.is_empty() => {
                let content = self.context_content.clone();
                self.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container.context_input_scroll.context_input_wrapper.context_input))
                    .set_text(cx, &content);
                self.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_status))
                    .set_text(cx, "Loaded");
                self.context_ui_populated = true;
                ::log::info!("Lazy loaded context UI ({} bytes)", content.len());
            }
            "student1" if !self.student1_ui_populated && !self.student1_config.system_prompt.is_empty() => {
                let models = self.student1_config.models.clone();
                let default_model = self.student1_config.default_model.clone();
                let voice = self.student1_config.voice.clone();
                let prompt = self.student1_config.system_prompt.clone();
                self.populate_role_dropdown(cx, "student1", &models, &default_model);
                self.populate_voice_dropdown(cx, "student1", &voice);
                self.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container.student1_prompt_scroll.student1_prompt_wrapper.student1_prompt_input))
                    .set_text(cx, &prompt);
                self.student1_ui_populated = true;
                ::log::info!("Lazy loaded student1 UI");
            }
            "student2" if !self.student2_ui_populated && !self.student2_config.system_prompt.is_empty() => {
                let models = self.student2_config.models.clone();
                let default_model = self.student2_config.default_model.clone();
                let voice = self.student2_config.voice.clone();
                let prompt = self.student2_config.system_prompt.clone();
                self.populate_role_dropdown(cx, "student2", &models, &default_model);
                self.populate_voice_dropdown(cx, "student2", &voice);
                self.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container.student2_prompt_scroll.student2_prompt_wrapper.student2_prompt_input))
                    .set_text(cx, &prompt);
                self.student2_ui_populated = true;
                ::log::info!("Lazy loaded student2 UI");
            }
            "tutor" if !self.tutor_ui_populated && !self.tutor_config.system_prompt.is_empty() => {
                let models = self.tutor_config.models.clone();
                let default_model = self.tutor_config.default_model.clone();
                let voice = self.tutor_config.voice.clone();
                let prompt = self.tutor_config.system_prompt.clone();
                self.populate_role_dropdown(cx, "tutor", &models, &default_model);
                self.populate_voice_dropdown(cx, "tutor", &voice);
                self.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container.tutor_prompt_scroll.tutor_prompt_wrapper.tutor_prompt_input))
                    .set_text(cx, &prompt);
                self.tutor_ui_populated = true;
                ::log::info!("Lazy loaded tutor UI");
            }
            _ => {}
        }
    }

    /// Toggle maximize state for an editor - takes over entire mofa-fm page
    fn toggle_maximize(&mut self, cx: &mut Cx, editor: &str) {
        // Lazy populate TextInput on first interaction
        self.lazy_populate_editor(cx, editor);

        let is_currently_maximized = self.maximized_editor.as_deref() == Some(editor);

        if is_currently_maximized {
            // Restore: show all UI elements, reset heights
            self.maximized_editor = None;

            // Show tab bar
            self.view.view(ids!(left_column.tab_bar)).set_visible(cx, true);

            // Show audio section
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section))
                .set_visible(cx, true);

            // Show all role config sections
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config))
                .set_visible(cx, true);

            // Show context header elements (except maximize btn)
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_title))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_status))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_save_row))
                .set_visible(cx, true);

            // Show role headers/dropdowns for student configs
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_label))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_save_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_label))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_save_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_label))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row))
                .set_visible(cx, true);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_save_row))
                .set_visible(cx, true);

            // Reset all editor container heights to normal and disable horizontal scroll
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container))
                .apply_over(cx, live!{ height: 200 });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container.context_input_scroll))
                .apply_over(cx, live!{ scroll_bars: { show_scroll_x: false } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container))
                .apply_over(cx, live!{ height: 120 });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container.student1_prompt_scroll))
                .apply_over(cx, live!{ scroll_bars: { show_scroll_x: false } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container))
                .apply_over(cx, live!{ height: 120 });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container.student2_prompt_scroll))
                .apply_over(cx, live!{ scroll_bars: { show_scroll_x: false } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container))
                .apply_over(cx, live!{ height: 120 });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container.tutor_prompt_scroll))
                .apply_over(cx, live!{ scroll_bars: { show_scroll_x: false } });

            // Update all maximize buttons to show expand icon (maximized: 0.0)
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_maximize_btn))
                .apply_over(cx, live!{ draw_bg: { maximized: 0.0 } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_header.student1_maximize_btn))
                .apply_over(cx, live!{ draw_bg: { maximized: 0.0 } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_header.student2_maximize_btn))
                .apply_over(cx, live!{ draw_bg: { maximized: 0.0 } });
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_header.tutor_maximize_btn))
                .apply_over(cx, live!{ draw_bg: { maximized: 0.0 } });
        } else {
            // Maximize: hide everything except the editor, take over entire page
            self.maximized_editor = Some(editor.to_string());

            // Hide tab bar
            self.view.view(ids!(left_column.tab_bar)).set_visible(cx, false);

            // Hide audio section
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section))
                .set_visible(cx, false);

            // Hide all role config sections except the one being maximized
            let show_context = editor == "context";
            let show_student1 = editor == "student1";
            let show_student2 = editor == "student2";
            let show_tutor = editor == "tutor";

            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section))
                .set_visible(cx, show_context);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config))
                .set_visible(cx, show_student1);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config))
                .set_visible(cx, show_student2);
            self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config))
                .set_visible(cx, show_tutor);

            // Hide extra elements within the maximized section (labels, dropdowns, save buttons)
            if show_context {
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_title))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_status))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_save_row))
                    .set_visible(cx, false);
            } else if show_student1 {
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_label))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_save_row))
                    .set_visible(cx, false);
            } else if show_student2 {
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_label))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_save_row))
                    .set_visible(cx, false);
            } else if show_tutor {
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_label))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row))
                    .set_visible(cx, false);
                self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_save_row))
                    .set_visible(cx, false);
            }

            // Expand the editor container to fill most of the screen and enable horizontal scroll
            match editor {
                "context" => {
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container))
                        .apply_over(cx, live!{ height: 800 });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container.context_input_scroll))
                        .apply_over(cx, live!{ scroll_bars: { show_scroll_x: true } });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_maximize_btn))
                        .apply_over(cx, live!{ draw_bg: { maximized: 1.0 } });
                }
                "student1" => {
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container))
                        .apply_over(cx, live!{ height: 800 });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container.student1_prompt_scroll))
                        .apply_over(cx, live!{ scroll_bars: { show_scroll_x: true } });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_header.student1_maximize_btn))
                        .apply_over(cx, live!{ draw_bg: { maximized: 1.0 } });
                }
                "student2" => {
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container))
                        .apply_over(cx, live!{ height: 800 });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container.student2_prompt_scroll))
                        .apply_over(cx, live!{ scroll_bars: { show_scroll_x: true } });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_header.student2_maximize_btn))
                        .apply_over(cx, live!{ draw_bg: { maximized: 1.0 } });
                }
                "tutor" => {
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container))
                        .apply_over(cx, live!{ height: 800 });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container.tutor_prompt_scroll))
                        .apply_over(cx, live!{ scroll_bars: { show_scroll_x: true } });
                    self.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_header.tutor_maximize_btn))
                        .apply_over(cx, live!{ draw_bg: { maximized: 1.0 } });
                }
                _ => {}
            }
        }

        self.view.redraw(cx);
    }

    /// Start async preloading in a background thread
    fn start_async_preload(&mut self) {
        // Compute paths upfront (on main thread)
        let context_path = self.get_context_path();
        let student1_path = get_role_config_path(self.dataflow_path.as_ref(), "student1");
        let student2_path = get_role_config_path(self.dataflow_path.as_ref(), "student2");
        let tutor_path = get_role_config_path(self.dataflow_path.as_ref(), "tutor");

        // Create shared state for background thread
        let preload = Arc::new(Mutex::new(PreloadedData::default()));
        self.async_preload = Some(preload.clone());

        // Spawn background thread for file I/O
        std::thread::spawn(move || {
            let mut data = PreloadedData::default();

            // Load context file
            if let Ok(content) = std::fs::read_to_string(&context_path) {
                ::log::info!("Async preloaded study-context.md ({} bytes)", content.len());
                data.context_content = Some(content);
            }

            // Load role configs
            if let Ok(config) = RoleConfig::load(&student1_path) {
                ::log::info!("Async preloaded student1 config");
                data.student1_config = Some(config);
            }
            if let Ok(config) = RoleConfig::load(&student2_path) {
                ::log::info!("Async preloaded student2 config");
                data.student2_config = Some(config);
            }
            if let Ok(config) = RoleConfig::load(&tutor_path) {
                ::log::info!("Async preloaded tutor config");
                data.tutor_config = Some(config);
            }

            data.loading_complete = true;

            // Store results in shared state
            if let Ok(mut shared) = preload.lock() {
                *shared = data;
            }
        });
    }

    /// Preload all configs into memory at startup (no UI updates) - DEPRECATED, use start_async_preload
    #[allow(dead_code)]
    fn preload_configs(&mut self) {
        // Preload context file
        let context_path = self.get_context_path();
        if let Ok(content) = std::fs::read_to_string(&context_path) {
            ::log::info!("Preloaded study-context.md ({} bytes)", content.len());
            self.context_content = content;
        }

        // Preload role configs
        let student1_path = get_role_config_path(self.dataflow_path.as_ref(), "student1");
        if let Ok(config) = RoleConfig::load(&student1_path) {
            ::log::info!("Preloaded student1 config");
            self.student1_config = config;
        }

        let student2_path = get_role_config_path(self.dataflow_path.as_ref(), "student2");
        if let Ok(config) = RoleConfig::load(&student2_path) {
            ::log::info!("Preloaded student2 config");
            self.student2_config = config;
        }

        let tutor_path = get_role_config_path(self.dataflow_path.as_ref(), "tutor");
        if let Ok(config) = RoleConfig::load(&tutor_path) {
            ::log::info!("Preloaded tutor config");
            self.tutor_config = config;
        }
    }

    /// Save context editor content to study-context.md
    fn save_context(&mut self, cx: &mut Cx) {
        let context_path = self.get_context_path();
        let content = self.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container.context_input_scroll.context_input_wrapper.context_input))
            .text();

        match std::fs::write(&context_path, &content) {
            Ok(_) => {
                self.context_content = content.clone();
                // Update status to "Saved"
                self.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_status))
                    .set_text(cx, "Saved");
                ::log::info!("Saved study-context.md ({} bytes)", content.len());
            }
            Err(e) => {
                ::log::error!("Failed to save study-context.md: {}", e);
                // Update status to show error
                self.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_status))
                    .set_text(cx, "Save failed");
            }
        }
        self.view.redraw(cx);
    }

    /// Get the path to study-context.md in the dataflow directory
    fn get_context_path(&self) -> PathBuf {
        // Try to use the dataflow_path if set, otherwise search common locations
        if let Some(ref dataflow_path) = self.dataflow_path {
            // Get directory containing the dataflow yaml
            if let Some(parent) = dataflow_path.parent() {
                return parent.join("study-context.md");
            }
        }

        // Fallback: search common locations
        let cwd = std::env::current_dir().unwrap_or_default();

        // First try: apps/mofa-fm/dataflow/study-context.md (workspace root)
        let app_path = cwd.join("apps").join("mofa-fm").join("dataflow").join("study-context.md");
        if app_path.exists() {
            return app_path;
        }

        // Second try: dataflow/study-context.md (run from app directory)
        let local_path = cwd.join("dataflow").join("study-context.md");
        if local_path.exists() {
            return local_path;
        }

        // Default: assume workspace root structure
        app_path
    }
}

impl MoFaFMScreenRef {
    /// Update dark mode for this screen
    /// Delegates to StateChangeListener::on_dark_mode_change for consistency
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        self.on_dark_mode_change(cx, dark_mode);
    }
}

impl TimerControl for MoFaFMScreenRef {
    /// Stop audio and dora timers - call this before hiding/removing the widget
    /// to prevent timer callbacks on inactive state
    /// Note: AEC blink animation is shader-driven and doesn't need stopping
    fn stop_timers(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow_mut() {
            cx.stop_timer(inner.audio_timer);
            cx.stop_timer(inner.dora_timer);
            ::log::debug!("MoFaFMScreen timers stopped");
        }
    }

    /// Restart audio and dora timers - call this when the widget becomes visible again
    /// Note: AEC blink animation is shader-driven and auto-resumes
    fn start_timers(&self, cx: &mut Cx) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.audio_timer = cx.start_interval(0.05);  // 50ms for mic level
            inner.dora_timer = cx.start_interval(0.1);    // 100ms for dora events
            ::log::debug!("MoFaFMScreen timers started");
        }
    }
}

impl StateChangeListener for MoFaFMScreenRef {
    fn on_dark_mode_change(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            // Apply dark mode to screen background
            inner.view.apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to chat section
            inner.view.view(ids!(left_column.running_tab_content.chat_container.chat_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to chat header and title
            inner.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.chat_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to copy chat button
            inner.view.view(ids!(left_column.running_tab_content.chat_container.chat_section.chat_header.copy_chat_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to chat content Markdown
            let chat_markdown = inner.view.markdown(ids!(left_column.running_tab_content.chat_container.chat_section.chat_scroll.chat_content_wrapper.chat_content));
            if dark_mode > 0.5 {
                let light_color = vec4(0.945, 0.961, 0.976, 1.0); // TEXT_PRIMARY_DARK (#f1f5f9)
                chat_markdown.apply_over(cx, live!{
                    font_color: (light_color)
                    draw_normal: { color: (light_color) }
                    draw_bold: { color: (light_color) }
                    draw_italic: { color: (light_color) }
                    draw_fixed: { color: (vec4(0.580, 0.639, 0.722, 1.0)) } // SLATE_400 for code
                });
            } else {
                let dark_color = vec4(0.122, 0.161, 0.216, 1.0); // TEXT_PRIMARY (#1f2937)
                chat_markdown.apply_over(cx, live!{
                    font_color: (dark_color)
                    draw_normal: { color: (dark_color) }
                    draw_bold: { color: (dark_color) }
                    draw_italic: { color: (dark_color) }
                    draw_fixed: { color: (vec4(0.420, 0.451, 0.502, 1.0)) } // GRAY_500 for code
                });
            }

            // Apply dark mode to tab bar
            inner.view.view(ids!(left_column.tab_bar)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.tab_bar.running_tab)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.tab_bar.running_tab.tab_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.tab_bar.settings_tab)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.tab_bar.settings_tab.tab_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to audio control containers
            inner.view.view(ids!(left_column.running_tab_content.audio_container.mic_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Apply dark mode to mic icon
            inner.view.view(ids!(left_column.running_tab_content.audio_container.mic_container.mic_group.mic_mute_btn.mic_icon_on.icon)).apply_over(cx, live!{
                draw_icon: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.running_tab_content.audio_container.aec_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.running_tab_content.audio_container.buffer_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.running_tab_content.audio_container.device_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to device dropdown (the "|" separator)
            inner.view.drop_down(ids!(left_column.running_tab_content.audio_container.device_container.device_selectors.audio_device_dropdown)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // NOTE: DropDown apply_over causes "target class not found" errors
            // TODO: Find alternative way to theme dropdowns

            // Apply dark mode to MofaHero
            inner.view.mofa_hero(ids!(left_column.mofa_hero)).update_dark_mode(cx, dark_mode);

            // Apply dark mode to participant panels
            inner.view.participant_panel(ids!(left_column.running_tab_content.participant_container.participant_bar.student1_panel)).update_dark_mode(cx, dark_mode);
            inner.view.participant_panel(ids!(left_column.running_tab_content.participant_container.participant_bar.student2_panel)).update_dark_mode(cx, dark_mode);
            inner.view.participant_panel(ids!(left_column.running_tab_content.participant_container.participant_bar.tutor_panel)).update_dark_mode(cx, dark_mode);

            // Apply dark mode to prompt section
            inner.view.view(ids!(left_column.running_tab_content.prompt_container.prompt_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // NOTE: TextInput apply_over causes "target class not found" errors
            inner.view.button(ids!(left_column.running_tab_content.prompt_container.prompt_section.prompt_row.button_group.reset_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to settings tab content
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Settings header labels
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_header.settings_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_header.settings_subtitle)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            // Dataflow section labels
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.dataflow_section.dataflow_section_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.dataflow_section.dataflow_path_row.dataflow_path_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.dataflow_section.dataflow_path_row.dataflow_path_value)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            // Role section - title
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.role_section_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            // Role section - student1 config
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_header.student1_name)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row.student1_model_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_model_row.student1_model_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row.student1_voice_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_voice_row.student1_voice_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_prompt_container.student1_prompt_scroll.student1_prompt_wrapper.student1_prompt_input)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
                draw_cursor: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student1_config.student1_header.student1_maximize_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Role section - student2 config
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_header.student2_name)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row.student2_model_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_model_row.student2_model_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row.student2_voice_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_voice_row.student2_voice_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_prompt_container.student2_prompt_scroll.student2_prompt_wrapper.student2_prompt_input)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
                draw_cursor: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.student2_config.student2_header.student2_maximize_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Role section - tutor config
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_header.tutor_name)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row.tutor_model_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_model_row.tutor_model_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row.tutor_voice_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.drop_down(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_voice_row.tutor_voice_dropdown)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_prompt_container.tutor_prompt_scroll.tutor_prompt_wrapper.tutor_prompt_input)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
                draw_cursor: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.tutor_config.tutor_header.tutor_maximize_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Role section - shared context
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.text_input(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_input_container.context_input_scroll.context_input_wrapper.context_input)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
                draw_cursor: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.role_section.context_section.context_header.context_maximize_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // Audio section labels
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section.audio_section_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section.sample_rate_row.sample_rate_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section.sample_rate_row.sample_rate_value)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section.buffer_size_row.buffer_size_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.settings_tab_content.settings_panel.settings_scroll.settings_content.audio_section.buffer_size_row.buffer_size_value)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to splitter
            inner.view.view(ids!(splitter)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to log section - toggle column
            inner.view.view(ids!(log_section.toggle_column)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.button(ids!(log_section.toggle_column.toggle_log_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to log section - log content column
            inner.view.view(ids!(log_section.log_content_column)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(log_section.log_content_column.log_header)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(log_section.log_content_column.log_header.log_title_row.log_title_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to copy log button
            inner.view.view(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to log content Markdown
            // Update dark_mode instance variable on each draw component (they have get_color shader functions)
            let log_markdown = inner.view.markdown(ids!(log_section.log_content_column.log_scroll.log_content_wrapper.log_content));
            log_markdown.apply_over(cx, live!{
                draw_normal: { dark_mode: (dark_mode) }
                draw_bold: { dark_mode: (dark_mode) }
                draw_fixed: { dark_mode: (dark_mode) }
            });

            inner.view.redraw(cx);
        }
    }
}
