//! MoFA Cast UI Design - Makepad live_design DSL

use makepad_widgets::*;
use super::CastScreen;

live_design! {
    use link::theme::*;
    use link::shaders::*;
    use link::widgets::*;

    use mofa_widgets::theme::*;

    // Local layout constants
    SECTION_SPACING = 12.0
    PANEL_RADIUS = 4.0
    PANEL_PADDING = 12.0

    // Reusable panel header
    PanelHeader = <View> {
        width: Fill, height: Fit
        padding: {left: 12, right: 12, top: 8, bottom: 8}  // 更紧凑
        align: {y: 0.5}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((SLATE_50), (SLATE_800), self.dark_mode);
            }
        }
    }

    // MoFA Cast Screen
    pub CastScreen = {{CastScreen}} {
        width: Fill, height: Fill
        flow: Down
        spacing: (SECTION_SPACING)
        padding: { left: 16, right: 16, top: 12, bottom: 12 }  // 减少上部空间
        align: {y: 0.0}
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }

        // Header section
        header = <View> {
            width: Fill, height: Fit
            flow: Right
            spacing: 12  // 减少间距
            align: {y: 0.5}

            // 图标 - 使用emoji作为简单的麦克风图标
            icon_label = <Label> {
                text: "🎙️"
                draw_text: {
                    text_style: <FONT_BOLD>{ font_size: 28.0 }
                }
            }

            title_label = <Label> {
                text: "MoFA Cast"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_BOLD>{ font_size: 24.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                    }
                }
            }

            header_description = <Label> {
                text: "Transform chat transcripts into podcast audio"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 13.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                    }
                }
            }
        }

        // Main content area with horizontal layout
        main_content = <View> {
            width: Fill, height: Fill
            flow: Right
            spacing: (SECTION_SPACING)

            // Left panel - Import and controls
            left_panel = <View> {
                width: 200, height: Fill  // 缩短为原来的2/3
                flow: Down
                spacing: (SECTION_SPACING)

                // Import section
                import_section = <RoundedView> {
                    width: Fill, height: Fit
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }
                    flow: Down
                    padding: (PANEL_PADDING)

                    <PanelHeader> {
                        label = <Label> {
                            text: "Import Transcript"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }

                    format_dropdown = <DropDown> {
                        width: Fill
                        labels: ["Auto Detect", "Plain Text", "JSON", "Markdown"]
                        values: [0, 1, 2, 3]
                        draw_text: {
                            instance text_hover: 0.0
                            text_style: <FONT_MEDIUM>{ font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                // Blue text on hover, default text color otherwise
                                return mix((TEXT_PRIMARY), (BLUE_600), self.text_hover);
                            }
                        }
                        draw_bg: {
                            instance bg_hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                // Light blue background
                                let bg_color = #dbeafe;
                                sdf.fill(bg_color);
                                // Dark gray border
                                sdf.stroke(vec4(0.4, 0.4, 0.4, 1.0), 1.0);
                                return sdf.result;
                            }
                        }
                        popup_menu: {
                            draw_bg: {
                                color: (WHITE)
                                border_color: (BORDER)
                                border_size: 1.0
                                border_radius: 4.0
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
                    }

                    import_button = <Button> {
                        width: Fill
                        text: "Select File"
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 13.0 }
                            color: (WHITE)
                        }
                        draw_bg: {
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                sdf.fill(#3b82f6);
                                return sdf.result;
                            }
                        }
                    }

                    file_info = <Label> {
                        text: "No file selected"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_REGULAR>{ font_size: 11.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                            }
                        }
                    }
                }

                // Recent files section
                recent_files_section = <RoundedView> {
                    width: Fill, height: Fit
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }
                    flow: Down
                    padding: (PANEL_PADDING)

                    <PanelHeader> {
                        label = <Label> {
                            text: "Recent Files"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }

                    recent_files_list = <View> {
                        width: Fill, height: Fit
                        flow: Down
                        spacing: 4

                        // Placeholder text when empty
                        recent_files_placeholder = <Label> {
                            text: "No recent files"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }
                }

                // Speakers section
                speakers_section = <RoundedView> {
                    width: Fill, height: Fill
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }
                    flow: Down
                    padding: (PANEL_PADDING)

                    <PanelHeader> {
                        label = <Label> {
                            text: "Speakers"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }

                    speakers_list = <View> {
                        width: Fill, height: Fit
                        flow: Down
                        spacing: 8

                        placeholder = <Label> {
                            text: "Import a transcript to see speakers"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 12.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }
                }
            }

            // Right panel - Editor
            right_panel = <View> {
                width: Fill, height: Fill
                flow: Down
                spacing: (SECTION_SPACING)

                // Control buttons
                control_bar = <RoundedView> {
                    width: Fill, height: Fit
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }
                    flow: Right
                    padding: {top: 8, bottom: 8, left: 16, right: 16}
                    spacing: 8
                    align: {y: 0.5, x: 0.0}

                    open_editor_button = <Button> {
                        width: Fit, height: 28
                        text: "📝 Open in Editor"
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 13.0 }
                            color: (WHITE)
                        }
                        draw_bg: {
                            instance hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                let color = mix(#6366f1, #818cf8, self.hover);
                                sdf.fill(color);
                                return sdf.result;
                            }
                        }
                        animator: {
                            hover = {
                                default: off
                                off = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                                on = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                            }
                        }
                    }

                    synthesize_button = <Button> {
                        width: Fit, height: 28
                        text: "🎙️ Synthesize Audio"
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 13.0 }
                            color: (WHITE)
                        }
                        draw_bg: {
                            instance hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                let color = mix(#10b981, #34d399, self.hover);
                                sdf.fill(color);
                                return sdf.result;
                            }
                        }
                        animator: {
                            hover = {
                                default: off
                                off = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                                on = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                            }
                        }
                    }

                    export_button = <Button> {
                        width: Fit, height: 28
                        text: "📥 Export Audio"
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 13.0 }
                            color: (WHITE)
                        }
                        draw_bg: {
                            instance hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                let color = mix(#f59e0b, #fbbf24, self.hover);
                                sdf.fill(color);
                                return sdf.result;
                            }
                        }
                        animator: {
                            hover = {
                                default: off
                                off = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 0.0}} }
                                on = { from: {all: Forward {duration: 0.1}} apply: {draw_bg: {hover: 1.0}} }
                            }
                        }
                    }

                    // Format label
                    format_label = <Label> {
                        text: "Format:"
                        draw_text: {
                            instance dark_mode: 0.0
                            text_style: <FONT_MEDIUM>{ font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                            }
                        }
                    }

                    // Export format dropdown
                    export_format_dropdown = <DropDown> {
                        width: 70, height: 24
                        labels: ["WAV", "MP3"]
                        values: [0, 1]
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                            color: (TEXT_PRIMARY)
                        }
                        draw_bg: {
                            instance bg_hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                let bg_color = mix(#f9fafb, #f3f4f6, self.bg_hover);
                                sdf.fill(bg_color);
                                sdf.stroke((GRAY_300), 1.0);
                                return sdf.result;
                            }
                        }
                        popup_menu: {
                            draw_bg: {
                                color: (WHITE)
                                border_color: (BORDER)
                                border_size: 1.0
                                border_radius: 4.0
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
                    }

                    // MP3 bitrate dropdown (shown when MP3 format is selected)
                    mp3_bitrate_dropdown = <DropDown> {
                        width: 110, height: 24
                        labels: ["128 kbps", "192 kbps", "256 kbps", "320 kbps"]
                        values: [0, 1, 2, 3]
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                            color: (TEXT_PRIMARY)
                        }
                        draw_bg: {
                            instance bg_hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                let bg_color = mix(#e0f2fe, #bae6fd, self.bg_hover);  // Light blue for MP3
                                sdf.fill(bg_color);
                                sdf.stroke((BLUE_300), 1.0);
                                return sdf.result;
                            }
                        }
                        popup_menu: {
                            draw_bg: {
                                color: (WHITE)
                                border_color: (BORDER)
                                border_size: 1.0
                                border_radius: 4.0
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
                    }
                }

                // Content area with editor and player (horizontal layout)
                content_area = <View> {
                    width: Fill, height: Fill
                    flow: Right
                    spacing: 12

                    // Script editor area (left)
                    editor_container = <View> {
                        width: 500, height: Fill
                        flow: Down

                    script_panel = <RoundedView> {
                        width: Fill, height: Fill
                        show_bg: true
                        draw_bg: {
                            instance dark_mode: 0.0
                            border_radius: (PANEL_RADIUS)
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                                sdf.fill(bg);
                                return sdf.result;
                            }
                        }
                        flow: Down

                        <PanelHeader> {
                            label = <Label> {
                                text: "Podcast Script"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                            }
                            subtitle = <Label> {
                                text: "Import your optimized script (ChatGPT/Claude)"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                            }
                        }

                        script_editor = <TextInput> {
                            width: 500, height: 300
                            text: "Click 'Import Script' to load your optimized podcast script..."
                            padding: {left: 12, right: 12, top: 10, bottom: 10}
                            draw_bg: {
                                instance dark_mode: 0.0
                                fn pixel(self) -> vec4 {
                                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                    sdf.rect(0., 0., self.rect_size.x, self.rect_size.y);
                                    let bg = mix((WHITE), (SLATE_900), self.dark_mode);
                                    sdf.fill(bg);
                                    return sdf.result;
                                }
                            }
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 12.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                                word: Wrap  // Enable auto-wrap
                            }
                            draw_selection: {
                                color: (INDIGO_200)
                            }
                        }
                    }
                }

                    // Audio player section (right)
                    audio_player_section = <RoundedView> {
                        width: Fill, height: Fit
                        show_bg: true
                        draw_bg: {
                            instance dark_mode: 0.0
                            border_radius: (PANEL_RADIUS)
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                                let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                                sdf.fill(bg);
                                return sdf.result;
                            }
                        }
                        flow: Down
                        padding: (PANEL_PADDING)

                        <PanelHeader> {
                            label = <Label> {
                                text: "Audio Player"
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                            }
                        }

                        player_status = <Label> {
                            text: "No audio exported yet"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                }
                            }
                        }

                        // Playback controls
                        playback_controls = <View> {
                            width: Fill, height: Fit
                            flow: Right
                            spacing: 8
                            align: {y: 0.5}

                            play_button = <Button> {
                                width: 36, height: 36
                                text: "▶"
                                draw_text: {
                                    text_style: <FONT_BOLD>{ font_size: 16.0 }
                                    color: (WHITE)
                                }
                                draw_bg: {
                                    instance hover: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.circle(0.5 * self.rect_size.x, 0.5 * self.rect_size.y, 0.5 * self.rect_size.x);
                                        let color = mix(#10b981, #34d399, self.hover);
                                        sdf.fill(color);
                                        return sdf.result;
                                    }
                                }
                            }

                            stop_button = <Button> {
                                width: 36, height: 36
                                text: "⏹"
                                draw_text: {
                                    text_style: <FONT_BOLD>{ font_size: 16.0 }
                                    color: (WHITE)
                                }
                                draw_bg: {
                                    instance hover: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.circle(0.5 * self.rect_size.x, 0.5 * self.rect_size.y, 0.5 * self.rect_size.x);
                                        let color = mix(#ef4444, #f87171, self.hover);
                                        sdf.fill(color);
                                        return sdf.result;
                                    }
                                }
                            }

                            open_in_player_button = <Button> {
                                width: Fit, height: 32
                                text: "Open in Player"
                                draw_text: {
                                    text_style: <FONT_MEDIUM>{ font_size: 11.0 }
                                    color: (WHITE)
                                }
                                draw_bg: {
                                    instance hover: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                        let color = mix(#6366f1, #818cf8, self.hover);
                                        sdf.fill(color);
                                        return sdf.result;
                                    }
                                }
                            }
                        }

                        // Audio info
                        audio_info = <View> {
                            width: Fill, height: Fit
                            flow: Down
                            spacing: 4

                            format_label = <Label> {
                                text: ""
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_MEDIUM>{ font_size: 12.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                    }
                                }
                            }

                            duration_label = <Label> {
                                text: ""
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                            }

                            file_size_label = <Label> {
                                text: ""
                                draw_text: {
                                    instance dark_mode: 0.0
                                    text_style: <FONT_REGULAR>{ font_size: 11.0 }
                                    fn get_color(self) -> vec4 {
                                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                                    }
                                }
                            }
                        }
                    }
                }

                // Templates section
                templates_section = <RoundedView> {
                    width: 200, height: Fit
                    show_bg: true
                    draw_bg: {
                        instance dark_mode: 0.0
                        border_radius: (PANEL_RADIUS)
                        fn pixel(self) -> vec4 {
                            let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                            sdf.box(0., 0., self.rect_size.x, self.rect_size.y, self.border_radius);
                            let bg = mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                            sdf.fill(bg);
                            return sdf.result;
                        }
                    }
                    flow: Down
                    padding: (PANEL_PADDING)

                    <PanelHeader> {
                        label = <Label> {
                            text: "Templates"
                            draw_text: {
                                instance dark_mode: 0.0
                                text_style: <FONT_SEMIBOLD>{ font_size: 14.0 }
                                fn get_color(self) -> vec4 {
                                    return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                                }
                            }
                        }
                    }

                    template_dropdown = <DropDown> {
                        width: Fill
                        labels: ["2-Person Interview", "3-Person Discussion", "Narrative"]
                        values: [0, 1, 2]
                        draw_text: {
                            instance text_hover: 0.0
                            text_style: <FONT_MEDIUM>{ font_size: 12.0 }
                            fn get_color(self) -> vec4 {
                                // Blue text on hover, default text color otherwise
                                return mix((TEXT_PRIMARY), (BLUE_600), self.text_hover);
                            }
                        }
                        draw_bg: {
                            instance bg_hover: 0.0
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                // Light blue background
                                let bg_color = #dbeafe;
                                sdf.fill(bg_color);
                                // Dark gray border
                                sdf.stroke(vec4(0.4, 0.4, 0.4, 1.0), 1.0);
                                return sdf.result;
                            }
                        }
                        popup_menu: {
                            draw_bg: {
                                color: (WHITE)
                                border_color: (BORDER)
                                border_size: 1.0
                                border_radius: 4.0
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
                    }

                    use_template_button = <Button> {
                        width: Fill
                        text: "Use Template"
                        draw_text: {
                            text_style: <FONT_MEDIUM>{ font_size: 13.0 }
                            color: (WHITE)
                        }
                        draw_bg: {
                            fn pixel(self) -> vec4 {
                                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                sdf.fill(#8b5cf6);
                                return sdf.result;
                            }
                        }
                    }
                }
            }

            // System Log panel - adaptive width, collapsible
            log_section = <View> {
                width: 320, height: Fill
                flow: Right
                align: {y: 0.0}
                visible: true

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
                        text: "<"
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
                                labels: ["ALL", "INFO", "WARN", "ERROR"]
                                values: [0, 1, 2, 3]
                                draw_text: {
                                    text_style: <FONT_MEDIUM>{ font_size: 10.0 }
                                    color: (TEXT_PRIMARY)
                                }
                                draw_bg: {
                                    instance hover: 0.0
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                        // 保持浅灰色背景，hover 时稍微深一点
                                        let bg_color = mix(#f9fafb, #f3f4f6, self.hover);
                                        sdf.fill(bg_color);
                                        sdf.stroke((GRAY_300), 1.0);
                                        return sdf.result;
                                    }
                                }
                                popup_menu: {
                                    draw_bg: {
                                        color: (WHITE)
                                        border_color: (BORDER)
                                        border_size: 1.0
                                        border_radius: 4.0
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
                            }

                            // Clear button
                            clear_log_btn = <Button> {
                                width: 60, height: 24
                                text: "Clear"
                                draw_text: {
                                    text_style: <FONT_MEDIUM>{ font_size: 10.0 }
                                    color: (TEXT_PRIMARY)
                                }
                                draw_bg: {
                                    fn pixel(self) -> vec4 {
                                        let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                                        sdf.box(0., 0., self.rect_size.x, self.rect_size.y, 4.0);
                                        sdf.fill((GRAY_200));
                                        return sdf.result;
                                    }
                                }
                            }
                        }
                    }

                    // Log content area
                    log_scroll = <ScrollYView> {
                        width: Fill, height: Fill
                        flow: Down
                        scroll_bars: <ScrollBars> {
                            show_scroll_x: false
                            show_scroll_y: true
                        }

                        log_content_wrapper = <View> {
                            width: Fill, height: Fit
                            padding: {left: 12, right: 12, top: 8, bottom: 8}
                            flow: Down

                            log_content = <Markdown> {
                                width: Fill,
                                font_size: 10.0
                                font_color: (GRAY_700)
                                paragraph_spacing: 4
                            }
                        }
                    }
                }
            }
        }
    }

    // Rust field initializations
    selected_format_id: 0  // Default to Auto Detect
    selected_template_id: 0  // Default to first template
    selected_export_format: 0  // Default to WAV
    selected_mp3_bitrate: 1  // Default to 192 kbps (recommended)
}
