//! Hello World Screen

use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::FONT_BOLD;
    use mofa_widgets::theme::FONT_REGULAR;
    use mofa_widgets::theme::DARK_BG;
    use mofa_widgets::theme::DARK_BG_DARK;
    use mofa_widgets::theme::TEXT_PRIMARY;
    use mofa_widgets::theme::TEXT_PRIMARY_DARK;
    use mofa_widgets::theme::TEXT_SECONDARY;
    use mofa_widgets::theme::TEXT_SECONDARY_DARK;
    use mofa_widgets::theme::PANEL_BG;
    use mofa_widgets::theme::PANEL_BG_DARK;

    pub HelloScreen = {{HelloScreen}} {
        width: Fill, height: Fill
        flow: Down
        align: {x: 0.5, y: 0.5}
        spacing: 16
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }

        // Card container
        card = <RoundedView> {
            width: 400, height: Fit
            padding: 40
            flow: Down
            align: {x: 0.5, y: 0.5}
            spacing: 16
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                border_radius: 12.0
                fn get_color(self) -> vec4 {
                    return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                }
            }

            title = <Label> {
                text: "Hello World!"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_BOLD>{ font_size: 32.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                    }
                }
            }

            subtitle = <Label> {
                text: "Welcome to MoFA Studio"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 14.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct HelloScreen {
    #[deref]
    view: View,
}

impl Widget for HelloScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl HelloScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
            inner.view.view(ids!(card)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(card.title)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(card.subtitle)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.redraw(cx);
        }
    }
}
