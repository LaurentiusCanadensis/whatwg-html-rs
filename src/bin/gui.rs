//! JustHTML GUI - An Iced-based HTML parser and sanitizer viewer.

use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_editor, text_input};
use iced::{Element, Length, Task, Theme};
use justhtml::{parse, sanitize::{sanitize_dom, DEFAULT_POLICY}, selector::query_all, serialize::serialize_to_html, NodeKind};

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("JustHTML")
        .window_size((1000.0, 800.0))
        .theme(theme)
        .centered()
        .run()
}

fn theme(_app: &App) -> Theme {
    Theme::Dark
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum OutputView {
    #[default]
    Sanitized,
    PlainText,
    RawHtml,
    Selector,
}

impl std::fmt::Display for OutputView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputView::Sanitized => write!(f, "Sanitized HTML"),
            OutputView::PlainText => write!(f, "Plain Text"),
            OutputView::RawHtml => write!(f, "Raw HTML"),
            OutputView::Selector => write!(f, "CSS Selector"),
        }
    }
}

const OUTPUT_VIEWS: [OutputView; 4] = [
    OutputView::Sanitized,
    OutputView::PlainText,
    OutputView::RawHtml,
    OutputView::Selector,
];

struct App {
    html_input: text_editor::Content,
    selector_input: String,
    output: String,
    selected_view: OutputView,
    error_message: Option<String>,
    parse_time_ms: Option<f64>,
}

impl Default for App {
    fn default() -> Self {
        let default_html = r#"<!DOCTYPE html>
<html>
<head><title>Example</title></head>
<body>
  <h1>Hello, World!</h1>
  <p>This is a <b>sample</b> HTML document.</p>
  <script>alert('This will be removed')</script>
  <a href="https://example.com">Safe link</a>
  <a href="javascript:alert('XSS')">Dangerous link</a>
</body>
</html>"#;

        Self {
            html_input: text_editor::Content::with_text(default_html),
            selector_input: String::from("p"),
            output: String::new(),
            selected_view: OutputView::Sanitized,
            error_message: None,
            parse_time_ms: None,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    HtmlInputChanged(text_editor::Action),
    SelectorChanged(String),
    ViewSelected(OutputView),
    Parse,
}

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::HtmlInputChanged(action) => {
                self.html_input.perform(action);
            }
            Message::SelectorChanged(selector) => {
                self.selector_input = selector;
            }
            Message::ViewSelected(view) => {
                self.selected_view = view;
                self.process_html();
            }
            Message::Parse => {
                self.process_html();
            }
        }
        Task::none()
    }

    fn process_html(&mut self) {
        let html = self.html_input.text();
        self.error_message = None;

        let start = std::time::Instant::now();

        match self.selected_view {
            OutputView::Sanitized => {
                let mut result = parse(&html);
                let _ = sanitize_dom(&mut result.dom, result.document, &DEFAULT_POLICY);
                self.output = serialize_to_html(&result.dom, result.document);
            }
            OutputView::PlainText => {
                let result = parse(&html);
                self.output = extract_text(&result.dom, result.document);
            }
            OutputView::RawHtml => {
                let result = parse(&html);
                self.output = result.to_html();
            }
            OutputView::Selector => {
                let result = parse(&html);
                match query_all(&result.dom, result.document, &self.selector_input) {
                    Ok(nodes) => {
                        let mut output = format!("Found {} matches for '{}'\n\n", nodes.len(), self.selector_input);
                        for (i, node_id) in nodes.iter().enumerate() {
                            let node_html = serialize_to_html(&result.dom, *node_id);
                            output.push_str(&format!("--- Match {} ---\n{}\n\n", i + 1, node_html));
                        }
                        self.output = output;
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Selector error: {}", e));
                        self.output = String::new();
                    }
                }
            }
        }

        self.parse_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
    }

    fn view(&self) -> Element<Message> {
        let title = text("JustHTML Viewer").size(24);

        let html_editor = text_editor(&self.html_input)
            .on_action(Message::HtmlInputChanged)
            .height(Length::FillPortion(1));

        let parse_button = button(text("Parse")).on_press(Message::Parse);

        let view_picker = pick_list(
            OUTPUT_VIEWS.as_slice(),
            Some(self.selected_view),
            Message::ViewSelected,
        );

        let timing = if let Some(ms) = self.parse_time_ms {
            text(format!("{:.2} ms", ms)).size(12)
        } else {
            text("").size(12)
        };

        let controls = row![parse_button, view_picker, timing]
            .spacing(10)
            .align_y(iced::Alignment::Center);

        let selector_row = if self.selected_view == OutputView::Selector {
            Some(
                row![
                    text("Selector:").size(14),
                    text_input("CSS selector...", &self.selector_input)
                        .on_input(Message::SelectorChanged)
                        .on_submit(Message::Parse)
                        .width(Length::Fill),
                ]
                .spacing(10)
                .align_y(iced::Alignment::Center),
            )
        } else {
            None
        };

        let error_text = if let Some(ref err) = self.error_message {
            Some(text(err).size(14).color([1.0, 0.4, 0.4]))
        } else {
            None
        };

        let output_display = scrollable(
            text(&self.output)
                .size(13)
                .font(iced::Font::MONOSPACE)
        )
        .height(Length::FillPortion(1));

        let mut content = column![title, html_editor, controls,].spacing(10);

        if let Some(selector_row) = selector_row {
            content = content.push(selector_row);
        }

        if let Some(error_text) = error_text {
            content = content.push(error_text);
        }

        content = content.push(output_display);

        container(content)
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

/// Extract plain text from DOM, recursively traversing nodes.
fn extract_text(dom: &justhtml::Dom, node_id: justhtml::NodeId) -> String {
    let mut result = String::new();
    extract_text_recursive(dom, node_id, &mut result);
    result
}

fn extract_text_recursive(dom: &justhtml::Dom, node_id: justhtml::NodeId, result: &mut String) {
    let node = dom.get(node_id);

    match &node.kind {
        NodeKind::Text(txt) => {
            result.push_str(txt);
        }
        NodeKind::Element(el) => {
            // Skip script and style content
            let tag = el.name.to_lowercase();
            if tag == "script" || tag == "style" {
                return;
            }
            // Add newlines for block elements
            let is_block = matches!(tag.as_str(),
                "div" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                "ul" | "ol" | "li" | "br" | "hr" | "table" | "tr" | "td" | "th" |
                "article" | "section" | "header" | "footer" | "nav" | "aside" | "main"
            );

            if is_block && !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }

            // Recurse into children
            let mut child = node.first_child;
            while let Some(child_id) = child {
                extract_text_recursive(dom, child_id, result);
                child = dom.get(child_id).next_sibling;
            }

            if is_block && !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
        }
        NodeKind::Document => {
            let mut child = node.first_child;
            while let Some(child_id) = child {
                extract_text_recursive(dom, child_id, result);
                child = dom.get(child_id).next_sibling;
            }
        }
        _ => {}
    }
}
