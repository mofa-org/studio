//! MoFA Studio App - Main application shell
//!
//! This file contains the main App struct and all UI definitions.
//! Organized into sections:
//! - UI Definitions (live_design! macro)
//! - Widget Structs (Dashboard, App)
//! - Event Handling (AppMain impl)
//! - Helper Methods (organized by responsibility)

use makepad_widgets::*;
use mofa_studio_shell::widgets::sidebar::SidebarWidgetRefExt;

// App plugin system imports
use mofa_debate::{MoFaDebateApp, MoFaDebateScreenWidgetRefExt};
use mofa_fm::{MoFaFMApp, MoFaFMScreenWidgetRefExt};
use mofa_hello::{MoFaHelloApp, HelloScreenWidgetRefExt};
use mofa_rss_newscaster::MoFaRSSNewscasterApp;
use mofa_settings::data::Preferences;
use mofa_settings::screen::SettingsScreenWidgetRefExt;
use mofa_settings::MoFaSettingsApp;
use mofa_widgets::{AppRegistry, MofaApp, TimerControl};

// ============================================================================
// TAB IDENTIFIER
// ============================================================================

/// Type-safe tab identifiers (replaces magic strings)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabId {
    Profile,
    Settings,
}

// ============================================================================
// UI DEFINITIONS
// ============================================================================

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    // Import from shared theme
    use mofa_widgets::theme::DARK_BG;
    use mofa_widgets::theme::DARK_BG_DARK;
    use mofa_widgets::theme::DIVIDER;
    use mofa_widgets::theme::BORDER;
    use mofa_widgets::theme::SLATE_50;
    use mofa_widgets::theme::SLATE_200;
    use mofa_widgets::theme::SLATE_500;
    use mofa_widgets::theme::SLATE_700;
    use mofa_widgets::theme::SLATE_800;
    use mofa_widgets::theme::GRAY_700;
    use mofa_widgets::theme::DIVIDER_DARK;
    use mofa_widgets::theme::BORDER_DARK;

    // Import extracted widgets
    use mofa_studio_shell::widgets::sidebar::Sidebar;
    use mofa_studio_shell::widgets::dashboard::Dashboard;

    // Import app screens
    use mofa_rss_newscaster::screen::RSSNewscasterScreen;

    // ------------------------------------------------------------------------
    // App Window
    // ------------------------------------------------------------------------

    App = {{App}} {
        ui: <Window> {
            window: { title: "MoFA Studio", inner_size: vec2(1400, 900) }
            pass: { clear_color: (DARK_BG) }
            flow: Overlay

            body = <View> {
                width: Fill, height: Fill
                // Dashboard fills the body
                dashboard_wrapper = <Dashboard> {}
            }

            // Pinned sidebar - positioned below header, pushes content_area only
            pinned_sidebar = <View> {
                width: 0, height: Fill
                abs_pos: vec2(0.0, 72.0)
                visible: false
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 0.0);
                        let bg = mix((SLATE_50), (SLATE_800), self.dark_mode);
                        sdf.fill(bg);
                        // Right border
                        sdf.rect(self.rect_size.x - 1.0, 0., 1.0, self.rect_size.y);
                        let border = mix((DIVIDER), (DIVIDER_DARK), self.dark_mode);
                        sdf.fill(border);
                        return sdf.result;
                    }
                }

                pinned_sidebar_content = <Sidebar> {
                    expand_to_fill: true  // Fill available space when "Show More" is clicked
                }
            }

            sidebar_trigger_overlay = <View> {
                width: 34, height: 34
                abs_pos: vec2(15.0, 13.0)
                cursor: Hand
            }

            sidebar_menu_overlay = <View> {
                width: 250, height: Fit
                abs_pos: vec2(0.0, 52.0)
                visible: false
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        let bg = mix((SLATE_50), (SLATE_800), self.dark_mode);
                        sdf.fill(bg);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        let border = mix((DIVIDER), (DIVIDER_DARK), self.dark_mode);
                        sdf.stroke(border, 1.0);
                        return sdf.result;
                    }
                }

                sidebar_content = <Sidebar> {
                    height: Fit  // Override to Fit for hover overlay
                }
            }

            user_btn_overlay = <View> {
                width: 60, height: 44
                abs_pos: vec2(1320.0, 10.0)
                cursor: Hand
            }

            user_menu = <View> {
                width: 140, height: Fit
                abs_pos: vec2(1250.0, 55.0)
                visible: false
                padding: 6
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        let bg = mix((SLATE_50), (SLATE_800), self.dark_mode);
                        sdf.fill(bg);
                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                        let border = mix((DIVIDER), (DIVIDER_DARK), self.dark_mode);
                        sdf.stroke(border, 1.0);
                        return sdf.result;
                    }
                }
                flow: Down
                spacing: 2

                menu_profile_btn = <Button> {
                    width: Fill, height: Fit
                    padding: {top: 10, bottom: 10, left: 10, right: 10}
                    align: {x: 0.0, y: 0.5}
                    text: "Profile"
                    icon_walk: {width: 14, height: 14, margin: {right: 8}}
                    draw_icon: {
                        svg_file: dep("crate://self/resources/icons/user.svg")
                        fn get_color(self) -> vec4 { return (SLATE_500); }
                    }
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 11.0 }
                        fn get_color(self) -> vec4 {
                            return mix((GRAY_700), (SLATE_200), self.dark_mode);
                        }
                    }
                    draw_bg: {
                        instance hover: 0.0
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            let light_normal = (SLATE_50);
                            let light_hover = (SLATE_200);
                            let dark_normal = (SLATE_800);
                            let dark_hover = (SLATE_700);
                            let normal = mix(light_normal, dark_normal, self.dark_mode);
                            let hover_color = mix(light_hover, dark_hover, self.dark_mode);
                            let color = mix(normal, hover_color, self.hover);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                            sdf.fill(color);
                            return sdf.result;
                        }
                    }
                }

                menu_divider = <View> {
                    width: Fill, height: 1
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix((BORDER), (BORDER_DARK), self.dark_mode);
                        }
                    }
                }

                menu_settings_btn = <Button> {
                    width: Fill, height: Fit
                    padding: {top: 10, bottom: 10, left: 10, right: 10}
                    align: {x: 0.0, y: 0.5}
                    text: "Settings"
                    icon_walk: {width: 14, height: 14, margin: {right: 8}}
                    draw_icon: {
                        svg_file: dep("crate://self/resources/icons/settings.svg")
                        fn get_color(self) -> vec4 { return (SLATE_500); }
                    }
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: { font_size: 11.0 }
                        fn get_color(self) -> vec4 {
                            return mix((GRAY_700), (SLATE_200), self.dark_mode);
                        }
                    }
                    draw_bg: {
                        instance hover: 0.0
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            let light_normal = (SLATE_50);
                            let light_hover = (SLATE_200);
                            let dark_normal = (SLATE_800);
                            let dark_hover = (SLATE_700);
                            let normal = mix(light_normal, dark_normal, self.dark_mode);
                            let hover_color = mix(light_hover, dark_hover, self.dark_mode);
                            let color = mix(normal, hover_color, self.hover);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                            sdf.fill(color);
                            return sdf.result;
                        }
                    }
                }
            }
        }
    }
}

