//! # MofaApp Trait - Plugin App Interface
//!
//! This module defines the standard interface for apps that integrate with the MoFA Studio shell.
//!
//! ## Architecture
//!
//! Due to Makepad's compile-time `live_design!` macro requirements, widget types must
//! still be imported directly in the shell. This trait provides:
//!
//! - **Standardized metadata** - App name, ID, description via [`AppInfo`]
//! - **Consistent registration** - Widget registration via [`MofaApp::live_design`]
//! - **Timer lifecycle** - Resource management via [`TimerControl`]
//! - **Runtime queries** - App discovery via [`AppRegistry`]
//!
//! ## Usage in Shell
//!
//! ```rust,ignore
//! use mofa_widgets::{MofaApp, AppRegistry};
//! use mofa_fm::MoFaFMApp;
//! use mofa_settings::MoFaSettingsApp;
//!
//! // In App struct
//! #[rust]
//! app_registry: AppRegistry,
//!
//! // In LiveHook::after_new_from_doc
//! fn after_new_from_doc(&mut self, _cx: &mut Cx) {
//!     self.app_registry.register(MoFaFMApp::info());
//!     self.app_registry.register(MoFaSettingsApp::info());
//! }
//!
//! // In LiveRegister
//! fn live_register(cx: &mut Cx) {
//!     <MoFaFMApp as MofaApp>::live_design(cx);
//!     <MoFaSettingsApp as MofaApp>::live_design(cx);
//! }
//! ```
//!
//! ## Creating a New App
//!
//! ```rust,ignore
//! use mofa_widgets::{MofaApp, AppInfo};
//!
//! pub struct MyApp;
//!
//! impl MofaApp for MyApp {
//!     fn info() -> AppInfo {
//!         AppInfo {
//!             name: "My App",
//!             id: "my-app",
//!             description: "My awesome MoFA app",
//!         }
//!     }
//!
//!     fn live_design(cx: &mut Cx) {
//!         crate::screen::live_design(cx);
//!         crate::widgets::live_design(cx);
//!     }
//! }
//! ```

use makepad_widgets::Cx;

/// Metadata about a registered app
#[derive(Clone, Debug)]
pub struct AppInfo {
    /// Display name shown in UI
    pub name: &'static str,
    /// Unique identifier for the app
    pub id: &'static str,
    /// Description of the app
    pub description: &'static str,
}

/// Trait for apps that integrate with MoFA Studio shell
///
/// # Example
/// ```ignore
/// impl MofaApp for MoFaFMApp {
///     fn info() -> AppInfo {
///         AppInfo {
///             name: "MoFA FM",
///             id: "mofa-fm",
///             description: "AI-powered audio streaming",
///         }
///     }
///
///     fn live_design(cx: &mut Cx) {
///         screen::live_design(cx);
///     }
/// }
/// ```
pub trait MofaApp {
    /// Returns metadata about this app
    fn info() -> AppInfo where Self: Sized;

    /// Register this app's widgets with Makepad
    fn live_design(cx: &mut Cx);
}

/// Trait for apps with timer-based animations that need lifecycle control
///
/// Apps implementing this trait should stop their timers when hidden
/// and restart them when shown, to prevent resource waste.
pub trait TimerControl {
    /// Stop all timers (call when app becomes hidden)
    fn stop_timers(&self, cx: &mut Cx);

    /// Start/restart timers (call when app becomes visible)
    fn start_timers(&self, cx: &mut Cx);
}

/// Registry of all installed apps
///
/// Note: Due to Makepad's architecture, apps must still be imported at compile time.
/// This registry provides metadata for runtime queries (e.g., sidebar generation).
pub struct AppRegistry {
    apps: Vec<AppInfo>,
}

impl AppRegistry {
    /// Create a new empty registry
    pub const fn new() -> Self {
        Self { apps: Vec::new() }
    }

    /// Register an app in the registry
    pub fn register(&mut self, info: AppInfo) {
        self.apps.push(info);
    }

    /// Get all registered apps
    pub fn apps(&self) -> &[AppInfo] {
        &self.apps
    }

    /// Find an app by ID
    pub fn find_by_id(&self, id: &str) -> Option<&AppInfo> {
        self.apps.iter().find(|app| app.id == id)
    }

    /// Number of registered apps
    pub fn len(&self) -> usize {
        self.apps.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.apps.is_empty()
    }
}

impl Default for AppRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for widgets that respond to global state changes
///
/// Apps implement this trait to receive notifications when global state
/// changes (e.g., dark mode toggle, provider configuration updates).
///
/// # Example
/// ```ignore
/// impl StateChangeListener for MyScreenRef {
///     fn on_dark_mode_change(&self, cx: &mut Cx, dark_mode: f64) {
///         if let Some(mut inner) = self.borrow_mut() {
///             inner.view.apply_over(cx, live!{
///                 draw_bg: { dark_mode: (dark_mode) }
///             });
///         }
///     }
/// }
/// ```
pub trait StateChangeListener {
    /// Called when dark mode setting changes
    ///
    /// # Arguments
    /// * `cx` - Makepad context for applying UI updates
    /// * `dark_mode` - Dark mode value (0.0 = light, 1.0 = dark)
    fn on_dark_mode_change(&self, cx: &mut Cx, dark_mode: f64);
}
