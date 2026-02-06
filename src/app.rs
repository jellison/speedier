use crate::calc::{prepend_ans_if_leading_operator, Entry, Evaluator, History};
use crate::prefs::Preferences;
use crate::syntax::{tokenize, TokenKind};
use crate::theme;
use gpui::prelude::*;
use gpui::{div, px, App, Context, Entity, Hsla, ScrollHandle, Subscription, Window};
use gpui_component::input::{Input, InputEvent, InputState, Position};
use gpui_component::scroll::ScrollableElement;
use gpui_component::Size as UiSize;
use gpui_component::{h_flex, v_flex};
use gpui_component::{Icon, IconName, Sizable};

const ENABLE_DIGIT_GROUPING: bool = true;
const DIGIT_GROUP_PADDING_PX: f32 = 3.0;
const INPUT_GROUP_SEPARATOR: char = '\u{2009}';

pub struct SpeedierApp {
    input: Entity<InputState>,
    input_value: String,
    evaluator: Evaluator,
    history: History,
    error: Option<String>,
    reference_visible: bool,
    window_size: Option<(f32, f32)>,
    pending_clear: bool,
    pending_focus: bool,
    pending_scroll_to_bottom: bool,
    pending_input_reformat: Option<PendingInputReformat>,
    history_scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

struct PendingInputReformat {
    value: String,
    cursor_character: u32,
}

impl SpeedierApp {
    pub fn new(
        input: Entity<InputState>,
        prefs: Preferences,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let window_size = prefs.window_size();
        let reference_visible = prefs.reference_visible.unwrap_or(true);
        let history_entries = prefs.history;

        let mut history = History::new(100);
        history.load_entries(history_entries);

        let last_result = prefs
            .last_result
            .or_else(|| last_result_from_history(&history))
            .unwrap_or(0.0);

        let _subscriptions =
            vec![
                cx.subscribe(&input, |this: &mut Self, _input, ev, cx| match ev {
                    InputEvent::Change => {
                        let (displayed, display_cursor) = {
                            let input = this.input.read(cx);
                            (
                                input.value().to_string(),
                                input.cursor_position().character as usize,
                            )
                        };

                        let canonical = strip_group_separators(&displayed);
                        let canonical_cursor =
                            canonical_cursor_from_display_cursor(&displayed, display_cursor);

                        this.input_value = canonical.clone();

                        if ENABLE_DIGIT_GROUPING {
                            let formatted = format_expression_for_input(&canonical);
                            if formatted != displayed {
                                this.pending_input_reformat = Some(PendingInputReformat {
                                    cursor_character: display_cursor_from_canonical_cursor(
                                        &formatted,
                                        canonical_cursor,
                                    ) as u32,
                                    value: formatted,
                                });
                                cx.notify();
                            }
                        }
                    }
                    InputEvent::PressEnter { .. } => {
                        this.submit();
                        cx.notify();
                    }
                    _ => {}
                }),
            ];

        input.update(cx, |state, cx| {
            state.set_placeholder("Type an expression", window, cx);
        });

        let has_history = history.len() > 0;

        Self {
            input,
            input_value: String::new(),
            evaluator: Evaluator::with_last(last_result),
            history,
            error: None,
            reference_visible,
            window_size,
            pending_clear: false,
            pending_focus: true,
            pending_scroll_to_bottom: has_history,
            pending_input_reformat: None,
            history_scroll: ScrollHandle::new(),
            _subscriptions,
        }
    }

    fn submit(&mut self) {
        let trimmed = self.input_value.trim();
        if trimmed.is_empty() {
            return;
        }

        let normalized = prepend_ans_if_leading_operator(trimmed);
        let display_expr = if normalized != trimmed {
            normalized.clone()
        } else {
            trimmed.to_string()
        };

        match self.evaluator.eval(&normalized) {
            Ok(result) => {
                let formatted = format_result(result);
                self.error = None;
                self.history.add(&display_expr, &formatted, None);
                self.pending_scroll_to_bottom = true;
            }
            Err(err) => {
                self.error = Some(format!("Error: {}", err));
            }
        }

        self.input_value.clear();
        self.pending_input_reformat = None;
        self.pending_clear = true;
        self.save_prefs();
    }

    fn save_prefs(&self) {
        let prefs = Preferences {
            window_width: self.window_size.map(|v| v.0),
            window_height: self.window_size.map(|v| v.1),
            reference_visible: Some(self.reference_visible),
            last_result: Some(self.evaluator.last_result()),
            history: self.history.to_vec(),
        };
        let _ = prefs.save();
    }