// ============================================================================
// WIDGET STRUCTS
// ============================================================================

// Dashboard widget is now in mofa_studio_shell::widgets::dashboard

#[derive(Live)]
pub struct App {
    #[live]
    ui: WidgetRef,
    #[rust]
    user_menu_open: bool,
    #[rust]
    sidebar_menu_open: bool,
    #[rust]
    open_tabs: Vec<TabId>,
    #[rust]
    active_tab: Option<TabId>,
    #[rust]
    last_window_size: DVec2,
    #[rust]
    sidebar_animating: bool,
    #[rust]
    sidebar_animation_start: f64,
    #[rust]
    sidebar_slide_in: bool,
    /// Sidebar pinned state (click to toggle squeeze effect)
    #[rust]
    sidebar_pinned: bool,
    #[rust]
    sidebar_pin_animating: bool,
    #[rust]
    sidebar_pin_anim_start: f64,
    #[rust]
    sidebar_pin_expanding: bool,
    /// Registry of installed apps (populated on init)
    #[rust]
    app_registry: AppRegistry,
    /// Dark mode state
    #[rust]
    dark_mode: bool,
    /// Dark mode animation progress (0.0 = light, 1.0 = dark)
    #[rust]
    dark_mode_anim: f64,
    /// Whether dark mode animation is in progress
    #[rust]
    dark_mode_animating: bool,
    /// Animation start time
    #[rust]
    dark_mode_anim_start: f64,
    /// Whether initial theme has been applied (on first draw)
    #[rust]
    theme_initialized: bool,
}

impl LiveHook for App {
    fn after_new_from_doc(&mut self, _cx: &mut Cx) {
        // Initialize the app registry with all installed apps
        self.app_registry.register(MoFaFMApp::info());
        self.app_registry.register(MoFaSettingsApp::info());
        self.app_registry.register(MoFaDebateApp::info());
        self.app_registry.register(MoFaHelloApp::info());
        self.app_registry.register(MoFaRSSNewscasterApp::info());

        // Load user preferences and restore dark mode
        let prefs = Preferences::load();
        self.dark_mode = prefs.dark_mode;
        self.dark_mode_anim = if prefs.dark_mode { 1.0 } else { 0.0 };
    }
}

// ============================================================================
// APP REGISTRY METHODS
// ============================================================================

impl App {
    /// Get the number of installed apps
    #[allow(dead_code)]
    pub fn app_count(&self) -> usize {
        self.app_registry.len()
    }

    /// Get app info by ID
    #[allow(dead_code)]
    pub fn get_app_info(&self, id: &str) -> Option<&mofa_widgets::AppInfo> {
        self.app_registry.find_by_id(id)
    }

    /// Get all registered apps
    #[allow(dead_code)]
    pub fn apps(&self) -> &[mofa_widgets::AppInfo] {
        self.app_registry.apps()
    }
}

// ============================================================================
// WIDGET REGISTRATION
// ============================================================================

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        // Core widget libraries
        makepad_widgets::live_design(cx);
        mofa_widgets::live_design(cx);

        // Register apps via MofaApp trait BEFORE dashboard (dashboard uses app widgets)
        // Note: Widget types in live_design! macro still require compile-time imports
        // (Makepad constraint), but registration uses the standardized trait interface
        <MoFaFMApp as MofaApp>::live_design(cx);
        <MoFaSettingsApp as MofaApp>::live_design(cx);
        <MoFaDebateApp as MofaApp>::live_design(cx);
        <MoFaHelloApp as MofaApp>::live_design(cx);
        <MoFaRSSNewscasterApp as MofaApp>::live_design(cx);

        // Shell widgets (order matters - tabs before dashboard, apps before dashboard)
        mofa_studio_shell::widgets::sidebar::live_design(cx);
        mofa_studio_shell::widgets::tabs::live_design(cx);
        mofa_studio_shell::widgets::dashboard::live_design(cx);
    }
}

