//! RSS Newscaster Screen

use crossbeam_channel::{unbounded, Receiver};
use makepad_widgets::*;
use std::thread;
use std::time::Duration;

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

    pub RSSNewscasterScreen = {{RSSNewscasterScreen}} {
        width: Fill, height: Fill
        flow: Down
        padding: 24
        spacing: 20
        show_bg: true
        draw_bg: {
            instance dark_mode: 0.0
            fn pixel(self) -> vec4 {
                return mix((DARK_BG), (DARK_BG_DARK), self.dark_mode);
            }
        }

        // Header
        header = <View> {
            width: Fill, height: Fit
            flow: Down
            spacing: 8

            title = <Label> {
                text: "RSS Newscaster"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_BOLD>{ font_size: 28.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                    }
                }
            }

            description = <Label> {
                text: "Convert RSS feeds into multi-anchor news broadcast scripts"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 14.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                    }
                }
            }
        }

        // Content Card
        content_card = <RoundedView> {
            width: Fill, height: Fit
            padding: 24
            flow: Down
            spacing: 16
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                border_radius: 8.0
                fn get_color(self) -> vec4 {
                    return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                }
            }

            // Input Section
            input_section = <View> {
                width: Fill, height: Fit
                flow: Down
                spacing: 12

                input_label = <Label> {
                    text: "RSS Feed URL"
                    draw_text: {
                        instance dark_mode: 0.0
                        text_style: <FONT_REGULAR>{ font_size: 14.0 }
                        fn get_color(self) -> vec4 {
                            return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                        }
                    }
                }

                rss_url_input = <TextInput> {
                    width: Fill, height: 40
                    empty_message: "Enter RSS feed URL (e.g., https://news.ycombinator.com/rss)"
                }
            }

            // Action Section
            action_section = <View> {
                width: Fill, height: Fit
                flow: Right
                spacing: 12
                align: {x: 0.0, y: 0.5}

                start_button = <Button> {
                    text: "Generate News Script"
                    width: Fit, height: 40
                }

                status_label = <Label> {
                    text: "Ready"
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

        // Results Card
        results_card = <RoundedView> {
            width: Fill, height: Fill
            padding: 24
            flow: Down
            spacing: 12
            show_bg: true
            draw_bg: {
                instance dark_mode: 0.0
                border_radius: 8.0
                fn get_color(self) -> vec4 {
                    return mix((PANEL_BG), (PANEL_BG_DARK), self.dark_mode);
                }
            }

            results_title = <Label> {
                text: "Generated News Script"
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_BOLD>{ font_size: 16.0 }
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_PRIMARY), (TEXT_PRIMARY_DARK), self.dark_mode);
                    }
                }
            }

            results_text = <Label> {
                width: Fill, height: Fill
                text: "Click 'Generate News Script' to start.\n\nThe dataflow will:\n1. Fetch RSS feed from the provided URL\n2. Extract article content from each link\n3. Generate news broadcast scripts using AI\n4. Assign scripts to 张明 (Male Anchor) and 李华 (Female Anchor)\n\nResults will appear here."
                draw_text: {
                    instance dark_mode: 0.0
                    text_style: <FONT_REGULAR>{ font_size: 14.0 }
                    wrap: Word
                    fn get_color(self) -> vec4 {
                        return mix((TEXT_SECONDARY), (TEXT_SECONDARY_DARK), self.dark_mode);
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct RSSNewscasterScreen {
    #[deref]
    view: View,
    #[rust]
    fetch_rx: Option<Receiver<FetchOutcome>>,
    #[rust]
    poll_timer: Timer,
    #[rust]
    request_id: u64,
    #[rust]
    last_results: String,
}

impl Widget for RSSNewscasterScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);

        if self.poll_timer.is_event(event).is_some() {
            self.poll_fetch(cx);
        }

        let actions = match event {
            Event::Actions(actions) => actions.as_slice(),
            _ => &[],
        };

        if self
            .view
            .button(ids!(content_card.action_section.start_button))
            .clicked(actions)
        {
            self.handle_generate(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}

impl RSSNewscasterScreen {
    fn handle_generate(&mut self, cx: &mut Cx) {
        let raw_url = self
            .view
            .text_input(ids!(content_card.input_section.rss_url_input))
            .text();
        let url = raw_url.trim().to_string();

        if url.is_empty() {
            self.set_status(cx, "Please enter an RSS URL");
            self.set_results(cx, "Enter a valid RSS feed URL to start.");
            self.view.redraw(cx);
            return;
        }

        self.set_status(cx, "Generating...");
        self.set_results(cx, &build_mock_result(&url));
        self.start_fetch(cx, url);
        self.view.redraw(cx);
    }

    fn start_fetch(&mut self, cx: &mut Cx, url: String) {
        let (tx, rx) = unbounded();
        self.fetch_rx = Some(rx);
        self.request_id = self.request_id.wrapping_add(1);
        let request_id = self.request_id;

        self.poll_timer = cx.start_interval(0.1);

        thread::spawn(move || {
            let outcome = match fetch_and_render(&url) {
                Ok(rendered) => FetchOutcome::Success {
                    request_id,
                    rendered,
                },
                Err(message) => FetchOutcome::Error {
                    request_id,
                    message,
                },
            };
            let _ = tx.send(outcome);
        });
    }

    fn poll_fetch(&mut self, cx: &mut Cx) {
        let rx = match self.fetch_rx.as_ref() {
            Some(rx) => rx.clone(),
            None => return,
        };

        while let Ok(outcome) = rx.try_recv() {
            if outcome.request_id() != self.request_id {
                continue;
            }

            match outcome {
                FetchOutcome::Success { rendered, .. } => {
                    self.set_results(cx, &rendered);
                    self.set_status(cx, "Completed");
                }
                FetchOutcome::Error { message, .. } => {
                    let combined = if self.last_results.is_empty() {
                        format!("Fetch failed: {}", message)
                    } else {
                        format!("{}\n\nFetch failed: {}", self.last_results, message)
                    };
                    self.set_results(cx, &combined);
                    self.set_status(cx, "Error");
                }
            }

            self.view.redraw(cx);
        }
    }

    fn set_status(&mut self, cx: &mut Cx, text: &str) {
        self.view
            .label(ids!(content_card.action_section.status_label))
            .set_text(cx, text);
    }

    fn set_results(&mut self, cx: &mut Cx, text: &str) {
        self.view
            .label(ids!(results_card.results_text))
            .set_text(cx, text);
        self.last_results = text.to_string();
    }
}

#[derive(Debug)]
enum FetchOutcome {
    Success { request_id: u64, rendered: String },
    Error { request_id: u64, message: String },
}

impl FetchOutcome {
    fn request_id(&self) -> u64 {
        match self {
            FetchOutcome::Success { request_id, .. } => *request_id,
            FetchOutcome::Error { request_id, .. } => *request_id,
        }
    }
}

fn build_mock_result(url: &str) -> String {
    let source = extract_source(url);
    let mut lines = vec![
        format!("Feed: {} Headlines", source),
        format!("URL: {}", url),
        "Items: 3".to_string(),
        String::new(),
        "1. 李华 (女主播)".to_string(),
        format!("Title: {} morning brief", source),
        format!(
            "Script: 李华：{} morning brief。我们先快速浏览三条焦点更新。",
            source
        ),
        String::new(),
        "2. 张明 (男主播)".to_string(),
        "Title: Markets react to overnight developments".to_string(),
        "Script: 张明：市场对最新动态迅速反应，投资者关注后续信号。".to_string(),
        String::new(),
        "3. 李华 (女主播)".to_string(),
        "Title: Tech leaders highlight upcoming launches".to_string(),
        "Script: 李华：科技行业预告新一轮发布，产品与AI议题成为焦点。".to_string(),
    ];

    while lines.last().map(|line| line.is_empty()).unwrap_or(false) {
        lines.pop();
    }
    lines.join("\n")
}

fn extract_source(url: &str) -> String {
    let trimmed = url.trim();
    let host = trimmed
        .split("//")
        .nth(1)
        .and_then(|rest| rest.split('/').next())
        .unwrap_or("the feed");
    host.trim_start_matches("www.").to_string()
}

fn fetch_and_render(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("MoFA RSS Newscaster/0.1")
        .build()
        .map_err(|e| format!("Client error: {}", e))?;

    let response = client
        .get(url)
        .send()
        .map_err(|e| format!("Request failed: {}", e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status));
    }
    let bytes = response
        .bytes()
        .map_err(|e| format!("Read failed: {}", e))?;

    let feed = feed_rs::parser::parse(std::io::Cursor::new(bytes))
        .map_err(|e| format!("Parse failed: {}", e))?;

    let feed_title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let max_items = 6usize;
    let anchors = [("李华", "女主播"), ("张明", "男主播")];
    let mut lines = Vec::new();

    lines.push(format!("Feed: {}", feed_title));
    lines.push(format!("URL: {}", url));
    lines.push(format!(
        "Items: {}",
        std::cmp::min(feed.entries.len(), max_items)
    ));
    lines.push(String::new());

    for (index, entry) in feed.entries.into_iter().take(max_items).enumerate() {
        let (anchor_name, anchor_role) = anchors[index % anchors.len()];
        let title = entry
            .title
            .map(|t| t.content)
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_string());
        let raw_summary = entry
            .summary
            .map(|s| s.content)
            .or_else(|| entry.content.and_then(|c| c.body))
            .unwrap_or_default();
        let summary = normalize_text(&raw_summary);
        let snippet = truncate_chars(&summary, 220);
        let script = if snippet.is_empty() {
            format!("{}：{}。", anchor_name, title)
        } else {
            format!("{}：{}。{}", anchor_name, title, snippet)
        };

        lines.push(format!("{}. {} ({})", index + 1, anchor_name, anchor_role));
        lines.push(format!("Title: {}", title));
        lines.push(format!("Script: {}", script));
        lines.push(String::new());
    }

    while lines.last().map(|line| line.is_empty()).unwrap_or(false) {
        lines.pop();
    }

    Ok(lines.join("\n"))
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(text: &str, max_len: usize) -> String {
    let mut result = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_len {
            result.push_str("...");
            break;
        }
        result.push(ch);
    }
    result
}

impl RSSNewscasterScreenRef {
    pub fn update_dark_mode(&self, cx: &mut Cx, dark_mode: f64) {
        if let Some(mut inner) = self.borrow_mut() {
            // Update background
            inner.view.apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );

            // Update header labels
            inner.view.label(ids!(header.title)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(header.description)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            // Update content card
            inner.view.view(ids!(content_card)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(content_card.input_section.input_label)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(content_card.action_section.status_label)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            // Update results card
            inner.view.view(ids!(results_card)).apply_over(
                cx,
                live! {
                    draw_bg: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(results_card.results_title)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );
            inner.view.label(ids!(results_card.results_text)).apply_over(
                cx,
                live! {
                    draw_text: { dark_mode: (dark_mode) }
                },
            );

            inner.view.redraw(cx);
        }
    }
}