    fn update_window_size(&mut self, window: &Window) {
        let bounds = window.bounds();
        let size = bounds.size;
        let next = (f32::from(size.width), f32::from(size.height));
        if self.window_size != Some(next) {
            self.window_size = Some(next);
            self.save_prefs();
        }
    }

    fn history_row(&self, entry: &Entry) -> gpui::Div {
        let expr_tokens = tokenize(&entry.expression);
        let expr_line = tokens_to_line(&expr_tokens);

        let result_line = if let Some(err) = &entry.err {
            div().text_color(theme::error()).child(format!("✕ {}", err))
        } else {
            let mut result_tokens = vec![crate::syntax::Token {
                kind: TokenKind::Operator,
                text: "= ".to_string(),
            }];
            result_tokens.extend(tokenize(&entry.result));
            tokens_to_line(&result_tokens)
        };

        v_flex().gap(px(2.0)).child(expr_line).child(result_line)
    }

    fn reference_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement + '_ {
        let close_button = icon_button(
            "reference-close",
            IconName::Close,
            cx.listener(|this, _, _window, cx| {
                this.reference_visible = false;
                this.save_prefs();
                cx.notify();
            }),
        );

        let header = h_flex()
            .justify_between()
            .items_center()
            .child(div().text_color(theme::fg()).child("Reference"))
            .child(close_button);

        let content = v_flex()
            .gap(px(6.0))
            .child(section_label("Operators"))
            .child(text_label("x! - factorial (non-negative integers)"))
            .child(div().h(px(1.0)).bg(theme::separator()))
            .child(section_label("Functions"))
            .child(text_label("sin(x) - sine (radians)"))
            .child(text_label("cos(x) - cosine (radians)"))
            .child(text_label("tan(x) - tangent (radians)"))
            .child(text_label("sqrt(x) - square root"))
            .child(text_label("pow(x, y) - power"))
            .child(text_label("log(x) - base-10 log"))
            .child(text_label("ln(x) - natural log"))
            .child(text_label("abs(x) - absolute value"))
            .child(text_label("ceil(x) - round up"))
            .child(text_label("floor(x) - round down"))
            .child(div().h(px(1.0)).bg(theme::separator()))
            .child(section_label("Constants"))
            .child(text_label("pi - 3.14159..."))
            .child(text_label("e - 2.71828..."))
            .child(div().h(px(1.0)).bg(theme::separator()))
            .child(section_label("Vars"))
            .child(text_label("ans - last result"));

        v_flex()
            .gap(px(12.0))
            .p(px(16.0))
            .bg(theme::panel())
            .h_full()
            .child(header)
            .child(div().flex_1().overflow_y_scrollbar().child(content))
    }
}

fn last_result_from_history(history: &History) -> Option<f64> {
    history.entries().iter().rev().find_map(|entry| {
        if entry.err.is_some() {
            return None;
        }
        entry.result.parse::<f64>().ok()
    })
}

impl Render for SpeedierApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.update_window_size(window);

        if self.pending_clear {
            self.input.update(cx, |state, cx| {
                state.set_value("", window, cx);
            });
            self.pending_clear = false;
        }

        if let Some(reformat) = self.pending_input_reformat.take() {
            self.input.update(cx, |state, cx| {
                state.set_value(reformat.value, window, cx);
                state.set_cursor_position(Position::new(0, reformat.cursor_character), window, cx);
            });
        }

        if self.pending_focus {
            self.input.update(cx, |state, cx| {
                state.focus(window, cx);
            });
            self.pending_focus = false;
        }

        if self.pending_scroll_to_bottom {
            self.history_scroll.scroll_to_bottom();
            self.pending_scroll_to_bottom = false;
        }

        let input = Input::new(&self.input);

        let error_line = match &self.error {
            Some(msg) => div().text_color(theme::error()).child(msg.clone()),
            None => div(),
        };

        let mut history_list = v_flex()
            .id("history-list")
            .gap(px(12.0))
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .track_scroll(&self.history_scroll);

        for (idx, entry) in self.history.entries().iter().enumerate() {
            history_list = history_list.child(self.history_row(entry));
            if idx + 1 < self.history.len() {
                history_list = history_list.child(
                    div()
                        .w_full()
                        .px(px(10.0))
                        .child(div().w_full().h(px(1.0)).bg(theme::separator())),
                );
            }
        }

        let history_scroll = div()
            .id("history-scroll")
            .relative()
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .child(history_list)
            .vertical_scrollbar(&self.history_scroll);

        let results_area = v_flex()
            .gap(px(10.0))
            .p(px(16.0))
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .child(error_line)
            .child(history_scroll);

        let input_area = div()
            .w_full()
            .px(px(16.0))
            .pt(px(10.0))
            .pb(px(16.0))
            .child(div().bg(theme::bg()).p(px(10.0)).child(input));

        let main_content = v_flex()
            .bg(theme::bg())
            .flex_1()
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .child(results_area)
            .child(div().w_full().h(px(1.0)).bg(theme::separator()))
            .child(input_area);

        let main = if self.reference_visible {
            main_content
        } else {
            let open_button = icon_button(
                "reference-open",
                IconName::BookOpen,
                cx.listener(|this, _, _window, cx| {
                    this.reference_visible = true;
                    this.save_prefs();
                    cx.notify();
                }),
            );
            div().size_full().child(main_content).child(
                div()
                    .absolute()
                    .top(px(18.0))
                    .right(px(18.0))
                    .child(open_button),
            )
        };

        if self.reference_visible {
            h_flex()
                .gap(px(0.0))
                .w_full()
                .h_full()
                .child(main)
                .child(div().w(px(1.0)).bg(theme::separator()))
                .child(div().w(px(260.0)).h_full().child(self.reference_panel(cx)))
        } else {
            main
        }
    }
}

