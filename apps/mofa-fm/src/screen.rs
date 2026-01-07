//! MoFA FM Screen - Main screen for AI-powered audio streaming

use makepad_widgets::*;
use crate::mofa_hero::{MofaHeroWidgetExt, MofaHeroAction, ConnectionStatus};
use crate::log_bridge;
use crate::dora_integration::{DoraIntegration, DoraCommand, DoraEvent};
use mofa_widgets::participant_panel::ParticipantPanelWidgetExt;
use mofa_widgets::StateChangeListener;
use mofa_settings::data::Preferences;
use std::collections::HashMap;
use std::path::PathBuf;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;
    use mofa_widgets::participant_panel::ParticipantPanel;
    use mofa_widgets::log_panel::LogPanel;
    use crate::mofa_hero::MofaHero;

    // Local layout constants (colors imported from theme)
    SECTION_SPACING = 12.0
    PANEL_RADIUS = 4.0
    PANEL_PADDING = 12.0

    // Reusable panel header style with dark mode support
    PanelHeader = <View> {
        width: Fill, height: Fit
        padding: {left: 16, right: 16, top: 12, bottom: 12}
        align: {y: 0.5}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((SLATE_50), (SLATE_800), self.dark_mode);
            }
        }
    }

    // Reusable vertical divider
    VerticalDivider = <View> {
        width: 1, height: Fill
        margin: {top: 4, bottom: 4}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DIVIDER), (DIVIDER_DARK), self.dark_mode);
            }
        }
    }

    // MoFA FM Screen - adaptive horizontal layout with left content and right log panel
    pub MoFaFMScreen = {{MoFaFMScreen}} {
        width: Fill, height: Fill
        flow: Right
        spacing: 0
        padding: { left: 16, right: 16, top: 16, bottom: 16 }
        align: {y: 0.0}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }

        // Left column - main content area (adaptive width)
        left_column = <View> {
            width: Fill, height: Fill
            flow: Down
            spacing: (SECTION_SPACING)
            align: {y: 0.0}

            // System status bar (self-contained widget)
            mofa_hero = <MofaHero> {
                width: Fill
            }

            // Participant status cards container
            participant_container = <View> {
                width: Fill, height: Fit
                flow: Down
                spacing: 8

                participant_bar = <View> {
                    width: Fill, height: Fit
                    flow: Right
                    spacing: (SECTION_SPACING)

                    student1_panel = <ParticipantPanel> {
                        width: Fill, height: Fit
                        header = { name_label = { text: "Student 1" } }
                    }
                    student2_panel = <ParticipantPanel> {
                        width: Fill, height: Fit
                        header = { name_label = { text: "Student 2" } }
                    }
                    tutor_panel = <ParticipantPanel> {
                        width: Fill, height: Fit
                        header = { name_label = { text: "Tutor" } }
                    }
                }
            }

            // Chat window container (fills remaining space)
            chat_container = <View> {
                width: Fill, height: Fill
                flow: Down

                chat_section = <RoundedView> {
                    width: Fill, height: Fill
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        border_size: 1.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                            sdf.fill(bg);
                            sdf.stroke(border, self.border_size);
                            return sdf.result;
                        }
                    }
                    flow: Down

                    // Chat header with copy button
                    chat_header = <PanelHeader> {
                        chat_title = <Label> {
                            text: "Chat History"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                        <Filler> {}
                        // Copy to clipboard button
                        copy_chat_btn = <Button> {
                            width: 28, height: 24
                            text: ""
                            draw_bg: {
                                instance hover: 0.0
                                instance pressed: 0.0
                                instance copied: 0.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    let c = self.rect_size * 0.5;

                                    // Background - flash green when copied
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                    let bg_color = mix((BORDER), (GRAY_300), self.hover);
                                    let bg_color = mix(bg_color, (TEXT_MUTED), self.pressed);
                                    let bg_color = mix(bg_color, #22c55e, self.copied);
                                    sdf.fill(bg_color);

                                    // Icon color - white when copied for contrast
                                    let icon_color = mix((GRAY_600), #ffffff, self.copied);

                                    // Always draw clipboard, color changes to indicate success
                                    // Clipboard icon - back rectangle
                                    sdf.box(c.x - 4.0, c.y - 2.0, 8.0, 9.0, 1.0);
                                    sdf.stroke(icon_color, 1.2);

                                    // Clipboard icon - front rectangle (overlapping)
                                    sdf.box(c.x - 2.0, c.y - 5.0, 8.0, 9.0, 1.0);
                                    sdf.fill(bg_color);
                                    sdf.box(c.x - 2.0, c.y - 5.0, 8.0, 9.0, 1.0);
                                    sdf.stroke(icon_color, 1.2);

                                    return sdf.result;
                                }
                            }
                            animator: {
                                hover = {
                                    default: off
                                    off = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                                    on = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                                }
                                pressed = {
                                    default: off
                                    off = { from: {all: Forward {duration: 0.05}} apply: {draw_bg: {pressed: 0.0}} }
                                    on = { from: {all: Forward {duration: 0.02}} apply: {draw_bg: {pressed: 1.0}} }
                                }
                            }
                        }
                    }

                    // Chat messages area (scrollable, fills space)
                    chat_scroll = <ScrollYView> {
                        width: Fill, height: Fill
                        flow: Down
                        scroll_bars: <ScrollBars> {
                            show_scroll_x: false
                            show_scroll_y: true
                        }

                        chat_content_wrapper = <View> {
                            width: Fill, height: Fit
                            padding: (PANEL_PADDING)
                            flow: Down

                            chat_content = <Markdown> {
                                width: Fill, height: Fit
                                font_size: 13.0
                                font_color: (TEXT_PRIMARY)
                                paragraph_spacing: 8

                                draw_normal: {
                                    text_style: <FONT_REGULAR>{ font_size: 13.0 }
                                }
                                draw_bold: {
                                    text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                                }
                            }
                        }
                    }
                }
            }

            // Audio control panel container - horizontal layout with individual containers
            audio_container = <View> {
                width: Fill, height: Fit
                flow: Right
                spacing: (SECTION_SPACING)

                // Mic level meter container
                mic_container = <RoundedView> {
                    width: Fit, height: Fit
                    padding: (PANEL_PADDING)
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        border_size: 1.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                            sdf.fill(bg);
                            sdf.stroke(border, self.border_size);
                            return sdf.result;
                        }
                    }

                    mic_group = <View> {
                        width: Fit, height: Fit
                        flow: Right
                        spacing: 10
                        align: {y: 0.5}

                        mic_mute_btn = <View> {
                            width: Fit, height: Fit
                            flow: Overlay
                            cursor: Hand
                            padding: 4

                            mic_icon_on = <View> {
                                width: Fit, height: Fit
                                <Icon> {
                                    draw_icon: {
                                        svg_file: dep("crate://self/resources/icons/mic.svg")
                                        fn get_color(self) -> vec4 { return (SLATE_500); }
                                    }
                                    icon_walk: {width: 20, height: 20}
                                }
                            }
                        }

                        mic_level_meter = <View> {
                            width: Fit, height: Fit
                            flow: Right
                            spacing: 3
                            align: {y: 0.5}
                            padding: {top: 2, bottom: 2}

                            mic_led_1 = <RoundedView> { width: 8, height: 14, draw_bg: { color: (GREEN_500), border_radius: 2.0 } }
                            mic_led_2 = <RoundedView> { width: 8, height: 14, draw_bg: { color: (GREEN_500), border_radius: 2.0 } }
                            mic_led_3 = <RoundedView> { width: 8, height: 14, draw_bg: { color: (SLATE_200), border_radius: 2.0 } }
                            mic_led_4 = <RoundedView> { width: 8, height: 14, draw_bg: { color: (SLATE_200), border_radius: 2.0 } }
                            mic_led_5 = <RoundedView> { width: 8, height: 14, draw_bg: { color: (SLATE_200), border_radius: 2.0 } }
                        }
                    }
                }

                // AEC toggle container
                aec_container = <RoundedView> {
                    width: Fit, height: Fit
                    padding: (PANEL_PADDING)
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        border_size: 1.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                            sdf.fill(bg);
                            sdf.stroke(border, self.border_size);
                            return sdf.result;
                        }
                    }

                    aec_group = <View> {
                        width: Fit, height: Fit
                        flow: Right
                        spacing: 8
                        align: {y: 0.5}

                        aec_toggle_btn = <View> {
                            width: Fit, height: Fit
                            padding: 6
                            flow: Overlay
                            cursor: Hand
                            show_bg: true
                            draw_bg: {
                                instance enabled: 1.0  // 1.0=on, 0.0=off
                                // Blink animation now driven by shader time - no timer needed!
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                    let green = vec4(0.133, 0.773, 0.373, 1.0);
                                    let bright = vec4(0.2, 0.9, 0.5, 1.0);
                                    let gray = vec4(0.667, 0.686, 0.725, 1.0);
                                    // When enabled, pulse between green and bright green using shader time
                                    // sin(time * speed) creates smooth oscillation, step makes it blink
                                    let blink = step(0.0, sin(self.time * 2.0)) * self.enabled;
                                    let base = mix(gray, green, self.enabled);
                                    let col = mix(base, bright, blink * 0.5);
                                    sdf.fill(col);
                                    return sdf.result;
                                }
                            }
                            align: {x: 0.5, y: 0.5}

                            <Icon> {
                                draw_icon: {
                                    svg_file: dep("crate://self/resources/icons/aec.svg")
                                    fn get_color(self) -> vec4 { return (WHITE); }
                                }
                                icon_walk: {width: 20, height: 20}
                            }
                        }
                    }
                }

                // Device selectors container - fills remaining space
                device_container = <RoundedView> {
                    width: Fill, height: Fit
                    padding: (PANEL_PADDING)
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        border_size: 1.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                            sdf.fill(bg);
                            sdf.stroke(border, self.border_size);
                            return sdf.result;
                        }
                    }

                    device_selectors = <View> {
                        width: Fill, height: Fit
                        flow: Right
                        spacing: 16
                        align: {y: 0.5}

                        // Input device group (fills available space)
                        input_device_group = <View> {
                            width: Fill, height: Fit
                            flow: Right
                            spacing: 8
                            align: {y: 0.5}

                            input_device_label = <Label> {
                                width: 70  // Fixed width for alignment with output label
                                text: "Mic:"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                            }

                            input_device_dropdown = <DropDown> {
                                width: Fill, height: Fit
                                padding: {left: 10, right: 10, top: 6, bottom: 6}
                                popup_menu_position: BelowInput
                                // Labels will be set at runtime by init_audio()
                                labels: []
                                values: []
                                selected_item: 0
                                draw_bg: {
                                    instance dark_mode: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 3.0);
                                        let bg = mix((WHITE), (SLATE_700), self.dark_mode);
                                        sdf.fill(bg);
                                        return sdf.result;
                                    }
                                }
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        let light = mix((GRAY_700), (TEXT_PRIMARY), self.focus);
                                        let dark = mix((SLATE_300), (TEXT_PRIMARY_DARK), self.focus);
                                        return mix(light, dark, self.dark_mode);
                                    }
                                }
                                popup_menu: {
                                    width: 250  // Initial width - will be synced at runtime
                                    draw_bg: {
                                        instance dark_mode: 0.0
                                        border_size: 1.0
                                        fn pixel(self) -> vec4 {
                                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                                            let bg = mix((WHITE), (SLATE_800), self.dark_mode);
                                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                                            sdf.fill(bg);
                                            sdf.stroke(border, self.border_size);
                                            return sdf.result;
                                        }
                                    }
                                    menu_item: {
                                        width: Fill
                                        draw_bg: {
                                            instance dark_mode: 0.0
                                            fn pixel(self) -> vec4 {
                                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                                sdf.rect(0., 0., self.rect_size.x, self.rect_size.y);
                                                let base = mix((WHITE), (SLATE_800), self.dark_mode);
                                                let hover_color = mix((GRAY_100), (SLATE_700), self.dark_mode);
                                                sdf.fill(mix(base, hover_color, self.hover));
                                                return sdf.result;
                                            }
                                        }
                                        draw_text: {
                                            instance dark_mode: 0.0
                                            fn get_color(self) -> vec4 {
                                                let light_base = mix((GRAY_700), (TEXT_PRIMARY), self.active);
                                                let dark_base = mix((SLATE_300), (TEXT_PRIMARY_DARK), self.active);
                                                let base = mix(light_base, dark_base, self.dark_mode);
                                                let light_hover = (TEXT_PRIMARY);
                                                let dark_hover = (TEXT_PRIMARY_DARK);
                                                let hover_color = mix(light_hover, dark_hover, self.dark_mode);
                                                return mix(base, hover_color, self.hover);
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        <VerticalDivider> {}

                        // Output device group (fills available space)
                        output_device_group = <View> {
                            width: Fill, height: Fit
                            flow: Right
                            spacing: 8
                            align: {y: 0.5}

                            output_device_label = <Label> {
                                width: 70  // Fixed width for alignment with input label
                                text: "Speaker:"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                            }

                            output_device_dropdown = <DropDown> {
                                width: Fill, height: Fit
                                padding: {left: 10, right: 10, top: 6, bottom: 6}
                                popup_menu_position: BelowInput
                                // Labels will be set at runtime by init_audio()
                                labels: []
                                values: []
                                selected_item: 0
                                draw_bg: {
                                    instance dark_mode: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 3.0);
                                        let bg = mix((WHITE), (SLATE_700), self.dark_mode);
                                        sdf.fill(bg);
                                        return sdf.result;
                                    }
                                }
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        let light = mix((GRAY_700), (TEXT_PRIMARY), self.focus);
                                        let dark = mix((SLATE_300), (TEXT_PRIMARY_DARK), self.focus);
                                        return mix(light, dark, self.dark_mode);
                                    }
                                }
                                popup_menu: {
                                    width: 250  // Initial width - will be synced at runtime
                                    draw_bg: {
                                        instance dark_mode: 0.0
                                        border_size: 1.0
                                        fn pixel(self) -> vec4 {
                                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                                            let bg = mix((WHITE), (SLATE_800), self.dark_mode);
                                            let border = mix((BORDER), (SLATE_600), self.dark_mode);
                                            sdf.fill(bg);
                                            sdf.stroke(border, self.border_size);
                                            return sdf.result;
                                        }
                                    }
                                    menu_item: {
                                        width: Fill
                                        draw_bg: {
                                            instance dark_mode: 0.0
                                            fn pixel(self) -> vec4 {
                                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                                sdf.rect(0., 0., self.rect_size.x, self.rect_size.y);
                                                let base = mix((WHITE), (SLATE_800), self.dark_mode);
                                                let hover_color = mix((GRAY_100), (SLATE_700), self.dark_mode);
                                                sdf.fill(mix(base, hover_color, self.hover));
                                                return sdf.result;
                                            }
                                        }
                                        draw_text: {
                                            instance dark_mode: 0.0
                                            fn get_color(self) -> vec4 {
                                                let light_base = mix((GRAY_700), (TEXT_PRIMARY), self.active);
                                                let dark_base = mix((SLATE_300), (TEXT_PRIMARY_DARK), self.active);
                                                let base = mix(light_base, dark_base, self.dark_mode);
                                                let light_hover = (TEXT_PRIMARY);
                                                let dark_hover = (TEXT_PRIMARY_DARK);
                                                let hover_color = mix(light_hover, dark_hover, self.dark_mode);
                                                return mix(base, hover_color, self.hover);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Prompt input area container
            prompt_container = <View> {
                width: Fill, height: Fit
                flow: Down

                prompt_section = <RoundedView> {
                    width: Fill, height: Fit
                    padding: (PANEL_PADDING)
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn get_color(self) -> vec4 {
                            return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                        }
                    }
                    flow: Down
                    spacing: 8

                    prompt_row = <View> {
                        width: Fill, height: Fit
                        flow: Right
                        spacing: 12
                        align: {y: 0.5}

                        prompt_input = <TextInput> {
                            width: Fill, height: Fit
                            padding: {left: 12, right: 12, top: 10, bottom: 10}
                            empty_text: "Enter prompt to send..."
                            draw_bg: {
                                instance dark_mode: 0.0
                                border_radius: 4.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                    let bg = mix((SLATE_50), (SLATE_700), self.dark_mode);
                                    sdf.fill(bg);
                                    return sdf.result;
                                }
                            }
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                            draw_selection: {
                                color: (INDIGO_200)
                            }
                        }

                        button_group = <View> {
                            width: Fit, height: Fit
                            flow: Right
                            spacing: 8

                            send_prompt_btn = <Button> {
                                width: Fit, height: 35
                                padding: {left: 16, right: 16}
                                text: "Send"
                                draw_text: {
                                    color: (WHITE)
                                    text_style: <FONT_SEMIBOLD>{ font_size: 11.0 }
                                }
                                draw_bg: {
                                    instance color: (ACCENT_BLUE)
                                    instance color_hover: (BLUE_700)
                                    border_radius: 4.0
                                    fn get_color(self) -> vec4 {
                                        return mix(self.color, self.color_hover, self.hover);
                                    }
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                        sdf.fill(self.get_color());
                                        return sdf.result;
                                    }
                                }
                            }

                            reset_btn = <Button> {
                                width: Fit, height: 35
                                padding: {left: 16, right: 16}
                                text: "Reset"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((GRAY_700), (SLATE_300), self.dark_mode);
                                    }
                                }
                                draw_bg: {
                                    instance dark_mode: 0.0
                                    border_radius: 4.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                        let base = mix((HOVER_BG), (SLATE_600), self.dark_mode);
                                        let hover_color = mix((SLATE_200), (SLATE_500), self.dark_mode);
                                        sdf.fill(mix(base, hover_color, self.hover));
                                        return sdf.result;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Splitter - draggable handle with padding
        splitter = <View> {
            width: 16, height: Fill
            margin: { left: 8, right: 8 }
            align: {y: 0.0}
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    // Draw thin line in center
                    sdf.rect(7.0, 16.0, 2.0, self.rect_size.y - 32.0);
                    let color = mix((SLATE_300), (SLATE_600), self.dark_mode);
                    sdf.fill(color);
                    return sdf.result;
                }
            }
            cursor: ColResize
        }

        // System Log panel - adaptive width, top-aligned
        log_section = <View> {
            width: 320, height: Fill
            flow: Right
            align: {y: 0.0}

            // Toggle button column
            toggle_column = <View> {
                width: Fit, height: Fill
                show_bg: true
                draw_bg: {
                    instance dark_mode: 0.0
                    fn pixel(self) -> vec4 {
                        return mix((SLATE_50), (SLATE_800), self.dark_mode);
                    }
                }
                align: {x: 0.5, y: 0.0}
                padding: {left: 4, right: 4, top: 8}

                toggle_log_btn = <Button> {
                    width: Fit, height: Fit
                    padding: {left: 8, right: 8, top: 6, bottom: 6}
                    text: ">"
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_BOLD>{ font_size: 11.0 }
                        fn get_color(self) -> vec4 {
                            return mix((SLATE_500), (SLATE_400), self.dark_mode);
                        }
                    }
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: 4.0
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let base = mix((SLATE_200), (SLATE_600), self.dark_mode);
                            let hover_color = mix((SLATE_300), (SLATE_500), self.dark_mode);
                            sdf.fill(mix(base, hover_color, self.hover));
                            return sdf.result;
                        }
                    }
                }
            }

            // Log content panel
            log_content_column = <RoundedView> {
                width: Fill, height: Fill
                draw_bg: {
                    instance dark_mode: 0.0
                    border_radius: (PANEL_RADIUS)
                    fn get_color(self) -> vec4 {
                        return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                    }
                }
                flow: Down

                log_header = <View> {
                    width: Fill, height: Fit
                    flow: Down
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        fn pixel(self) -> vec4 {
                            return mix((SLATE_50), (SLATE_800), self.dark_mode);
                        }
                    }

                    // Title row
                    log_title_row = <View> {
                        width: Fill, height: Fit
                        padding: {left: 12, right: 12, top: 10, bottom: 6}
                        log_title_label = <Label> {
                            text: "System Log"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 13.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }

                    // Filter row
                    log_filter_row = <View> {
                        width: Fill, height: 32
                        flow: Right
                        align: {y: 0.5}
                        padding: {left: 8, right: 8, bottom: 6}
                        spacing: 6

                        // Level filter dropdown
                        level_filter = <DropDown> {
                            width: 70, height: 24
                            popup_menu_position: BelowInput
                            draw_bg: {
                                color: (HOVER_BG)
                                border_color: (SLATE_200)
                                border_radius: 2.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    // Background
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                                    sdf.fill((HOVER_BG));
                                    // Down arrow on right side
                                    let ax = self.rect_size.x - 12.0;
                                    let ay = self.rect_size.y * 0.5 - 2.0;
                                    sdf.move_to(ax - 3.0, ay);
                                    sdf.line_to(ax, ay + 4.0);
                                    sdf.line_to(ax + 3.0, ay);
                                    sdf.stroke((TEXT_PRIMARY), 1.5);
                                    return sdf.result;
                                }
                            }
                            draw_text: {
                                text_style: <FONT_MEDIUM>{ font_size: 10.0 }
                                fn get_color(self) -> vec4 {
                                    return (TEXT_PRIMARY);
                                }
                            }
                            popup_menu: {
                                draw_bg: {
                                    color: (WHITE)
                                    border_color: (BORDER)
                                    border_size: 1.0
                                    border_radius: 2.0
                                }
                                menu_item: {
                                    draw_bg: {
                                        color: (WHITE)
                                        color_hover: (GRAY_100)
                                    }
                                    draw_text: {
                                        fn get_color(self) -> vec4 {
                                            return mix(
                                                mix((GRAY_700), (TEXT_PRIMARY), self.active),
                                                (TEXT_PRIMARY),
                                                self.hover
                                            );
                                        }
                                    }
                                }
                            }
                            labels: ["ALL", "DEBUG", "INFO", "WARN", "ERROR"]
                            values: [ALL, DEBUG, INFO, WARN, ERROR]
                        }

                        // Node filter dropdown
                        node_filter = <DropDown> {
                            width: 85, height: 24
                            popup_menu_position: BelowInput
                            draw_bg: {
                                color: (HOVER_BG)
                                border_color: (SLATE_200)
                                border_radius: 2.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    // Background
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 2.0);
                                    sdf.fill((HOVER_BG));
                                    // Down arrow on right side
                                    let ax = self.rect_size.x - 12.0;
                                    let ay = self.rect_size.y * 0.5 - 2.0;
                                    sdf.move_to(ax - 3.0, ay);
                                    sdf.line_to(ax, ay + 4.0);
                                    sdf.line_to(ax + 3.0, ay);
                                    sdf.stroke((TEXT_PRIMARY), 1.5);
                                    return sdf.result;
                                }
                            }
                            draw_text: {
                                text_style: <FONT_MEDIUM>{ font_size: 10.0 }
                                fn get_color(self) -> vec4 {
                                    return (TEXT_PRIMARY);
                                }
                            }
                            popup_menu: {
                                draw_bg: {
                                    color: (WHITE)
                                    border_color: (BORDER)
                                    border_size: 1.0
                                    border_radius: 2.0
                                }
                                menu_item: {
                                    draw_bg: {
                                        color: (WHITE)
                                        color_hover: (GRAY_100)
                                    }
                                    draw_text: {
                                        fn get_color(self) -> vec4 {
                                            return mix(
                                                mix((GRAY_700), (TEXT_PRIMARY), self.active),
                                                (TEXT_PRIMARY),
                                                self.hover
                                            );
                                        }
                                    }
                                }
                            }
                            labels: ["All Nodes", "ASR", "TTS", "LLM", "Bridge", "Monitor", "App"]
                            values: [ALL, ASR, TTS, LLM, BRIDGE, MONITOR, APP]
                        }

                        // Search icon
                        search_icon = <View> {
                            width: 20, height: 24
                            align: {x: 0.5, y: 0.5}
                            show_bg: true
                            draw_bg: {
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    let c = self.rect_size * 0.5;
                                    // Magnifying glass circle
                                    sdf.circle(c.x - 2.0, c.y - 2.0, 5.0);
                                    sdf.stroke((GRAY_500), 1.5);
                                    // Handle
                                    sdf.move_to(c.x + 1.5, c.y + 1.5);
                                    sdf.line_to(c.x + 6.0, c.y + 6.0);
                                    sdf.stroke((GRAY_500), 1.5);
                                    return sdf.result;
                                }
                            }
                        }

                        // Search field
                        log_search = <TextInput> {
                            width: Fill, height: 24
                            empty_text: "Search..."
                            draw_bg: {
                                instance dark_mode: 0.0
                                border_radius: 2.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                    let bg = mix((WHITE), (SLATE_700), self.dark_mode);
                                    sdf.fill(bg);
                                    return sdf.result;
                                }
                            }
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 10.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                            draw_selection: {
                                color: (INDIGO_200)
                            }
                        }

                        // Copy to clipboard button
                        copy_log_btn = <Button> {
                            width: 28, height: 24
                            text: ""
                            draw_bg: {
                                instance hover: 0.0
                                instance pressed: 0.0
                                instance copied: 0.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    let c = self.rect_size * 0.5;

                                    // Background - flash green when copied
                                    sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                    let bg_color = mix((BORDER), (GRAY_300), self.hover);
                                    let bg_color = mix(bg_color, (TEXT_MUTED), self.pressed);
                                    let bg_color = mix(bg_color, #22c55e, self.copied);
                                    sdf.fill(bg_color);

                                    // Icon color - white when copied for contrast
                                    let icon_color = mix((GRAY_600), #ffffff, self.copied);

                                    // Clipboard icon - back rectangle
                                    sdf.box(c.x - 4.0, c.y - 2.0, 8.0, 9.0, 1.0);
                                    sdf.stroke(icon_color, 1.2);

                                    // Clipboard icon - front rectangle (overlapping)
                                    sdf.box(c.x - 2.0, c.y - 5.0, 8.0, 9.0, 1.0);
                                    sdf.fill(bg_color);
                                    sdf.box(c.x - 2.0, c.y - 5.0, 8.0, 9.0, 1.0);
                                    sdf.stroke(icon_color, 1.2);

                                    return sdf.result;
                                }
                            }
                            animator: {
                                hover = {
                                    default: off
                                    off = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                                    on = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                                }
                                pressed = {
                                    default: off
                                    off = { from: {all: Forward {duration: 0.05}} apply: {draw_bg: {pressed: 0.0}} }
                                    on = { from: {all: Forward {duration: 0.02}} apply: {draw_bg: {pressed: 1.0}} }
                                }
                            }
                        }
                    }
                }

                log_scroll = <ScrollYView> {
                    width: Fill, height: Fill
                    flow: Down
                    scroll_bars: <ScrollBars> {
                        show_scroll_x: false
                        show_scroll_y: true
                    }

                    log_content_wrapper = <View> {
                        width: Fill, height: Fit
                        padding: { left: 12, right: 12, top: 8, bottom: 8 }
                        flow: Down

                        log_content = <Markdown> {
                            width: Fill, height: Fit
                            font_size: 10.0
                            font_color: (GRAY_600)
                            paragraph_spacing: 4

                            draw_normal: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 10.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((GRAY_600), (SLATE_300), self.dark_mode);
                                }
                            }
                            draw_bold: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 10.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((GRAY_600), (SLATE_300), self.dark_mode);
                                }
                            }
                            draw_fixed: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 9.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((GRAY_600), (SLATE_300), self.dark_mode);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
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
    log_level_filter: usize,  // 0=ALL, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR
    #[rust]
    log_node_filter: usize,   // 0=ALL, 1=ASR, 2=TTS, 3=LLM, 4=Bridge, 5=Monitor, 6=App
    #[rust]
    log_entries: Vec<String>,  // Raw log entries for filtering

    // Dropdown width caching for popup menu sync
    #[rust]
    dropdown_widths_initialized: bool,
    #[rust]
    cached_input_dropdown_width: f64,
    #[rust]
    cached_output_dropdown_width: f64,

    // AEC toggle state
    #[rust]
    aec_enabled: bool,
    // Note: AEC blink animation is now shader-driven (self.time), no timer needed

    // Dora integration
    #[rust]
    dora_integration: Option<DoraIntegration>,
    #[rust]
    dataflow_path: Option<PathBuf>,
    #[rust]
    dora_timer: Timer,
    #[rust]
    copy_chat_feedback_timer: Timer,
    #[rust]
    copy_log_feedback_timer: Timer,
    #[rust]
    chat_messages: Vec<ChatMessageEntry>,
    #[rust]
    last_chat_count: usize,
    // Pending streaming messages (updated in-place, removed when streaming ends)
    #[rust]
    pending_streaming_messages: Vec<ChatMessageEntry>,

    // Audio playback
    #[rust]
    audio_player: Option<std::sync::Arc<crate::audio_player::AudioPlayer>>,
    // Participant audio levels for decay animation (matches conference-dashboard)
    #[rust]
    participant_levels: [f64; 3],  // 0=student1, 1=student2, 2=tutor
}