// ============================================================================
// EVENT HANDLING
// ============================================================================

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.ui.handle_event(cx, event, &mut Scope::empty());

        // Initialize theme on first draw (widgets are ready)
        if !self.theme_initialized {
            if let Event::Draw(_) = event {
                self.theme_initialized = true;
                // Apply initial dark mode from preferences (full update)
                self.apply_dark_mode_panels(cx);
                self.apply_dark_mode_screens(cx);
                // Update header theme toggle icon
                self.update_theme_toggle_icon(cx);
            }
        }

        // Window resize handling
        self.handle_window_resize(cx, event);

        // Sidebar overlay animation (hover effect)
        if self.sidebar_animating {
            self.update_sidebar_animation(cx);
        }

        // Pinned sidebar animation (squeeze effect)
        if self.sidebar_pin_animating {
            self.update_sidebar_pin_animation(cx);
        }

        // Dark mode animation
        if self.dark_mode_animating {
            self.update_dark_mode_animation(cx);
        }

        // Extract actions
        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        // Handle hover events
        self.handle_user_menu_hover(cx, event);
        self.handle_sidebar_hover(cx, event);
        self.handle_theme_toggle(cx, event);

        // Handle click events
        self.handle_sidebar_clicks(cx, &actions);
        self.handle_user_menu_clicks(cx, &actions);
        self.handle_mofa_hero_buttons(cx, event);
        self.handle_tab_clicks(cx, &actions);
        self.handle_tab_close_clicks(cx, event);
    }
}

// ============================================================================
// WINDOW & LAYOUT METHODS
// ============================================================================

impl App {
    /// Handle window resize events
    fn handle_window_resize(&mut self, cx: &mut Cx, event: &Event) {
        if let Event::WindowGeomChange(wg) = event {
            let new_size = wg.new_geom.inner_size;
            if new_size != self.last_window_size {
                self.last_window_size = new_size;
                self.update_overlay_positions(cx);
            }
        }

        if let Event::Draw(_) = event {
            let window_rect = self.ui.area().rect(cx);
            if window_rect.size.x > 0.0 && window_rect.size != self.last_window_size {
                self.last_window_size = window_rect.size;
                self.update_overlay_positions(cx);
            }
        }
    }

    /// Update overlay positions based on window size
    fn update_overlay_positions(&mut self, cx: &mut Cx) {
        let window_width = self.last_window_size.x;
        let window_height = self.last_window_size.y;

        if window_width <= 0.0 {
            return;
        }

        let user_btn_x = window_width - 80.0;
        self.ui.view(ids!(user_btn_overlay)).apply_over(
            cx,
            live! {
                abs_pos: (dvec2(user_btn_x, 10.0))
            },
        );

        let user_menu_x = window_width - 150.0;
        self.ui.view(ids!(user_menu)).apply_over(
            cx,
            live! {
                abs_pos: (dvec2(user_menu_x, 55.0))
            },
        );

        let max_scroll_height = (window_height - 230.0).max(200.0);
        self.ui
            .sidebar(ids!(sidebar_menu_overlay.sidebar_content))
            .set_max_scroll_height(max_scroll_height);

        // Pinned sidebar: starts at header bottom (~72px), so less available height
        // Reserved space: header(72) + sidebar padding(30) + logo(5) + mofa_fm(44) + spacing(12)
        //                + divider(17) + settings(44) + more spacing(8) = ~232px
        let pinned_max_scroll = (window_height - 232.0).max(200.0);
        self.ui
            .sidebar(ids!(pinned_sidebar.pinned_sidebar_content))
            .set_max_scroll_height(pinned_max_scroll);

        self.ui.redraw(cx);
    }
}

// ============================================================================
// USER MENU METHODS
// ============================================================================

impl App {
    /// Handle user menu hover
    fn handle_user_menu_hover(&mut self, cx: &mut Cx, event: &Event) {
        let user_btn = self.ui.view(ids!(user_btn_overlay));
        let user_menu = self.ui.view(ids!(user_menu));

        match event.hits(cx, user_btn.area()) {
            Hit::FingerHoverIn(_) => {
                if !self.user_menu_open {
                    self.user_menu_open = true;
                    user_menu.set_visible(cx, true);
                    self.ui.redraw(cx);
                }
            }
            _ => {}
        }

        if self.user_menu_open {
            if let Event::MouseMove(mm) = event {
                let btn_rect = user_btn.area().rect(cx);
                let menu_rect = user_menu.area().rect(cx);

                let in_btn = mm.abs.x >= btn_rect.pos.x - 5.0
                    && mm.abs.x <= btn_rect.pos.x + btn_rect.size.x + 5.0
                    && mm.abs.y >= btn_rect.pos.y - 5.0
                    && mm.abs.y <= btn_rect.pos.y + btn_rect.size.y + 10.0;

                let in_menu = mm.abs.x >= menu_rect.pos.x - 5.0
                    && mm.abs.x <= menu_rect.pos.x + menu_rect.size.x + 5.0
                    && mm.abs.y >= menu_rect.pos.y - 5.0
                    && mm.abs.y <= menu_rect.pos.y + menu_rect.size.y + 5.0;

                if !in_btn && !in_menu {
                    self.user_menu_open = false;
                    user_menu.set_visible(cx, false);
                    self.ui.redraw(cx);
                }
            }
        }
    }

    /// Handle user menu button clicks
    fn handle_user_menu_clicks(&mut self, cx: &mut Cx, actions: &[Action]) {
        if self
            .ui
            .button(ids!(user_menu.menu_profile_btn))
            .clicked(actions)
        {
            self.user_menu_open = false;
            self.ui.view(ids!(user_menu)).set_visible(cx, false);
            self.open_or_switch_tab(cx, TabId::Profile);
        }

        if self
            .ui
            .button(ids!(user_menu.menu_settings_btn))
            .clicked(actions)
        {
            self.user_menu_open = false;
            self.ui.view(ids!(user_menu)).set_visible(cx, false);
            self.open_or_switch_tab(cx, TabId::Settings);
        }
    }

