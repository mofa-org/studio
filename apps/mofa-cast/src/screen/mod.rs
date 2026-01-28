//! MoFA Cast Screen Module
//!
//! This module contains the main UI for script to multi-voice podcast transformation.

// Re-export the main screen type
pub use self::main::CastScreen;

mod main;
pub mod design;

/// Register live design for this module
pub fn live_design(cx: &mut makepad_widgets::Cx) {
    design::live_design(cx);
}