fn format_result(val: f64) -> String {
    if val.abs() < 1e-14 {
        return "0".to_string();
    }
    let mut out = format!("{:.15}", val);
    if out.contains('.') {
        while out.ends_with('0') {
            out.pop();
        }
        if out.ends_with('.') {
            out.pop();
        }
    }
    out
}

fn tokens_to_line(tokens: &[crate::syntax::Token]) -> gpui::Div {
    let mut line = h_flex().gap(px(0.0));
    for token in tokens {
        let color = match token.kind {
            TokenKind::Number => theme::syntax_number(),
            TokenKind::Operator => theme::syntax_operator(),
            TokenKind::Function => theme::syntax_function(),
            TokenKind::Constant => theme::syntax_constant(),
            TokenKind::Paren | TokenKind::Comma => theme::syntax_paren(),
            TokenKind::Identifier => theme::syntax_ident(),
            TokenKind::Whitespace => theme::syntax_dim(),
            TokenKind::Unknown => theme::fg(),
        };
        if ENABLE_DIGIT_GROUPING && token.kind == TokenKind::Number {
            line = line.child(render_number_token(&token.text, color));
        } else {
            line = line.child(div().text_color(color).child(token.text.clone()));
        }
    }
    line
}

#[derive(Debug)]
struct NumberSegment {
    text: String,
    pad_before: bool,
}

fn render_number_token(number: &str, color: Hsla) -> gpui::Div {
    let mut line = h_flex().gap(px(0.0));
    for segment in split_number_segments(number) {
        let mut part = div().text_color(color).child(segment.text);
        if segment.pad_before {
            part = part.pl(px(DIGIT_GROUP_PADDING_PX));
        }
        line = line.child(part);
    }
    line
}

fn split_number_segments(number: &str) -> Vec<NumberSegment> {
    let (mantissa, exponent) = split_exponent(number);
    let (sign, unsigned_mantissa) = split_leading_sign(mantissa);
    let (int_part, frac_part, has_dot) = split_decimal_parts(unsigned_mantissa);
    let mut segments = Vec::new();

    if let Some(sign) = sign {
        segments.push(NumberSegment {
            text: sign.to_string(),
            pad_before: false,
        });
    }

    push_grouped_integer_segments(&mut segments, int_part);

    if has_dot {
        segments.push(NumberSegment {
            text: ".".to_string(),
            pad_before: false,
        });
    }

    push_grouped_fraction_segments(&mut segments, frac_part);

    if let Some(exponent) = exponent {
        segments.push(NumberSegment {
            text: exponent.to_string(),
            pad_before: false,
        });
    }

    if segments.is_empty() {
        segments.push(NumberSegment {
            text: number.to_string(),
            pad_before: false,
        });
    }

    segments
}

fn format_expression_for_input(expr: &str) -> String {
    tokenize(expr)
        .into_iter()
        .map(|token| {
            if token.kind == TokenKind::Number {
                join_number_segments_with_separator(&token.text, INPUT_GROUP_SEPARATOR)
            } else {
                token.text
            }
        })
        .collect()
}

fn strip_group_separators(text: &str) -> String {
    text.chars()
        .filter(|ch| *ch != INPUT_GROUP_SEPARATOR)
        .collect()
}