    /// Handle header theme toggle button
    fn handle_theme_toggle(&mut self, cx: &mut Cx, event: &Event) {
        let theme_btn = self.ui.view(ids!(
            body.dashboard_wrapper.dashboard_base.header.theme_toggle
        ));

        match event.hits(cx, theme_btn.area()) {
            Hit::FingerHoverIn(_) => {
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper.dashboard_base.header.theme_toggle
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { hover: 1.0 }
                        },
                    );
                self.ui.redraw(cx);
            }
            Hit::FingerHoverOut(_) => {
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper.dashboard_base.header.theme_toggle
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { hover: 0.0 }
                        },
                    );
                self.ui.redraw(cx);
            }
            Hit::FingerUp(_) => {
                self.toggle_dark_mode(cx);
                self.update_theme_toggle_icon(cx);

                // Save preference to disk
                let mut prefs = Preferences::load();
                prefs.dark_mode = self.dark_mode;
                if let Err(e) = prefs.save() {
                    eprintln!("Failed to save dark mode preference: {}", e);
                }
            }
            _ => {}
        }
    }

    /// Update the theme toggle icon based on current mode
    fn update_theme_toggle_icon(&mut self, cx: &mut Cx) {
        let is_dark = self.dark_mode;
        self.ui
            .view(ids!(
                body.dashboard_wrapper
                    .dashboard_base
                    .header
                    .theme_toggle
                    .sun_icon
            ))
            .set_visible(cx, !is_dark);
        self.ui
            .view(ids!(
                body.dashboard_wrapper
                    .dashboard_base
                    .header
                    .theme_toggle
                    .moon_icon
            ))
            .set_visible(cx, is_dark);
        self.ui.redraw(cx);
    }
}

// ============================================================================
// SIDEBAR METHODS
// ============================================================================

impl App {
    /// Handle sidebar hover and click
    fn handle_sidebar_hover(&mut self, cx: &mut Cx, event: &Event) {
        let sidebar_trigger = self.ui.view(ids!(sidebar_trigger_overlay));
        let sidebar_menu = self.ui.view(ids!(sidebar_menu_overlay));

        match event.hits(cx, sidebar_trigger.area()) {
            Hit::FingerHoverIn(_) => {
                // Hover: show overlay sidebar (only if not pinned)
                if !self.sidebar_pinned && !self.sidebar_menu_open && !self.sidebar_animating {
                    self.sidebar_menu_open = true;
                    self.start_sidebar_slide_in(cx);
                }
            }
            Hit::FingerUp(_) => {
                // Click: toggle pinned sidebar (squeeze effect)
                if !self.sidebar_pin_animating {
                    self.toggle_sidebar_pinned(cx);
                }
            }
            _ => {}
        }

        if self.sidebar_menu_open && !self.sidebar_animating {
            if let Event::MouseMove(mm) = event {
                let trigger_rect = sidebar_trigger.area().rect(cx);
                let sidebar_rect = sidebar_menu.area().rect(cx);

                let in_trigger = mm.abs.x >= trigger_rect.pos.x - 5.0
                    && mm.abs.x <= trigger_rect.pos.x + trigger_rect.size.x + 5.0
                    && mm.abs.y >= trigger_rect.pos.y - 5.0
                    && mm.abs.y <= trigger_rect.pos.y + trigger_rect.size.y + 5.0;

                let in_sidebar = mm.abs.x >= sidebar_rect.pos.x - 5.0
                    && mm.abs.x <= sidebar_rect.pos.x + sidebar_rect.size.x + 10.0
                    && mm.abs.y >= sidebar_rect.pos.y - 5.0
                    && mm.abs.y <= sidebar_rect.pos.y + sidebar_rect.size.y + 5.0;

                if !in_trigger && !in_sidebar {
                    self.sidebar_menu_open = false;
                    self.start_sidebar_slide_out(cx);
                }
            }
        }
    }