impl Widget for MoFaFMScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        // Initialize audio and log bridge on first event
        if !self.audio_initialized {
            // Initialize log bridge to capture Rust logs
            log_bridge::init();
            self.init_audio(cx);
            self.audio_initialized = true;
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

        // Handle copy chat feedback timer - reset animation
        if self.copy_chat_feedback_timer.is_event(event).is_some() {
            self.view.button(ids!(left_column.chat_container.chat_section.chat_header.copy_chat_btn))
                .apply_over(cx, live!{ draw_bg: { copied: 0.0 } });
            self.view.redraw(cx);
        }

        // Handle copy log feedback timer - reset animation
        if self.copy_log_feedback_timer.is_event(event).is_some() {
            self.view.button(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn))
                .apply_over(cx, live!{ draw_bg: { copied: 0.0 } });
            self.view.redraw(cx);
        }

        // Handle AEC toggle button click
        // Note: AEC blink animation is now shader-driven, no timer needed
        let aec_btn = self.view.view(ids!(audio_container.aec_container.aec_group.aec_toggle_btn));
        match event.hits(cx, aec_btn.area()) {
            Hit::FingerUp(_) => {
                self.aec_enabled = !self.aec_enabled;
                let enabled_val = if self.aec_enabled { 1.0 } else { 0.0 };
                self.view.view(ids!(audio_container.aec_container.aec_group.aec_toggle_btn))
                    .apply_over(cx, live!{ draw_bg: { enabled: (enabled_val) } });
                self.view.redraw(cx);
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

        // Handle input device selection
        if let Some(item) = self.view.drop_down(ids!(audio_container.device_container.device_selectors.input_device_group.input_device_dropdown)).selected(actions) {
            if item < self.input_devices.len() {
                let device_name = self.input_devices[item].clone();
                self.select_input_device(cx, &device_name);
            }
        }

        // Handle output device selection
        if let Some(item) = self.view.drop_down(ids!(audio_container.device_container.device_selectors.output_device_group.output_device_dropdown)).selected(actions) {
            if item < self.output_devices.len() {
                let device_name = self.output_devices[item].clone();
                self.select_output_device(&device_name);
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

        // Handle copy log button
        if self.view.button(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn)).clicked(actions) {
            self.copy_logs_to_clipboard(cx);
            // Trigger copied feedback animation
            self.view.button(ids!(log_section.log_content_column.log_header.log_filter_row.copy_log_btn))
                .apply_over(cx, live!{ draw_bg: { copied: 1.0 } });
            self.view.redraw(cx);
            // Start timer to reset animation after 1 second
            self.copy_log_feedback_timer = cx.start_timeout(1.0);
        }

        // Handle copy chat button
        if self.view.button(ids!(left_column.chat_container.chat_section.chat_header.copy_chat_btn)).clicked(actions) {
            self.copy_chat_to_clipboard(cx);
            // Trigger copied feedback animation
            self.view.button(ids!(left_column.chat_container.chat_section.chat_header.copy_chat_btn))
                .apply_over(cx, live!{ draw_bg: { copied: 1.0 } });
            self.view.redraw(cx);
            // Start timer to reset animation after 1 second
            self.copy_chat_feedback_timer = cx.start_timeout(1.0);
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
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        // Update popup menu widths to match dropdown widths
        // This handles first-frame zero width and caches values for performance
        let input_dropdown = self.view.drop_down(ids!(audio_container.device_container.device_selectors.input_device_group.input_device_dropdown));
        let input_width = input_dropdown.area().rect(cx).size.x;

        // Only update if width changed significantly (> 1px) to avoid unnecessary apply_over calls
        if input_width > 0.0 && (input_width - self.cached_input_dropdown_width).abs() > 1.0 {
            self.cached_input_dropdown_width = input_width;
            input_dropdown.apply_over(cx, live! {
                popup_menu: { width: (input_width) }
            });
        }

        let output_dropdown = self.view.drop_down(ids!(audio_container.device_container.device_selectors.output_device_group.output_device_dropdown));
        let output_width = output_dropdown.area().rect(cx).size.x;

        // Only update if width changed significantly (> 1px)
        if output_width > 0.0 && (output_width - self.cached_output_dropdown_width).abs() > 1.0 {
            self.cached_output_dropdown_width = output_width;
            output_dropdown.apply_over(cx, live! {
                popup_menu: { width: (output_width) }
            });
        }

        // Force an extra redraw on first frame to ensure widths are properly captured
        // This fixes the issue where first click shows narrow popup (width=0 on first frame)
        if !self.dropdown_widths_initialized {
            self.dropdown_widths_initialized = true;
            self.view.redraw(cx);
        }

        self.view.draw_walk(cx, scope, walk)
    }
}

impl MoFaFMScreen {
    /// Initialize audio manager and populate device dropdowns
    fn init_audio(&mut self, cx: &mut Cx) {
        let mut audio_manager = crate::audio::AudioManager::new();

        // Get input devices
        let input_devices = audio_manager.get_input_devices();
        let input_labels: Vec<String> = input_devices.iter().map(|d| {
            if d.is_default {
                format!("{} (Default)", d.name)
            } else {
                d.name.clone()
            }
        }).collect();
        self.input_devices = input_devices.iter().map(|d| d.name.clone()).collect();

        // Get output devices
        let output_devices = audio_manager.get_output_devices();
        let output_labels: Vec<String> = output_devices.iter().map(|d| {
            if d.is_default {
                format!("{} (Default)", d.name)
            } else {
                d.name.clone()
            }
        }).collect();
        self.output_devices = output_devices.iter().map(|d| d.name.clone()).collect();

        // Populate input dropdown
        if !input_labels.is_empty() {
            let dropdown = self.view.drop_down(ids!(audio_container.device_container.device_selectors.input_device_group.input_device_dropdown));
            dropdown.set_labels(cx, input_labels);
            dropdown.set_selected_item(cx, 0);
        }

        // Populate output dropdown
        if !output_labels.is_empty() {
            let dropdown = self.view.drop_down(ids!(audio_container.device_container.device_selectors.output_device_group.output_device_dropdown));
            dropdown.set_labels(cx, output_labels);
            dropdown.set_selected_item(cx, 0);
        }

        // Start mic monitoring with default device
        if let Err(e) = audio_manager.start_mic_monitoring(None) {
            eprintln!("Failed to start mic monitoring: {}", e);
        }

        self.audio_manager = Some(audio_manager);

        // Initialize audio player for TTS playback (32kHz for PrimeSpeech)
        match crate::audio_player::create_audio_player(32000) {
            Ok(player) => {
                ::log::info!("Audio player initialized (32kHz)");
                self.audio_player = Some(player);
            }
            Err(e) => {
                ::log::error!("Failed to create audio player: {}", e);
            }
        }

        // Start timer for mic level updates (50ms for smooth visualization)
        self.audio_timer = cx.start_interval(0.05);

        // Start dora timer for participant panel updates (needed for audio visualization)
        self.dora_timer = cx.start_interval(0.1);

        // AEC enabled by default (blink animation is shader-driven, no timer needed)
        self.aec_enabled = true;

        // Initialize demo log entries
        self.init_demo_logs(cx);

        self.view.redraw(cx);
    }

    /// Initialize log entries with a startup message
    fn init_demo_logs(&mut self, cx: &mut Cx) {
        // Start with empty logs - real logs will come from log_bridge
        self.log_entries = vec![
            "[INFO] [App] MoFA FM initialized".to_string(),
            "[INFO] [App] System log ready - Rust logs will appear here".to_string(),
        ];

        // Update the log display
        self.update_log_display(cx);
    }
    /// Update mic level LEDs based on current audio input
    fn update_mic_level(&mut self, cx: &mut Cx) {
        let level = if let Some(ref audio_manager) = self.audio_manager {
            audio_manager.get_mic_level()
        } else {
            return;
        };

        // Map level (0.0-1.0) to 5 LEDs
        // Use non-linear scaling for better visualization (human hearing is logarithmic)
        let scaled_level = (level * 3.0).min(1.0); // Amplify for visibility
        let active_leds = (scaled_level * 5.0).ceil() as u32;

        // Colors as vec4: green=#22c55f, yellow=#eab308, orange=#f97316, red=#ef4444, off=#e2e8f0
        let green = vec4(0.133, 0.773, 0.373, 1.0);
        let yellow = vec4(0.918, 0.702, 0.031, 1.0);
        let orange = vec4(0.976, 0.451, 0.086, 1.0);
        let red = vec4(0.937, 0.267, 0.267, 1.0);
        let off = vec4(0.886, 0.910, 0.941, 1.0);

        // LED colors by index: 0,1=green, 2=yellow, 3=orange, 4=red
        let led_colors = [green, green, yellow, orange, red];
        let led_ids = [
            ids!(audio_container.mic_container.mic_group.mic_level_meter.mic_led_1),
            ids!(audio_container.mic_container.mic_group.mic_level_meter.mic_led_2),
            ids!(audio_container.mic_container.mic_group.mic_level_meter.mic_led_3),
            ids!(audio_container.mic_container.mic_group.mic_level_meter.mic_led_4),
            ids!(audio_container.mic_container.mic_group.mic_level_meter.mic_led_5),
        ];

        for (i, led_id) in led_ids.iter().enumerate() {
            let is_active = (i + 1) as u32 <= active_leds;
            let color = if is_active { led_colors[i] } else { off };
            self.view.view(led_id.clone()).apply_over(cx, live! {
                draw_bg: { color: (color) }
            });
        }

        self.view.redraw(cx);
    }

    /// Select input device for mic monitoring
    fn select_input_device(&mut self, cx: &mut Cx, device_name: &str) {
        if let Some(ref mut audio_manager) = self.audio_manager {
            if let Err(e) = audio_manager.set_input_device(device_name) {
                eprintln!("Failed to set input device '{}': {}", device_name, e);
            }
        }
        self.view.redraw(cx);
    }

    /// Select output device
    fn select_output_device(&mut self, device_name: &str) {
        if let Some(ref mut audio_manager) = self.audio_manager {
            audio_manager.set_output_device(device_name);
        }
    }

    fn toggle_log_panel(&mut self, cx: &mut Cx) {
        self.log_panel_collapsed = !self.log_panel_collapsed;

        if self.log_panel_width == 0.0 {
            self.log_panel_width = 320.0;
        }

        if self.log_panel_collapsed {
            // Collapse: hide log content, show only toggle button
            self.view.view(ids!(log_section)).apply_over(cx, live!{ width: Fit });
            self.view.view(ids!(log_section.log_content_column)).set_visible(cx, false);
            self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).set_text(cx, "<");
            self.view.view(ids!(splitter)).apply_over(cx, live!{ width: 0 });
        } else {
            // Expand: show log content at saved width
            let width = self.log_panel_width;
            self.view.view(ids!(log_section)).apply_over(cx, live!{ width: (width) });
            self.view.view(ids!(log_section.log_content_column)).set_visible(cx, true);
            self.view.button(ids!(log_section.toggle_column.toggle_log_btn)).set_text(cx, ">");
            self.view.view(ids!(splitter)).apply_over(cx, live!{ width: 16 });
        }

        self.view.redraw(cx);
    }

    fn resize_log_panel(&mut self, cx: &mut Cx, abs_x: f64) {
        let container_rect = self.view.area().rect(cx);
        let padding = 16.0; // Match screen padding
        let new_log_width = (container_rect.pos.x + container_rect.size.x - abs_x - padding)
            .max(150.0)  // Minimum log panel width
            .min(container_rect.size.x - 400.0);  // Leave space for main content

        self.log_panel_width = new_log_width;

        self.view.view(ids!(log_section)).apply_over(cx, live!{
            width: (new_log_width)
        });

        self.view.redraw(cx);
    }

    /// Update log display based on current filter and search
    fn update_log_display(&mut self, cx: &mut Cx) {
        let search_text = self.view.text_input(ids!(log_section.log_content_column.log_header.log_filter_row.log_search)).text().to_lowercase();
        let level_filter = self.log_level_filter;
        let node_filter = self.log_node_filter;

        // Filter log entries
        let filtered_logs: Vec<&String> = self.log_entries.iter().filter(|entry| {
            // Level filter: 0=ALL, 1=DEBUG, 2=INFO, 3=WARN, 4=ERROR
            let level_match = match level_filter {
                0 => true, // ALL
                1 => entry.contains("[DEBUG]"),
                2 => entry.contains("[INFO]"),
                3 => entry.contains("[WARN]"),
                4 => entry.contains("[ERROR]"),
                _ => true,
            };

            // Node filter: 0=ALL, 1=ASR, 2=TTS, 3=LLM, 4=Bridge, 5=Monitor, 6=App
            let node_match = match node_filter {
                0 => true, // All Nodes
                1 => entry.contains("[ASR]") || entry.to_lowercase().contains("asr"),
                2 => entry.contains("[TTS]") || entry.to_lowercase().contains("tts"),
                3 => entry.contains("[LLM]") || entry.to_lowercase().contains("llm"),
                4 => entry.contains("[Bridge]") || entry.to_lowercase().contains("bridge"),
                5 => entry.contains("[Monitor]") || entry.to_lowercase().contains("monitor"),
                6 => entry.contains("[App]") || entry.to_lowercase().contains("app"),
                _ => true,
            };

            // Search filter
            let search_match = search_text.is_empty() || entry.to_lowercase().contains(&search_text);

            level_match && node_match && search_match
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

    /// Copy filtered logs to clipboard
    fn copy_logs_to_clipboard(&mut self, cx: &mut Cx) {
        let search_text = self.view.text_input(ids!(log_section.log_content_column.log_header.log_filter_row.log_search)).text().to_lowercase();
        let level_filter = self.log_level_filter;
        let node_filter = self.log_node_filter;

        // Filter log entries (same as update_log_display)
        let filtered_logs: Vec<&String> = self.log_entries.iter().filter(|entry| {
            let level_match = match level_filter {
                0 => true,
                1 => entry.contains("[DEBUG]"),
                2 => entry.contains("[INFO]"),
                3 => entry.contains("[WARN]"),
                4 => entry.contains("[ERROR]"),
                _ => true,
            };
            let node_match = match node_filter {
                0 => true,
                1 => entry.contains("[ASR]") || entry.to_lowercase().contains("asr"),
                2 => entry.contains("[TTS]") || entry.to_lowercase().contains("tts"),
                3 => entry.contains("[LLM]") || entry.to_lowercase().contains("llm"),
                4 => entry.contains("[Bridge]") || entry.to_lowercase().contains("bridge"),
                5 => entry.contains("[Monitor]") || entry.to_lowercase().contains("monitor"),
                6 => entry.contains("[App]") || entry.to_lowercase().contains("app"),
                _ => true,
            };
            let search_match = search_text.is_empty() || entry.to_lowercase().contains(&search_text);
            level_match && node_match && search_match
        }).collect();

        let log_text = if filtered_logs.is_empty() {
            "No log entries".to_string()
        } else {
            // Use single newlines for clipboard (plain text, not Markdown)
            filtered_logs.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
        };

        cx.copy_to_clipboard(&log_text);
    }

    /// Copy chat messages to clipboard
    fn copy_chat_to_clipboard(&mut self, cx: &mut Cx) {
        let chat_text = if self.chat_messages.is_empty() {
            "No chat messages".to_string()
        } else {
            self.chat_messages.iter().map(|msg| {
                format!("[{}] {}", msg.sender, msg.content)
            }).collect::<Vec<_>>().join("\n\n")
        };

        cx.copy_to_clipboard(&chat_text);
    }

    /// Add a log entry
    pub fn add_log(&mut self, cx: &mut Cx, entry: &str) {
        self.log_entries.push(entry.to_string());
        self.update_log_display(cx);
    }

    /// Poll Rust log messages and add them to the system log
    fn poll_rust_logs(&mut self, cx: &mut Cx) {
        let logs = log_bridge::poll_logs();
        if logs.is_empty() {
            return;
        }

        for log_msg in logs {
            self.log_entries.push(log_msg.format());
        }

        // Only update display if we got new logs
        self.update_log_display(cx);
    }

    /// Clear all logs
    pub fn clear_logs(&mut self, cx: &mut Cx) {
        self.log_entries.clear();
        self.update_log_display(cx);
    }

    // =====================================================
    // Dora Integration Methods
    // =====================================================

    /// Initialize dora integration (lazy initialization)
    fn init_dora(&mut self, cx: &mut Cx) {
        if self.dora_integration.is_some() {
            return;
        }

        ::log::info!("Initializing Dora integration");
        let integration = DoraIntegration::new();
        self.dora_integration = Some(integration);

        // Start timer to poll for dora events (100ms interval)
        self.dora_timer = cx.start_interval(0.1);

        // Look for default dataflow relative to current working directory
        let dataflow_path = std::env::current_dir()
            .ok()
            .map(|p| p.join("dataflow").join("voice-chat.yml"))
            .filter(|p| p.exists());
        self.dataflow_path = dataflow_path;

        ::log::info!("Dora integration initialized, dataflow: {:?}", self.dataflow_path);
    }

    /// Start a dataflow
    pub fn start_dataflow(&mut self, cx: &mut Cx, path: impl Into<PathBuf>) {
        self.init_dora(cx);

        let path = path.into();
        if let Some(ref dora) = self.dora_integration {
            if dora.start_dataflow(&path) {
                ::log::info!("Starting dataflow: {:?}", path);
                self.dataflow_path = Some(path);
                self.add_log(cx, &format!("[INFO] [App] Starting dataflow..."));
            } else {
                ::log::error!("Failed to start dataflow: {:?}", path);
                self.add_log(cx, &format!("[ERROR] [App] Failed to start dataflow"));
            }
        }
    }

    /// Stop the current dataflow
    pub fn stop_dataflow(&mut self, cx: &mut Cx) {
        if let Some(ref dora) = self.dora_integration {
            if dora.stop_dataflow() {
                ::log::info!("Stopping dataflow");
                self.add_log(cx, "[INFO] [App] Dataflow stopped");
            }
        }
    }

    /// Poll for dora events and update UI
    fn poll_dora_events(&mut self, cx: &mut Cx) {
        // Get dora events if integration is running
        let events = if let Some(ref dora) = self.dora_integration {
            dora.poll_events()
        } else {
            Vec::new()  // Continue to update audio visualization even without dora
        };

        for event in events {
            match event {
                DoraEvent::DataflowStarted { dataflow_id } => {
                    ::log::info!("Dataflow started: {}", dataflow_id);
                    self.add_log(cx, &format!("[INFO] [App] Dataflow started: {}", dataflow_id));
                    self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Connected);
                }
                DoraEvent::DataflowStopped => {
                    ::log::info!("Dataflow stopped");
                    self.add_log(cx, "[INFO] [App] Dataflow stopped");
                    self.view.mofa_hero(ids!(left_column.mofa_hero)).set_running(cx, false);
                    self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Stopped);
                }
                DoraEvent::BridgeConnected { bridge_name } => {
                    ::log::info!("Bridge connected: {}", bridge_name);
                    let display_name = Self::format_bridge_name(&bridge_name);
                    self.add_log(cx, &format!("[INFO] [Bridge] {} connected to dora dataflow", display_name));
                }
                DoraEvent::BridgeDisconnected { bridge_name } => {
                    ::log::info!("Bridge disconnected: {}", bridge_name);
                    let display_name = Self::format_bridge_name(&bridge_name);
                    self.add_log(cx, &format!("[WARN] [Bridge] {} disconnected from dora dataflow", display_name));
                }
                DoraEvent::ChatReceived { message } => {
                    // Handle streaming message consolidation
                    // Match by BOTH sender AND session_id to avoid confusion between participants
                    let sender = message.sender.clone();
                    let session_id = message.session_id.clone();

                    // Debug logging for chat messages
                    ::log::info!("[Chat] sender={}, session_id={:?}, is_streaming={}, content_len={}, pending_count={}, finalized_count={}",
                        sender,
                        session_id,
                        message.is_streaming,
                        message.content.len(),
                        self.pending_streaming_messages.len(),
                        self.chat_messages.len()
                    );

                    if message.is_streaming {
                        // Update or create pending streaming message
                        let entry = ChatMessageEntry {
                            sender: sender.clone(),
                            content: message.content.clone(),
                            timestamp: message.timestamp,
                            is_streaming: true,
                            session_id: session_id.clone(),
                        };

                        // Find existing pending message with same sender AND session_id
                        // (or same sender if session_id is "unknown")
                        let found = self.pending_streaming_messages.iter_mut()
                            .find(|m| {
                                m.sender == sender && (
                                    // Match by session_id if both have real session_ids
                                    (m.session_id.as_ref().map(|s| s != "unknown").unwrap_or(false)
                                        && session_id.as_ref().map(|s| s != "unknown").unwrap_or(false)
                                        && m.session_id == session_id)
                                    ||
                                    // For "unknown" session_ids, just match by sender
                                    (m.session_id.as_ref().map(|s| s == "unknown").unwrap_or(true)
                                        && session_id.as_ref().map(|s| s == "unknown").unwrap_or(true))
                                )
                            });

                        if let Some(pending) = found {
                            // Update existing pending message
                            pending.content = entry.content;
                            pending.timestamp = entry.timestamp;
                            pending.session_id = entry.session_id;
                        } else {
                            // Add new pending message
                            self.pending_streaming_messages.push(entry);
                        }

                        // Update display with pending messages (shown but not finalized)
                        self.update_chat_display(cx);
                    } else {
                        // Streaming complete - finalize the message
                        let entry = ChatMessageEntry {
                            sender: sender.clone(),
                            content: message.content.clone(),
                            timestamp: message.timestamp,
                            is_streaming: false,
                            session_id: session_id.clone(),
                        };

                        // Remove from pending - match by sender AND session_id
                        self.pending_streaming_messages.retain(|m| {
                            !(m.sender == sender && (
                                m.session_id == session_id ||
                                (m.session_id.as_ref().map(|s| s == "unknown").unwrap_or(true)
                                    && session_id.as_ref().map(|s| s == "unknown").unwrap_or(true))
                            ))
                        });

                        // Add to finalized messages
                        self.chat_messages.push(entry);
                        // Keep chat messages bounded (prevents O(n²) slowdown and markdown overflow)
                        if self.chat_messages.len() > 500 {
                            self.chat_messages.remove(0);
                        }
                        self.update_chat_display(cx);
                    }
                }
                DoraEvent::LogReceived { entry } => {
                    let level_str = format!("{:?}", entry.level).to_uppercase();
                    let log_line = format!("[{}] [{}] {}", level_str, entry.node_id, entry.message);
                    self.add_log(cx, &log_line);
                }
                DoraEvent::AudioReceived { data } => {
                    // Forward to audio player for playback
                    if let Some(ref player) = self.audio_player {
                        player.write_audio(&data.samples, data.participant_id.clone());
                    }
                }
                // NOTE: ParticipantAudioReceived removed - LED visualization calculated below
                // from output waveform (more accurate since it reflects what's actually playing)
                DoraEvent::Error { message } => {
                    ::log::error!("Dora error: {}", message);
                    self.add_log(cx, &format!("[ERROR] [Dora] {}", message));
                    self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Failed);
                }
            }
        }

        // Update audio buffer level in mofa_hero (from audio player)
        let (is_playing, active_idx, waveform_data) = if let Some(ref player) = self.audio_player {
            let buffer_pct = player.buffer_fill_percentage() / 100.0;
            self.view.mofa_hero(ids!(left_column.mofa_hero)).set_buffer_level(cx, buffer_pct);
            (player.is_playing(), player.current_participant_idx(), player.get_waveform_data())
        } else {
            (false, None, Vec::new())
        };

        {
            // Calculate band levels from waveform data (same as conference-dashboard)
            let band_levels: [f32; 8] = if waveform_data.is_empty() {
                [0.0f32; 8]
            } else {
                let samples = &waveform_data;
                let band_size = samples.len() / 8;
                let mut levels = [0.0f32; 8];
                let peak = samples.iter().map(|s| s.abs()).fold(0.0f32, |a, b| a.max(b));
                let norm_factor = if peak > 0.01 { 1.0 / peak } else { 1.0 };

                for i in 0..8 {
                    let start = i * band_size;
                    let end = ((i + 1) * band_size).min(samples.len());
                    if end > start {
                        let sum_sq: f32 = samples[start..end].iter().map(|s| s * s).sum();
                        let rms = (sum_sq / (end - start) as f32).sqrt();
                        levels[i] = (rms * norm_factor * 1.5).clamp(0.0, 1.0);
                    }
                }
                levels
            };

            // Update participant panels using direct apply_over (exactly like conference-dashboard)
            let panel_ids: [&[LiveId]; 3] = [
                ids!(left_column.participant_container.participant_bar.student1_panel),
                ids!(left_column.participant_container.participant_bar.student2_panel),
                ids!(left_column.participant_container.participant_bar.tutor_panel),
            ];

            for (i, panel_id) in panel_ids.into_iter().enumerate() {
                let panel = self.view.view(panel_id);
                let is_current_audio_speaker = is_playing && active_idx == Some(i);

                // Calculate level with decay (matches conference-dashboard)
                let new_level = if is_current_audio_speaker && !waveform_data.is_empty() {
                    let samples = &waveform_data;
                    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
                    let rms = (sum_sq / samples.len() as f32).sqrt();
                    (rms * 2.0).clamp(0.0, 1.0) as f64
                } else {
                    self.participant_levels[i] * 0.85
                };
                self.participant_levels[i] = new_level;

                // Update waveform - exactly like conference-dashboard
                let active_val = if is_current_audio_speaker { 1.0 } else { 0.0 };
                panel.view(ids!(waveform)).apply_over(cx, live! {
                    draw_bg: {
                        level: (new_level),
                        active: (active_val),
                        band0: (if is_current_audio_speaker { band_levels[0] as f64 } else { 0.0 }),
                        band1: (if is_current_audio_speaker { band_levels[1] as f64 } else { 0.0 }),
                        band2: (if is_current_audio_speaker { band_levels[2] as f64 } else { 0.0 }),
                        band3: (if is_current_audio_speaker { band_levels[3] as f64 } else { 0.0 }),
                        band4: (if is_current_audio_speaker { band_levels[4] as f64 } else { 0.0 }),
                        band5: (if is_current_audio_speaker { band_levels[5] as f64 } else { 0.0 }),
                        band6: (if is_current_audio_speaker { band_levels[6] as f64 } else { 0.0 }),
                        band7: (if is_current_audio_speaker { band_levels[7] as f64 } else { 0.0 }),
                    }
                });
            }
        }
    }

    /// Send prompt to dora
    fn send_prompt(&mut self, cx: &mut Cx) {
        let input_text = self.view.text_input(ids!(left_column.prompt_container.prompt_section.prompt_row.prompt_input)).text();
        // Use default prompt if input is empty
        let prompt_text = if input_text.is_empty() {
            "开始吧".to_string()
        } else {
            input_text
        };

        // Initialize dora if needed
        self.init_dora(cx);

        // Add user message to chat
        let user_msg = ChatMessageEntry::new("You", prompt_text.clone());
        self.chat_messages.push(user_msg);
        // Keep chat messages bounded (prevents O(n²) slowdown and markdown overflow)
        if self.chat_messages.len() > 500 {
            self.chat_messages.remove(0);
        }
        self.update_chat_display(cx);

        // Clear input field
        self.view.text_input(ids!(left_column.prompt_container.prompt_section.prompt_row.prompt_input)).set_text(cx, "");

        // Send through dora if connected
        if let Some(ref dora) = self.dora_integration {
            if dora.is_running() {
                dora.send_prompt(&prompt_text);
                self.add_log(cx, &format!("[INFO] [App] Sent prompt: {}",
                    if prompt_text.len() > 50 { format!("{}...", &prompt_text[..50]) } else { prompt_text.to_string() }));
            } else {
                self.add_log(cx, "[WARN] [App] Dataflow not running - prompt not sent to LLM");
            }
        }

        self.view.redraw(cx);
    }

    /// Reset conversation - sends reset to conference controller
    fn reset_conversation(&mut self, cx: &mut Cx) {
        ::log::info!("Reset clicked");

        // Send reset command to conference controller via dora
        if let Some(ref dora) = self.dora_integration {
            if dora.is_running() {
                dora.send_control("reset");
                self.add_log(cx, "[INFO] [App] Sent reset command to conference controller");
            } else {
                self.add_log(cx, "[WARN] [App] Dataflow not running - reset not sent");
            }
        }

        // Clear chat messages and pending streaming messages
        self.chat_messages.clear();
        self.pending_streaming_messages.clear();
        self.update_chat_display(cx);

        // Clear prompt input
        self.view.text_input(ids!(left_column.prompt_container.prompt_section.prompt_row.prompt_input)).set_text(cx, "");

        // Reset audio player buffer
        if let Some(ref audio_player) = self.audio_player {
            audio_player.reset();
            self.add_log(cx, "[INFO] [App] Audio buffer reset");
        }

        self.view.redraw(cx);
    }

    /// Update chat display with current messages
    fn update_chat_display(&mut self, cx: &mut Cx) {
        // Combine finalized messages with pending streaming messages
        let all_messages: Vec<&ChatMessageEntry> = self.chat_messages.iter()
            .chain(self.pending_streaming_messages.iter())
            .collect();

        let chat_text = if all_messages.is_empty() {
            "Waiting for conversation...".to_string()
        } else {
            all_messages.into_iter()
                .map(|msg| {
                    let timestamp = Self::format_timestamp(msg.timestamp);
                    let streaming_indicator = if msg.is_streaming { " ⌛" } else { "" };
                    format!("**{}**{} ({}):  \n{}", msg.sender, streaming_indicator, timestamp, msg.content)
                })
                .collect::<Vec<_>>()
                .join("\n\n---\n\n")
        };

        ::log::debug!("[Chat] update_display: text_len={}, finalized={}, pending={}",
            chat_text.len(),
            self.chat_messages.len(),
            self.pending_streaming_messages.len()
        );

        self.view.markdown(ids!(left_column.chat_container.chat_section.chat_scroll.chat_content_wrapper.chat_content))
            .set_text(cx, &chat_text);

        // Auto-scroll to bottom when new messages arrive
        let chat_count = self.chat_messages.len() + self.pending_streaming_messages.len();
        if chat_count > self.last_chat_count {
            self.view.view(ids!(left_column.chat_container.chat_section.chat_scroll))
                .set_scroll_pos(cx, DVec2 { x: 0.0, y: 1e10 });
            self.last_chat_count = chat_count;
        }

        self.view.redraw(cx);
    }

    /// Format Unix timestamp (milliseconds) to readable HH:MM:SS format
    /// Matches conference-dashboard's get_timestamp() format
    fn format_timestamp(timestamp_ms: u64) -> String {
        // Convert milliseconds to seconds
        let total_secs = timestamp_ms / 1000;
        // Get time of day (seconds since midnight UTC)
        let secs_in_day = total_secs % 86400;
        let hours = secs_in_day / 3600;
        let minutes = (secs_in_day % 3600) / 60;
        let seconds = secs_in_day % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    // =====================================================
    // Helper Methods
    // =====================================================

    /// Format bridge node ID to a display-friendly name
    /// e.g., "mofa-audio-player" -> "Audio Player"
    ///       "mofa-system-log" -> "System Log"
    ///       "mofa-prompt-input" -> "Prompt Input"
    fn format_bridge_name(node_id: &str) -> String {
        // Remove "mofa-" prefix if present
        let name = node_id.strip_prefix("mofa-").unwrap_or(node_id);

        // Convert kebab-case to Title Case
        name.split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    // =====================================================
    // MoFA Start/Stop Handlers
    // =====================================================

    /// Handle MoFA start button click
    fn handle_mofa_start(&mut self, cx: &mut Cx) {
        ::log::info!("MoFA Start clicked");

        // Clear chat window and system log
        self.chat_messages.clear();
        self.pending_streaming_messages.clear();
        self.last_chat_count = 0;
        self.update_chat_display(cx);
        self.clear_logs(cx);

        // Initialize dora if not already done
        self.init_dora(cx);

        // Load API keys from preferences
        let env_vars = self.load_api_keys_from_preferences();

        // Log which keys are available
        let has_openai = env_vars.contains_key("OPENAI_API_KEY");
        let has_deepseek = env_vars.contains_key("DEEPSEEK_API_KEY");
        self.add_log(cx, &format!("[INFO] [App] API Keys: OpenAI={}, DeepSeek={}",
            if has_openai { "✓" } else { "✗" },
            if has_deepseek { "✓" } else { "✗" }
        ));

        // Find the dataflow file relative to current working directory
        let dataflow_path = self.dataflow_path.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_default()
                .join("dataflow")
                .join("voice-chat.yml")
        });

        if !dataflow_path.exists() {
            self.add_log(cx, &format!("[ERROR] [App] Dataflow not found: {:?}", dataflow_path));
            self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Failed);
            return;
        }

        self.add_log(cx, &format!("[INFO] [App] Starting dataflow: {:?}", dataflow_path));

        // Update UI state - show connecting
        self.view.mofa_hero(ids!(left_column.mofa_hero)).set_running(cx, true);
        self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Connecting);

        // Start dataflow with environment variables
        if let Some(ref dora) = self.dora_integration {
            if !dora.start_dataflow_with_env(&dataflow_path, env_vars) {
                self.add_log(cx, "[ERROR] [App] Failed to send start command");
                self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Failed);
            }
        }

        self.dataflow_path = Some(dataflow_path);
    }

    /// Handle MoFA stop button click
    fn handle_mofa_stop(&mut self, cx: &mut Cx) {
        ::log::info!("MoFA Stop clicked");

        self.add_log(cx, "[INFO] [App] Force stopping MoFA dataflow...");

        // Show "Stopping" state while stop is in progress
        self.view.mofa_hero(ids!(left_column.mofa_hero)).set_connection_status(cx, ConnectionStatus::Stopping);

        // Force stop dataflow immediately (0s grace period)
        // The actual status update will come from DoraEvent::DataflowStopped
        if let Some(ref dora) = self.dora_integration {
            dora.force_stop_dataflow();
        }

        // Note: Don't set Stopped here - wait for DoraEvent::DataflowStopped
        // to confirm the dataflow actually stopped
    }

    /// Load API keys from preferences
    fn load_api_keys_from_preferences(&self) -> HashMap<String, String> {
        let mut env_vars = HashMap::new();

        // Load preferences
        let prefs = Preferences::load();

        // Get OpenAI API key
        if let Some(provider) = prefs.get_provider("openai") {
            if let Some(ref api_key) = provider.api_key {
                if !api_key.is_empty() {
                    env_vars.insert("OPENAI_API_KEY".to_string(), api_key.clone());
                }
            }
        }

        // Get DeepSeek API key
        if let Some(provider) = prefs.get_provider("deepseek") {
            if let Some(ref api_key) = provider.api_key {
                if !api_key.is_empty() {
                    env_vars.insert("DEEPSEEK_API_KEY".to_string(), api_key.clone());
                }
            }
        }

        // Get Alibaba Cloud API key
        if let Some(provider) = prefs.get_provider("alibaba_cloud") {
            if let Some(ref api_key) = provider.api_key {
                if !api_key.is_empty() {
                    env_vars.insert("DASHSCOPE_API_KEY".to_string(), api_key.clone());
                }
            }
        }

        env_vars
    }
}

