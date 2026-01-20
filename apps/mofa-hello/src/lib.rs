//! MoFA Hello - A simple Hello World app

pub mod screen;

pub use screen::HelloScreen;
pub use screen::HelloScreenWidgetRefExt;

use makepad_widgets::Cx;
use mofa_widgets::{AppInfo, MofaApp};

/// MoFA Hello app descriptor
pub struct MoFaHelloApp;

impl MofaApp for MoFaHelloApp {
    fn info() -> AppInfo {
        AppInfo {
            name: "Hello World",
            id: "mofa-hello",
            description: "A simple Hello World app",
        }
    }

    fn live_design(cx: &mut Cx) {
        screen::live_design(cx);
    }
}

/// Register all MoFA Hello widgets with Makepad
pub fn live_design(cx: &mut Cx) {
    MoFaHelloApp::live_design(cx);
}