    /// Handle sidebar menu item clicks (both overlay and pinned sidebars)
    fn handle_sidebar_clicks(&mut self, cx: &mut Cx, actions: &[Action]) {
        // MoFA FM tab (overlay or pinned)
        let fm_clicked = self
            .ui
            .button(ids!(
                sidebar_menu_overlay
                    .sidebar_content
                    .main_content
                    .mofa_fm_tab
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .mofa_fm_tab
                ))
                .clicked(actions);

        if fm_clicked {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            // Stop debate timers when leaving debate page
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .stop_timers(cx);
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .rss_newscaster_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .start_timers(cx);
            self.ui.redraw(cx);
        }

        // Debate tab (overlay or pinned)
        let debate_clicked = self
            .ui
            .button(ids!(
                sidebar_menu_overlay.sidebar_content.main_content.debate_tab
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .debate_tab
                ))
                .clicked(actions);

        if debate_clicked {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            // Stop FM timers when leaving FM page
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .rss_newscaster_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: false });
            // Start debate timers
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .start_timers(cx);
            self.ui.redraw(cx);
        }

        // Hello tab (overlay or pinned)
        let hello_clicked = self
            .ui
            .button(ids!(
                sidebar_menu_overlay.sidebar_content.main_content.hello_tab
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .hello_tab
                ))
                .clicked(actions);

        if hello_clicked {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            // Stop timers when leaving other pages
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .stop_timers(cx);
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui.redraw(cx);
        }

        // RSS Newscaster tab (overlay or pinned)
        let rss_newscaster_clicked = self
            .ui
            .button(ids!(
                sidebar_menu_overlay.sidebar_content.main_content.rss_newscaster_tab
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .rss_newscaster_tab
                ))
                .clicked(actions);

        if rss_newscaster_clicked {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            // Stop timers when leaving other pages
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .stop_timers(cx);
            // Hide all other pages
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: false });
            // Show RSS Newscaster page
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .rss_newscaster_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui.redraw(cx);
        }

        // Settings tab (overlay or pinned)
        let settings_clicked = self
            .ui
            .button(ids!(sidebar_menu_overlay.sidebar_content.settings_tab))
            .clicked(actions)
            || self
                .ui
                .button(ids!(pinned_sidebar.pinned_sidebar_content.settings_tab))
                .clicked(actions);

        if settings_clicked {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .stop_timers(cx);
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .rss_newscaster_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui.redraw(cx);
        }

        // App buttons (1-20) - check if any was clicked (overlay or pinned)
        let app_clicked_overlay = self
            .ui
            .button(ids!(
                sidebar_menu_overlay
                    .sidebar_content
                    .main_content
                    .apps_wrapper
                    .apps_scroll
                    .app1_btn
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app2_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app3_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app4_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app5_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app6_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app7_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app8_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app9_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app10_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app11_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app12_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app13_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app14_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app15_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app16_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app17_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app18_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app19_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    sidebar_menu_overlay
                        .sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app20_btn
                ))
                .clicked(actions);

        let app_clicked_pinned = self
            .ui
            .button(ids!(
                pinned_sidebar
                    .pinned_sidebar_content
                    .main_content
                    .apps_wrapper
                    .apps_scroll
                    .app1_btn
            ))
            .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app2_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app3_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app4_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app5_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app6_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app7_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app8_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app9_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app10_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app11_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app12_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app13_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app14_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app15_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app16_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app17_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app18_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app19_btn
                ))
                .clicked(actions)
            || self
                .ui
                .button(ids!(
                    pinned_sidebar
                        .pinned_sidebar_content
                        .main_content
                        .apps_wrapper
                        .apps_scroll
                        .app20_btn
                ))
                .clicked(actions);

        if app_clicked_overlay || app_clicked_pinned {
            // Close overlay if open
            if self.sidebar_menu_open {
                self.sidebar_menu_open = false;
                self.start_sidebar_slide_out(cx);
            }
            self.open_tabs.clear();
            self.active_tab = None;
            self.ui.view(ids!(body.tab_overlay)).set_visible(cx, false);
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
            self.ui
                .mo_fa_debate_screen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .stop_timers(cx);
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .debate_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .app_page
                ))
                .apply_over(cx, live! { visible: true });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .hello_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui
                .view(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .settings_page
                ))
                .apply_over(cx, live! { visible: false });
            self.ui.redraw(cx);
        }
    }
}

// ============================================================================
// ANIMATION METHODS
// ============================================================================

impl App {
    /// Update sidebar slide animation (hover overlay effect)
    fn update_sidebar_animation(&mut self, cx: &mut Cx) {
        const ANIMATION_DURATION: f64 = 0.2;
        const SIDEBAR_WIDTH: f64 = 250.0;

        let elapsed = Cx::time_now() - self.sidebar_animation_start;
        let progress = (elapsed / ANIMATION_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);

        let x = if self.sidebar_slide_in {
            -SIDEBAR_WIDTH * (1.0 - eased)
        } else {
            -SIDEBAR_WIDTH * eased
        };

        self.ui.view(ids!(sidebar_menu_overlay)).apply_over(
            cx,
            live! {
                abs_pos: (dvec2(x, 52.0))
            },
        );

        if progress >= 1.0 {
            self.sidebar_animating = false;
            if !self.sidebar_slide_in {
                self.ui
                    .view(ids!(sidebar_menu_overlay))
                    .set_visible(cx, false);
                self.ui
                    .sidebar(ids!(sidebar_menu_overlay.sidebar_content))
                    .collapse_show_more(cx);
            }
        }

        self.ui.redraw(cx);
    }

    /// Start sidebar slide-in animation
    fn start_sidebar_slide_in(&mut self, cx: &mut Cx) {
        self.sidebar_animating = true;
        self.sidebar_animation_start = Cx::time_now();
        self.sidebar_slide_in = true;
        self.ui.view(ids!(sidebar_menu_overlay)).apply_over(
            cx,
            live! {
                abs_pos: (dvec2(-250.0, 52.0))
            },
        );
        self.ui
            .view(ids!(sidebar_menu_overlay))
            .set_visible(cx, true);
        self.ui
            .sidebar(ids!(sidebar_menu_overlay.sidebar_content))
            .restore_selection_state(cx);
        self.ui.redraw(cx);
    }

    /// Start sidebar slide-out animation
    fn start_sidebar_slide_out(&mut self, cx: &mut Cx) {
        self.sidebar_animating = true;
        self.sidebar_animation_start = Cx::time_now();
        self.sidebar_slide_in = false;
        self.ui.redraw(cx);
    }

    /// Toggle pinned sidebar (squeeze effect)
    fn toggle_sidebar_pinned(&mut self, cx: &mut Cx) {
        // Close hover overlay if open
        if self.sidebar_menu_open {
            self.sidebar_menu_open = false;
            self.ui
                .view(ids!(sidebar_menu_overlay))
                .set_visible(cx, false);
        }

        self.sidebar_pinned = !self.sidebar_pinned;
        self.sidebar_pin_animating = true;
        self.sidebar_pin_anim_start = Cx::time_now();
        self.sidebar_pin_expanding = self.sidebar_pinned;

        // Show/hide pinned sidebar
        self.ui.view(ids!(pinned_sidebar)).set_visible(cx, true);
        self.ui.redraw(cx);
    }

