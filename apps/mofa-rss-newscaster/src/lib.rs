//! MoFA RSS Newscaster - RSS feed to multi-anchor news broadcast script generator

pub mod screen;
pub mod dora_integration;

use makepad_widgets::Cx;
use mofa_widgets::{AppInfo, MofaApp};

/// RSS Newscaster app descriptor
pub struct MoFaRSSNewscasterApp;

impl MofaApp for MoFaRSSNewscasterApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "RSS Newscaster",
            id: "mofa-rss-newscaster",
            description: "Convert RSS feeds into multi-anchor news broadcast scripts",
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

/// Register all RSS Newscaster widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaRSSNewscasterApp::live_design(cx);
}
