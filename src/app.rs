use crate::calc::{prepend_ans_if_leading_operator, Entry, Evaluator, History};
use crate::prefs::Preferences;
use crate::syntax::{tokenize, TokenKind};
use crate::theme;
use gpui::prelude::*;
use gpui::{div, px, App, Context, Entity, ScrollHandle, Subscription, Window};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{Icon, IconName, Sizable};
use gpui_component::scroll::ScrollableElement;
use gpui_component::Size as UiSize;
use gpui_component::{h_flex, v_flex};

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
    history_scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
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

        let _subscriptions = vec![cx.subscribe(&input, |this: &mut Self, _input, ev, cx| {
            match ev {
                InputEvent::Change => {
                    this.input_value = this.input.read(cx).value().to_string();
                }
                InputEvent::PressEnter { .. } => {
                    this.submit();
                    cx.notify();
                }
                _ => {}
            }
        })];

        input.update(cx, |state, cx| {
            state.set_placeholder("Type an expression", window, cx);
        });

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
            pending_scroll_to_bottom: false,
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
            div()
                .text_color(theme::error())
                .child(format!("✕ {}", err))
        } else {
            let mut result_tokens = vec![crate::syntax::Token {
                kind: TokenKind::Operator,
                text: "= ".to_string(),
            }];
            result_tokens.extend(tokenize(&entry.result));
            tokens_to_line(&result_tokens)
        };

        v_flex()
            .gap(px(2.0))
            .child(expr_line)
            .child(result_line)
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
    history
        .entries()
        .iter()
        .rev()
        .find_map(|entry| {
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

        let mut history_list = v_flex().gap(px(12.0));
        for (idx, entry) in self.history.entries().iter().enumerate() {
            history_list = history_list.child(self.history_row(entry));
            if idx + 1 < self.history.len() {
                history_list = history_list.child(div().h(px(1.0)).bg(theme::separator()));
            }
        }

        let history_scroll = div()
            .id("history-scroll")
            .flex_1()
            .w_full()
            .min_h(px(0.0))
            .track_scroll(&self.history_scroll)
            .overflow_y_scroll()
            .child(history_list)
            .vertical_scrollbar(&self.history_scroll);

        let input_area = v_flex()
            .gap(px(8.0))
            .child(div().h(px(1.0)).bg(theme::separator()))
            .child(div().bg(theme::input()).p(px(10.0)).child(input));

        let main_content = v_flex()
            .gap(px(10.0))
            .p(px(16.0))
            .bg(theme::bg())
            .flex_1()
            .w_full()
            .h_full()
            .min_h(px(0.0))
            .child(error_line)
            .child(history_scroll)
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
            div()
                .size_full()
                .child(main_content)
                .child(
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
        line = line.child(div().text_color(color).child(token.text.clone()));
    }
    line
}

fn section_label(text: &str) -> gpui::Div {
    div()
        .text_color(theme::fg())
        .child(text.to_string())
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