    /// Update pinned sidebar animation (squeeze effect)
    fn update_sidebar_pin_animation(&mut self, cx: &mut Cx) {
        const ANIMATION_DURATION: f64 = 0.25;
        const SIDEBAR_WIDTH: f64 = 250.0;

        let elapsed = Cx::time_now() - self.sidebar_pin_anim_start;
        let progress = (elapsed / ANIMATION_DURATION).min(1.0);
        let eased = 1.0 - (1.0 - progress).powi(3); // Cubic ease-out

        // Calculate sidebar width based on animation
        let sidebar_width = if self.sidebar_pin_expanding {
            SIDEBAR_WIDTH * eased
        } else {
            SIDEBAR_WIDTH * (1.0 - eased)
        };

        // Get header's actual bottom position
        let header_rect = self
            .ui
            .view(ids!(body.dashboard_wrapper.dashboard_base.header))
            .area()
            .rect(cx);
        let header_bottom = header_rect.pos.y + header_rect.size.y;

        // Apply width and position to pinned sidebar
        self.ui.view(ids!(pinned_sidebar)).apply_over(
            cx,
            live! {
                width: (sidebar_width)
                abs_pos: (dvec2(0.0, header_bottom))
            },
        );

        // Apply left margin to content_area to push it (not the header)
        self.ui
            .view(ids!(body.dashboard_wrapper.dashboard_base.content_area))
            .apply_over(
                cx,
                live! {
                    margin: { left: (sidebar_width) }
                },
            );

        // Trigger overlay stays at original position - hamburger is in header which doesn't move

        if progress >= 1.0 {
            self.sidebar_pin_animating = false;
            if !self.sidebar_pin_expanding {
                self.ui.view(ids!(pinned_sidebar)).set_visible(cx, false);
            }
        }

        self.ui.redraw(cx);
    }

    /// Toggle dark mode with animation
    pub fn toggle_dark_mode(&mut self, cx: &mut Cx) {
        self.dark_mode = !self.dark_mode;
        self.dark_mode_animating = true;
        self.dark_mode_anim_start = Cx::time_now();

        // Apply screens immediately at target value (snap, not animated)
        // This avoids calling update_dark_mode on every frame
        let target = if self.dark_mode { 1.0 } else { 0.0 };
        self.apply_dark_mode_screens_with_value(cx, target);

        self.ui.redraw(cx);
    }

    /// Update dark mode animation
    fn update_dark_mode_animation(&mut self, cx: &mut Cx) {
        let elapsed = Cx::time_now() - self.dark_mode_anim_start;
        let duration = 0.3; // 300ms animation

        // Ease-out cubic
        let t = (elapsed / duration).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);

        // Animate from current to target
        let target = if self.dark_mode { 1.0 } else { 0.0 };
        let start = if self.dark_mode { 0.0 } else { 1.0 };
        self.dark_mode_anim = start + (target - start) * eased;

        // During animation: only update main panels (no errors)
        // Full update with screens happens only at the end
        self.apply_dark_mode_panels(cx);

        if t >= 1.0 {
            self.dark_mode_animating = false;
            self.dark_mode_anim = target;
            // Apply to ALL widgets including screens at animation end
            self.apply_dark_mode_screens(cx);
        }