impl MoFaFMScreenRef {
    /// Stop audio and dora timers - call this before hiding/removing the widget
    /// to prevent timer callbacks on inactive state
    /// Note: AEC blink animation is shader-driven and doesn't need stopping
    pub fn stop_timers(&self, cx: &mut Cx) {
        if let Some(inner) = self.borrow_mut() {
            cx.stop_timer(inner.audio_timer);
            cx.stop_timer(inner.dora_timer);
            ::log::debug!("MoFaFMScreen timers stopped");
        }
    }

    /// Restart audio and dora timers - call this when the widget becomes visible again
    /// Note: AEC blink animation is shader-driven and auto-resumes
    pub fn start_timers(&self, cx: &mut Cx) {
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
            inner.view.view(ids!(left_column.chat_container.chat_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to chat header and title
            inner.view.view(ids!(left_column.chat_container.chat_section.chat_header)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.chat_container.chat_section.chat_header.chat_title)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to audio control containers
            inner.view.view(ids!(left_column.audio_container.mic_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.audio_container.aec_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            inner.view.view(ids!(left_column.audio_container.device_container)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });

            // Apply dark mode to device labels
            inner.view.label(ids!(left_column.audio_container.device_container.device_selectors.input_device_group.input_device_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });
            inner.view.label(ids!(left_column.audio_container.device_container.device_selectors.output_device_group.output_device_label)).apply_over(cx, live!{
                draw_text: { dark_mode: (dark_mode) }
            });

            // NOTE: DropDown apply_over causes "target class not found" errors
            // TODO: Find alternative way to theme dropdowns

            // Apply dark mode to MofaHero
            inner.view.mofa_hero(ids!(left_column.mofa_hero)).update_dark_mode(cx, dark_mode);

            // Apply dark mode to participant panels
            inner.view.participant_panel(ids!(left_column.participant_container.participant_bar.student1_panel)).update_dark_mode(cx, dark_mode);
            inner.view.participant_panel(ids!(left_column.participant_container.participant_bar.student2_panel)).update_dark_mode(cx, dark_mode);
            inner.view.participant_panel(ids!(left_column.participant_container.participant_bar.tutor_panel)).update_dark_mode(cx, dark_mode);

            // Apply dark mode to prompt section
            inner.view.view(ids!(left_column.prompt_container.prompt_section)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
            });
            // NOTE: TextInput apply_over causes "target class not found" errors
            inner.view.button(ids!(left_column.prompt_container.prompt_section.prompt_row.button_group.reset_btn)).apply_over(cx, live!{
                draw_bg: { dark_mode: (dark_mode) }
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

            // Apply dark mode to log content Markdown
            // Use apply_over with font_color - this works because font_color is a top-level property
            if dark_mode > 0.5 {
                inner.view.markdown(ids!(log_section.log_content_column.log_scroll.log_content_wrapper.log_content))
                    .apply_over(cx, live!{ font_color: (vec4(0.796, 0.835, 0.882, 1.0)) }); // SLATE_300
            } else {
                inner.view.markdown(ids!(log_section.log_content_column.log_scroll.log_content_wrapper.log_content))
                    .apply_over(cx, live!{ font_color: (vec4(0.294, 0.333, 0.388, 1.0)) }); // GRAY_600
            }

            inner.view.redraw(cx);
        }
    }
}