fn canonical_cursor_from_display_cursor(display: &str, display_cursor: usize) -> usize {
    display
        .chars()
        .take(display_cursor)
        .filter(|ch| *ch != INPUT_GROUP_SEPARATOR)
        .count()
}

fn display_cursor_from_canonical_cursor(display: &str, canonical_cursor: usize) -> usize {
    let mut canonical_count = 0usize;
    for (display_ix, ch) in display.chars().enumerate() {
        if canonical_count >= canonical_cursor {
            return display_ix;
        }
        if ch != INPUT_GROUP_SEPARATOR {
            canonical_count += 1;
        }
    }
    display.chars().count()
}

fn join_number_segments_with_separator(number: &str, separator: char) -> String {
    let mut out = String::new();
    for segment in split_number_segments(number) {
        if segment.pad_before {
            out.push(separator);
        }
        out.push_str(&segment.text);
    }
    out
}

fn split_exponent(number: &str) -> (&str, Option<&str>) {
    if let Some(index) = number.find(|ch| ch == 'e' || ch == 'E') {
        (&number[..index], Some(&number[index..]))
    } else {
        (number, None)
    }
}

fn split_leading_sign(text: &str) -> (Option<char>, &str) {
    if let Some(sign) = text.chars().next() {
        if sign == '+' || sign == '-' {
            return (Some(sign), &text[sign.len_utf8()..]);
        }
    }
    (None, text)
}

fn split_decimal_parts(number: &str) -> (&str, &str, bool) {
    if let Some(dot_index) = number.find('.') {
        (&number[..dot_index], &number[dot_index + 1..], true)
    } else {
        (number, "", false)
    }
}

fn push_grouped_integer_segments(segments: &mut Vec<NumberSegment>, int_part: &str) {
    if int_part.is_empty() {
        return;
    }

    let chars: Vec<char> = int_part.chars().collect();
    let len = chars.len();
    let first_group_len = if len % 3 == 0 { 3 } else { len % 3 };
    let mut index = 0;
    let mut first = true;

    while index < len {
        let group_len = if first { first_group_len } else { 3 };
        let text: String = chars[index..index + group_len].iter().collect();
        segments.push(NumberSegment {
            text,
            pad_before: !first,
        });
        first = false;
        index += group_len;
    }
}

fn push_grouped_fraction_segments(segments: &mut Vec<NumberSegment>, frac_part: &str) {
    if frac_part.is_empty() {
        return;
    }

    let chars: Vec<char> = frac_part.chars().collect();
    let mut index = 0;
    let mut first = true;
    while index < chars.len() {
        let end = (index + 3).min(chars.len());
        let text: String = chars[index..end].iter().collect();
        segments.push(NumberSegment {
            text,
            pad_before: !first,
        });
        first = false;
        index = end;
    }
}

fn section_label(text: &str) -> gpui::Div {
    div().text_color(theme::fg()).child(text.to_string())
}

fn text_label(text: &str) -> gpui::Div {
    div()
        .text_color(theme::syntax_dim())
        .child(text.to_string())
}

fn icon_button(
    id: &str,
    icon: IconName,
    on_click: impl Fn(&gpui::ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id.to_string())
        .size(px(32.0))
        .flex()
        .items_center()
        .justify_center()
        .bg(theme::panel())
        .border(px(1.0))
        .border_color(theme::separator())
        .rounded(px(8.0))
        .hover(|this| this.bg(theme::hover()))
        .on_click(on_click)
        .child(
            Icon::new(icon)
                .with_size(UiSize::Size(px(18.0)))
                .text_color(theme::fg()),
        )
}

#[cfg(test)]
mod tests {
    use super::split_number_segments;

    fn to_grouped_text(input: &str) -> String {
        let mut out = String::new();
        for segment in split_number_segments(input) {
            if segment.pad_before {
                out.push(' ');
            }
            out.push_str(&segment.text);
        }
        out
    }

    #[test]
    fn groups_integer_digits_every_three() {
        assert_eq!(to_grouped_text("160000"), "160 000");
        assert_eq!(to_grouped_text("123456789"), "123 456 789");
    }

    #[test]
    fn groups_fraction_digits_every_three() {
        assert_eq!(to_grouped_text("0.004222"), "0.004 222");
        assert_eq!(to_grouped_text(".123456"), ".123 456");
    }

    #[test]
    fn keeps_sign_and_exponent_ungrouped() {
        assert_eq!(to_grouped_text("-1234567.89012"), "-1 234 567.890 12");
        assert_eq!(to_grouped_text("1.234567e-10"), "1.234 567e-10");
    }
}