        self.ui.redraw(cx);
    }

    /// Apply dark mode to main panels only (safe for animation frames, no errors)
    fn apply_dark_mode_panels(&mut self, cx: &mut Cx) {
        let dm = self.dark_mode_anim;

        // Apply to dashboard wrapper background
        self.ui.view(ids!(body.dashboard_wrapper)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
            },
        );

        // Apply to header
        self.ui
            .view(ids!(body.dashboard_wrapper.dashboard_base.header))
            .apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dm) }
                },
            );

        // Apply to pinned sidebar background
        self.ui.view(ids!(pinned_sidebar)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
            },
        );

        // Apply to pinned sidebar content
        self.ui
            .sidebar(ids!(pinned_sidebar.pinned_sidebar_content))
            .update_dark_mode(cx, dm);

        // Apply to sidebar menu overlay
        self.ui.view(ids!(sidebar_menu_overlay)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
            },
        );

        // Apply to sidebar content (this is safe, sidebar widget handles it internally)
        self.ui
            .sidebar(ids!(sidebar_menu_overlay.sidebar_content))
            .update_dark_mode(cx, dm);

        // Apply to user menu
        self.ui.view(ids!(user_menu)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
            },
        );

        // Apply to user menu buttons
        self.ui.button(ids!(user_menu.menu_profile_btn)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
                draw_text: { dark_mode: (dm) }
            },
        );
        self.ui
            .button(ids!(user_menu.menu_settings_btn))
            .apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dm) }
                    draw_text: { dark_mode: (dm) }
                },
            );
        self.ui.view(ids!(user_menu.menu_divider)).apply_over(
            cx,
            live! {
                draw_bg: { dark_mode: (dm) }
            },
        );

        // Apply to tab overlay - only when tabs are open
        if !self.open_tabs.is_empty() {
            self.ui.view(ids!(body.tab_overlay)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dm) }
                },
            );

            // Apply to tab bar
            self.ui.view(ids!(body.tab_overlay.tab_bar)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dm) }
                },
            );

            // Apply to tab widgets
            if self.open_tabs.contains(&TabId::Profile) {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.profile_tab))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .label(ids!(body.tab_overlay.tab_bar.profile_tab.tab_label))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.profile_tab.close_btn))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
            }

            if self.open_tabs.contains(&TabId::Settings) {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.settings_tab))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .label(ids!(body.tab_overlay.tab_bar.settings_tab.tab_label))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.settings_tab.close_btn))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
            }

            // Tab content backgrounds
            if self.open_tabs.contains(&TabId::Profile) {
                // Profile page background
                self.ui
                    .view(ids!(body.tab_overlay.tab_content.profile_page))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
                // Profile page internal widgets
                self.ui
                    .label(ids!(
                        body.tab_overlay.tab_content.profile_page.profile_title
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .view(ids!(
                        body.tab_overlay.tab_content.profile_page.profile_divider
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .view(ids!(
                        body.tab_overlay
                            .tab_content
                            .profile_page
                            .profile_row
                            .profile_avatar
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_bg: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .label(ids!(
                        body.tab_overlay
                            .tab_content
                            .profile_page
                            .profile_row
                            .profile_info
                            .profile_name
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .label(ids!(
                        body.tab_overlay
                            .tab_content
                            .profile_page
                            .profile_row
                            .profile_info
                            .profile_email
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
                self.ui
                    .label(ids!(
                        body.tab_overlay
                            .tab_content
                            .profile_page
                            .profile_coming_soon
                    ))
                    .apply_over(
                        cx,
                        live! {
                            draw_text: { dark_mode: (dm) }
                        },
                    );
            }
        }
    }

    /// Apply dark mode to screens (may produce errors, called once at start/end only)
    fn apply_dark_mode_screens(&mut self, cx: &mut Cx) {
        self.apply_dark_mode_screens_with_value(cx, self.dark_mode_anim);
    }

    /// Apply dark mode to screens with a specific value
    fn apply_dark_mode_screens_with_value(&mut self, cx: &mut Cx, dm: f64) {
        // Apply to MoFA FM screen
        self.ui
            .mo_fa_fmscreen(ids!(
                body.dashboard_wrapper
                    .dashboard_base
                    .content_area
                    .main_content
                    .content
                    .fm_page
            ))
            .update_dark_mode(cx, dm);

        // Apply to Hello screen
        self.ui
            .hello_screen(ids!(
                body.dashboard_wrapper
                    .dashboard_base
                    .content_area
                    .main_content
                    .content
                    .hello_page
            ))
            .update_dark_mode(cx, dm);

        // Apply to Settings screen in main content
        self.ui
            .settings_screen(ids!(
                body.dashboard_wrapper
                    .dashboard_base
                    .content_area
                    .main_content
                    .content
                    .settings_page
            ))
            .update_dark_mode(cx, dm);

        // Apply to tab overlay content - only when tabs are open
        if !self.open_tabs.is_empty() {
            if self.open_tabs.contains(&TabId::Settings) {
                self.ui
                    .settings_screen(ids!(body.tab_overlay.tab_content.settings_tab_page))
                    .update_dark_mode(cx, dm);
            }
        }
    }
}

// ============================================================================
// TAB MANAGEMENT METHODS
// ============================================================================

impl App {
    /// Open a tab or switch to it if already open
    fn open_or_switch_tab(&mut self, cx: &mut Cx, tab_id: TabId) {
        if !self.open_tabs.contains(&tab_id) {
            self.open_tabs.push(tab_id);
        }

        self.active_tab = Some(tab_id);
        self.update_tab_ui(cx);
    }

    /// Close a tab
    fn close_tab(&mut self, cx: &mut Cx, tab_id: TabId) {
        self.open_tabs.retain(|t| *t != tab_id);

        if self.active_tab == Some(tab_id) {
            self.active_tab = self.open_tabs.last().copied();
        }

        self.update_tab_ui(cx);
    }

    /// Handle tab widget clicks
    fn handle_tab_clicks(&mut self, cx: &mut Cx, actions: &[Action]) {
        if self
            .ui
            .view(ids!(body.tab_overlay.tab_bar.profile_tab))
            .finger_up(actions)
            .is_some()
        {
            if self.open_tabs.contains(&TabId::Profile) {
                self.active_tab = Some(TabId::Profile);
                self.update_tab_ui(cx);
            }
        }

        if self
            .ui
            .view(ids!(body.tab_overlay.tab_bar.settings_tab))
            .finger_up(actions)
            .is_some()
        {
            if self.open_tabs.contains(&TabId::Settings) {
                self.active_tab = Some(TabId::Settings);
                self.update_tab_ui(cx);
            }
        }
    }

    /// Handle tab close button clicks
    fn handle_tab_close_clicks(&mut self, cx: &mut Cx, event: &Event) {
        let profile_close = self
            .ui
            .view(ids!(body.tab_overlay.tab_bar.profile_tab.close_btn));
        match event.hits(cx, profile_close.area()) {
            Hit::FingerUp(_) => {
                self.close_tab(cx, TabId::Profile);
                return;
            }
            Hit::FingerHoverIn(_) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.profile_tab.close_btn))
                    .apply_over(cx, live! { draw_bg: { hover: 1.0 } });
                self.ui.redraw(cx);
            }
            Hit::FingerHoverOut(_) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.profile_tab.close_btn))
                    .apply_over(cx, live! { draw_bg: { hover: 0.0 } });
                self.ui.redraw(cx);
            }
            _ => {}
        }

        let settings_close = self
            .ui
            .view(ids!(body.tab_overlay.tab_bar.settings_tab.close_btn));
        match event.hits(cx, settings_close.area()) {
            Hit::FingerUp(_) => {
                self.close_tab(cx, TabId::Settings);
                return;
            }
            Hit::FingerHoverIn(_) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.settings_tab.close_btn))
                    .apply_over(cx, live! { draw_bg: { hover: 1.0 } });
                self.ui.redraw(cx);
            }
            Hit::FingerHoverOut(_) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_bar.settings_tab.close_btn))
                    .apply_over(cx, live! { draw_bg: { hover: 0.0 } });
                self.ui.redraw(cx);
            }
            _ => {}
        }
    }

    /// Update tab bar and content visibility
    fn update_tab_ui(&mut self, cx: &mut Cx) {
        let profile_open = self.open_tabs.contains(&TabId::Profile);
        let settings_open = self.open_tabs.contains(&TabId::Settings);
        let any_tabs_open = !self.open_tabs.is_empty();

        let profile_active = self.active_tab == Some(TabId::Profile);
        let settings_active = self.active_tab == Some(TabId::Settings);

        let was_overlay_visible = self.ui.view(ids!(body.tab_overlay)).visible();

        self.ui
            .view(ids!(body.tab_overlay))
            .set_visible(cx, any_tabs_open);

        // Manage FM page timers
        if any_tabs_open && !was_overlay_visible {
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .stop_timers(cx);
        } else if !any_tabs_open && was_overlay_visible {
            self.ui
                .mo_fa_fmscreen(ids!(
                    body.dashboard_wrapper
                        .dashboard_base
                        .content_area
                        .main_content
                        .content
                        .fm_page
                ))
                .start_timers(cx);
        }

        // Update tab visibility
        self.ui
            .view(ids!(body.tab_overlay.tab_bar.profile_tab))
            .set_visible(cx, profile_open);
        self.ui
            .view(ids!(body.tab_overlay.tab_bar.settings_tab))
            .set_visible(cx, settings_open);

        // Update profile tab active state
        let profile_active_val = if profile_active { 1.0 } else { 0.0 };
        self.ui
            .view(ids!(body.tab_overlay.tab_bar.profile_tab))
            .apply_over(cx, live! { draw_bg: { active: (profile_active_val) } });
        self.ui
            .label(ids!(body.tab_overlay.tab_bar.profile_tab.tab_label))
            .apply_over(cx, live! { draw_text: { active: (profile_active_val) } });

        // Update settings tab active state
        let settings_active_val = if settings_active { 1.0 } else { 0.0 };
        self.ui
            .view(ids!(body.tab_overlay.tab_bar.settings_tab))
            .apply_over(cx, live! { draw_bg: { active: (settings_active_val) } });
        self.ui
            .label(ids!(body.tab_overlay.tab_bar.settings_tab.tab_label))
            .apply_over(cx, live! { draw_text: { active: (settings_active_val) } });

        // Hide all content pages first
        self.ui
            .view(ids!(body.tab_overlay.tab_content.profile_page))
            .set_visible(cx, false);
        self.ui
            .view(ids!(body.tab_overlay.tab_content.settings_tab_page))
            .set_visible(cx, false);

        // Show active tab content
        match self.active_tab {
            Some(TabId::Profile) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_content.profile_page))
                    .set_visible(cx, true);
            }
            Some(TabId::Settings) => {
                self.ui
                    .view(ids!(body.tab_overlay.tab_content.settings_tab_page))
                    .set_visible(cx, true);
            }
            None => {
                if profile_open {
                    self.ui
                        .view(ids!(body.tab_overlay.tab_content.profile_page))
                        .set_visible(cx, true);
                } else if settings_open {
                    self.ui
                        .view(ids!(body.tab_overlay.tab_content.settings_tab_page))
                        .set_visible(cx, true);
                }
            }
        }

        self.ui.redraw(cx);
    }
}

// ============================================================================
// MOFA HERO METHODS
// ============================================================================

impl App {
    /// Handle MofaHero start/stop button clicks
    fn handle_mofa_hero_buttons(&mut self, cx: &mut Cx, event: &Event) {
        let start_view = self.ui.view(ids!(
            body.dashboard_wrapper
                .dashboard_base
                .content_area
                .main_content
                .content
                .fm_page
                .mofa_hero
                .action_section
                .start_view
        ));
        match event.hits(cx, start_view.area()) {
            Hit::FingerUp(_) => {
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper
                            .dashboard_base
                            .content_area
                            .main_content
                            .content
                            .fm_page
                            .mofa_hero
                            .action_section
                            .start_view
                    ))
                    .set_visible(cx, false);
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper
                            .dashboard_base
                            .content_area
                            .main_content
                            .content
                            .fm_page
                            .mofa_hero
                            .action_section
                            .stop_view
                    ))
                    .set_visible(cx, true);
                self.ui.redraw(cx);
            }
            _ => {}
        }
        let stop_view = self.ui.view(ids!(
            body.dashboard_wrapper
                .dashboard_base
                .content_area
                .main_content
                .content
                .fm_page
                .mofa_hero
                .action_section
                .stop_view
        ));
        match event.hits(cx, stop_view.area()) {
            Hit::FingerUp(_) => {
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper
                            .dashboard_base
                            .content_area
                            .main_content
                            .content
                            .fm_page
                            .mofa_hero
                            .action_section
                            .start_view
                    ))
                    .set_visible(cx, true);
                self.ui
                    .view(ids!(
                        body.dashboard_wrapper
                            .dashboard_base
                            .content_area
                            .main_content
                            .content
                            .fm_page
                            .mofa_hero
                            .action_section
                            .stop_view
                    ))
                    .set_visible(cx, false);
                self.ui.redraw(cx);
            }
            _ => {}
        }
    }
}

// ============================================================================
// APP ENTRY POINT
// ============================================================================

app_main!(App);
